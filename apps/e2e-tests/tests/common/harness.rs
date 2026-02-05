//! Test harness for black-box e2e tests.
//!
//! Spawns actual server processes and runs CLI commands as subprocesses
//! to test the full stack end-to-end.

use std::fs::{self, File};
use std::io::Read as _;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::time::Duration;
use tokio::time::sleep;

// ═══════════════════════════════════════════════════════════════════════════
// Test Harness
// ═══════════════════════════════════════════════════════════════════════════

/// Test harness that manages server lifecycle and provides test utilities.
///
/// Creates isolated test environments with:
/// - Its own SQLite database
/// - Server process running on a random port
/// - Test users with isolated HOME directories
pub struct TestHarness {
    /// Server URL for CLI connections
    pub server_url: String,
    /// Test directory (temp)
    pub test_dir: PathBuf,
    /// Path to flaglite CLI binary
    pub flaglite_bin: PathBuf,
    /// Path to flaglite-api server binary
    pub flaglite_api_bin: PathBuf,
    /// Server process handle
    server_process: Option<Child>,
    /// Server port
    port: u16,
    /// Database URL (SQLite)
    database_url: String,
    /// Server stdout log file path (for diagnostics)
    server_stdout_path: PathBuf,
    /// Server stderr log file path (for diagnostics)
    server_stderr_path: PathBuf,
}

impl TestHarness {
    /// Create a new test harness.
    ///
    /// This will:
    /// 1. Create a temporary directory for the test
    /// 2. Create a SQLite database
    /// 3. Find an available port
    /// 4. Start the flaglite-api server
    /// 5. Wait for the server to be ready
    pub async fn new(test_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Get binary paths
        let (flaglite_api_bin, flaglite_bin) = get_binary_paths()?;

        // Create test directory
        let test_id = std::process::id();
        let test_dir = std::env::temp_dir().join(format!("flaglite-e2e-{test_name}-{test_id}"));
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir)?;
        }
        fs::create_dir_all(&test_dir)?;

        // Find available port
        let port = find_available_port()?;

        // Setup SQLite database
        let db_path = test_dir.join("test.db");
        let database_url = format!("sqlite://{}?mode=rwc", db_path.display());

        // Create log file paths
        let server_stdout_path = test_dir.join("server_stdout.log");
        let server_stderr_path = test_dir.join("server_stderr.log");

        let mut harness = Self {
            server_url: format!("http://127.0.0.1:{port}"),
            test_dir,
            flaglite_bin,
            flaglite_api_bin,
            server_process: None,
            port,
            database_url,
            server_stdout_path,
            server_stderr_path,
        };

        // Start the server
        harness.start_server().await?;

        Ok(harness)
    }

    /// Start the server process
    async fn start_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let _server_addr = format!("0.0.0.0:{}", self.port);

        // Create files to capture server output
        let stdout_file = File::create(&self.server_stdout_path)?;
        let stderr_file = File::create(&self.server_stderr_path)?;

        // Generate a JWT secret for testing
        let jwt_secret = "test-jwt-secret-for-e2e-tests-12345";

        let server = Command::new(&self.flaglite_api_bin)
            .env("DATABASE_URL", &self.database_url)
            .env("JWT_SECRET", jwt_secret)
            .env("RUST_LOG", "flaglite=debug")
            .args([
                "serve",
                "--port",
                &self.port.to_string(),
                "--host",
                "127.0.0.1",
            ])
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file))
            .spawn()?;

        self.server_process = Some(server);

        // Wait for server to be ready
        self.wait_for_server().await?;

        Ok(())
    }

    /// Wait for the server to become ready
    async fn wait_for_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let timeout_secs: u64 = std::env::var("FLAGLITE_E2E_SERVER_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);

        let health_url = format!("{}/health", self.server_url);
        let client = reqwest::Client::new();

        // Exponential backoff: start at 50ms, double each time, cap at 2s
        let mut delay_ms: u64 = 50;
        let max_delay_ms: u64 = 2000;
        let start = std::time::Instant::now();

        loop {
            sleep(Duration::from_millis(delay_ms)).await;

            // Check if server process exited early
            if let Some(ref mut server) = self.server_process {
                if let Ok(Some(_status)) = server.try_wait() {
                    let stderr = self.read_server_log(&self.server_stderr_path);
                    let stdout = self.read_server_log(&self.server_stdout_path);

                    return Err(format!(
                        "Server exited unexpectedly\n\
                         Server stdout:\n{}\n\
                         Server stderr:\n{}",
                        stdout.unwrap_or_else(|e| format!("<failed to read: {e}>")),
                        stderr.unwrap_or_else(|e| format!("<failed to read: {e}>"))
                    )
                    .into());
                }
            }

            if let Ok(resp) = client.get(&health_url).send().await {
                if resp.status().is_success() {
                    return Ok(());
                }
            }

            let elapsed = start.elapsed();
            if elapsed.as_secs() >= timeout_secs {
                // Shutdown server and collect diagnostics
                if let Some(ref mut server) = self.server_process {
                    let _ = server.kill();
                }

                let stdout = self.read_server_log(&self.server_stdout_path);
                let stderr = self.read_server_log(&self.server_stderr_path);

                return Err(format!(
                    "Server failed to become ready within {} seconds\n\
                     Server stdout:\n{}\n\
                     Server stderr:\n{}",
                    timeout_secs,
                    stdout.unwrap_or_else(|e| format!("<failed to read: {e}>")),
                    stderr.unwrap_or_else(|e| format!("<failed to read: {e}>"))
                )
                .into());
            }

            // Exponential backoff with cap
            delay_ms = (delay_ms * 2).min(max_delay_ms);
        }
    }

    /// Read server log file contents (last 100 lines max)
    fn read_server_log(&self, path: &PathBuf) -> Result<String, std::io::Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let lines: Vec<&str> = contents.lines().collect();
        if lines.len() > 100 {
            Ok(format!(
                "... ({} lines truncated) ...\n{}",
                lines.len() - 100,
                lines[lines.len() - 100..].join("\n")
            ))
        } else {
            Ok(contents)
        }
    }

    /// Create a test user with isolated HOME directory.
    ///
    /// Each user gets their own HOME directory so credentials are isolated.
    pub fn create_user(&self, name: &str) -> TestUser {
        let home_dir = self.test_dir.join(format!("user_{name}"));
        fs::create_dir_all(&home_dir).expect("Failed to create user home dir");

        TestUser {
            name: name.to_string(),
            home_dir,
            flaglite_bin: self.flaglite_bin.clone(),
            server_url: self.server_url.clone(),
        }
    }

    /// Get the test directory path
    pub fn test_dir(&self) -> &PathBuf {
        &self.test_dir
    }

    /// Get the database URL
    pub fn database_url(&self) -> &str {
        &self.database_url
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        // Stop the server gracefully
        if let Some(ref mut server) = self.server_process {
            // Try graceful shutdown first with SIGTERM
            #[cfg(unix)]
            {
                unsafe {
                    libc::kill(server.id() as i32, libc::SIGTERM);
                }
                // Give it a moment to shutdown gracefully
                std::thread::sleep(Duration::from_millis(100));
            }

            // Then force kill if still running
            let _ = server.kill();
            let _ = server.wait();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Test User
// ═══════════════════════════════════════════════════════════════════════════

/// Represents a test user with isolated configuration.
///
/// Each user has their own HOME directory, so CLI credentials are isolated.
/// All CLI commands are executed with HOME pointed to this directory.
pub struct TestUser {
    /// User name (for identification in logs)
    pub name: String,
    /// Isolated HOME directory for this user
    pub home_dir: PathBuf,
    /// Path to flaglite CLI binary
    pub flaglite_bin: PathBuf,
    /// Server URL for CLI connections
    pub server_url: String,
}

impl TestUser {
    /// Execute a flaglite CLI command and return the result.
    ///
    /// The command is run with:
    /// - HOME set to this user's isolated directory
    /// - FLAGLITE_API_URL set to the test server
    pub fn exec(&self, args: &[&str]) -> CommandResult {
        let output = Command::new(&self.flaglite_bin)
            .env("HOME", &self.home_dir)
            .env("FLAGLITE_API_URL", &self.server_url)
            .env("XDG_CONFIG_HOME", self.home_dir.join(".config"))
            .args(args)
            .output()
            .expect("Failed to execute command");

        CommandResult::new(output)
    }

    /// Execute a flaglite CLI command with JSON output format.
    pub fn exec_json(&self, args: &[&str]) -> CommandResult {
        let mut full_args = vec!["--format", "json"];
        full_args.extend(args);
        self.exec(&full_args)
    }

    /// Get the raw Output for cases needing more control.
    pub fn raw_exec(&self, args: &[&str]) -> Output {
        Command::new(&self.flaglite_bin)
            .env("HOME", &self.home_dir)
            .env("FLAGLITE_API_URL", &self.server_url)
            .env("XDG_CONFIG_HOME", self.home_dir.join(".config"))
            .args(args)
            .output()
            .expect("Failed to execute command")
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Auth Commands
    // ─────────────────────────────────────────────────────────────────────────

    /// Sign up a new user via CLI (non-interactive mode).
    ///
    /// Returns signup info if successful.
    pub fn signup(&self, username: Option<&str>, password: &str) -> Result<SignupInfo, String> {
        let mut args = vec!["signup", "--password", password];

        if let Some(user) = username {
            args.push("--username");
            args.push(user);
        }

        let result = self.exec(&args);

        if result.failed() {
            return Err(format!("Signup failed: {}", result.stderr()));
        }

        let stdout = result.stdout();

        // Parse output to extract username and API key
        // Expected output format:
        // ✓ Account created successfully!
        //   Username: user_xxx
        //   API Key: flg_xxx
        let parsed_username = stdout
            .lines()
            .find(|line| line.contains("Username:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .ok_or_else(|| format!("Failed to parse username from output: {stdout}"))?;

        let api_key = stdout
            .lines()
            .find(|line| line.contains("API Key:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .ok_or_else(|| format!("Failed to parse API key from output: {stdout}"))?;

        Ok(SignupInfo {
            username: parsed_username,
            api_key,
        })
    }

    /// Login with username and password via CLI (non-interactive mode).
    pub fn login(&self, username: &str, password: &str) -> Result<(), String> {
        let result = self.exec(&["login", "--username", username, "--password", password]);

        if result.failed() {
            return Err(format!("Login failed: {}", result.stderr()));
        }

        Ok(())
    }

    /// Get current user info via whoami command.
    pub fn whoami(&self) -> Result<WhoamiInfo, String> {
        let result = self.exec(&["whoami"]);

        if result.failed() {
            return Err(format!("Whoami failed: {}", result.stderr()));
        }

        let stdout = result.stdout();

        // Parse whoami output
        // Expected format:
        // Logged in as: username
        let username = stdout
            .lines()
            .find(|line| line.contains("Logged in as:") || line.contains("Username:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .ok_or("Failed to parse username from whoami output")?;

        Ok(WhoamiInfo { username })
    }

    /// Logout via CLI.
    pub fn logout(&self) -> Result<(), String> {
        let result = self.exec(&["logout"]);

        if result.failed() {
            return Err(format!("Logout failed: {}", result.stderr()));
        }

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Project Commands
    // ─────────────────────────────────────────────────────────────────────────

    /// List projects via CLI.
    pub fn projects_list(&self) -> Result<Vec<ProjectInfo>, String> {
        let result = self.exec_json(&["projects", "list"]);

        let stdout = result.stdout();
        let stderr = result.stderr();

        // Check for error in stdout (JSON mode outputs errors to stdout)
        if stdout.contains("\"error\"") {
            return Err(format!("Projects list failed: {stdout}"));
        }

        if result.failed() {
            return Err(format!("Projects list failed: {stdout} {stderr}"));
        }

        // Try to parse as JSON array
        if let Ok(projects) = serde_json::from_str::<Vec<ProjectInfo>>(&stdout) {
            return Ok(projects);
        }

        // Fallback: parse pretty output
        // Format typically:
        // ID | Name | Slug
        // ---+------+-----
        // xxx | My Project | my-project
        let mut projects = Vec::new();
        let lines: Vec<&str> = stdout.lines().collect();

        for line in lines.iter().skip(2) {
            // Skip header and separator
            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 3 {
                projects.push(ProjectInfo {
                    id: parts[0].to_string(),
                    name: parts[1].to_string(),
                    slug: parts[2].to_string(),
                });
            }
        }

        Ok(projects)
    }

    /// Create a project via CLI.
    pub fn projects_create(
        &self,
        name: &str,
        description: Option<&str>,
    ) -> Result<ProjectInfo, String> {
        let mut args = vec!["projects", "create", name];
        if let Some(desc) = description {
            args.push("--description");
            args.push(desc);
        }

        let result = self.exec(&args);

        if result.failed() {
            return Err(format!("Projects create failed: {}", result.stderr()));
        }

        let stdout = result.stdout();

        // Parse output to extract project info
        // Expected output:
        // ✓ Project created successfully!
        //   ID: xxx
        //   Name: My Project
        //   Slug: my-project
        let id = stdout
            .lines()
            .find(|line| line.contains("ID:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let project_name = stdout
            .lines()
            .find(|line| line.trim().starts_with("Name:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| name.to_string());

        let slug = stdout
            .lines()
            .find(|line| line.contains("Slug:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        Ok(ProjectInfo {
            id,
            name: project_name,
            slug,
        })
    }

    /// Use (select) a project.
    pub fn projects_use(&self, project: &str) -> Result<(), String> {
        let result = self.exec(&["projects", "use", project]);

        if result.failed() {
            return Err(format!("Projects use failed: {}", result.stderr()));
        }

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Flag Commands
    // ─────────────────────────────────────────────────────────────────────────

    /// List flags via CLI.
    pub fn flags_list(&self) -> Result<Vec<FlagInfo>, String> {
        let result = self.exec_json(&["flags", "list"]);

        if result.failed() {
            return Err(format!("Flags list failed: {}", result.stderr()));
        }

        let stdout = result.stdout();

        // Try to parse as JSON
        if let Ok(flags) = serde_json::from_str::<Vec<FlagInfo>>(&stdout) {
            return Ok(flags);
        }

        // Fallback: parse table output
        let mut flags = Vec::new();
        let lines: Vec<&str> = stdout.lines().collect();

        for line in lines.iter().skip(2) {
            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 4 {
                flags.push(FlagInfo {
                    key: parts[0].to_string(),
                    name: parts[1].to_string(),
                    flag_type: parts[2].to_string(),
                    enabled: parts[3].to_lowercase().contains("true")
                        || parts[3].contains("✓")
                        || parts[3].contains("on"),
                });
            }
        }

        Ok(flags)
    }

    /// Create a flag via CLI.
    pub fn flags_create(
        &self,
        key: &str,
        name: Option<&str>,
        flag_type: Option<&str>,
        enabled: bool,
    ) -> Result<FlagInfo, String> {
        let mut args = vec!["flags", "create", key];

        if let Some(n) = name {
            args.push("--name");
            args.push(n);
        }

        if let Some(t) = flag_type {
            args.push("--flag-type");
            args.push(t);
        }

        if enabled {
            args.push("--enabled");
        }

        let result = self.exec(&args);

        if result.failed() {
            return Err(format!("Flags create failed: {}", result.stderr()));
        }

        let stdout = result.stdout();

        // Parse output
        let flag_key = stdout
            .lines()
            .find(|line| line.contains("Key:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| key.to_string());

        let flag_name = stdout
            .lines()
            .find(|line| line.trim().starts_with("Name:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| name.unwrap_or(key).to_string());

        let ft = stdout
            .lines()
            .find(|line| line.contains("Type:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| flag_type.unwrap_or("boolean").to_string());

        Ok(FlagInfo {
            key: flag_key,
            name: flag_name,
            flag_type: ft,
            enabled,
        })
    }

    /// Get a flag via CLI.
    pub fn flags_get(&self, key: &str) -> Result<FlagInfo, String> {
        let result = self.exec(&["flags", "get", key]);

        if result.failed() {
            return Err(format!("Flags get failed: {}", result.stderr()));
        }

        let stdout = result.stdout();

        let flag_key = stdout
            .lines()
            .find(|line| line.contains("Key:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| key.to_string());

        let name = stdout
            .lines()
            .find(|line| line.trim().starts_with("Name:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let flag_type = stdout
            .lines()
            .find(|line| line.contains("Type:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "boolean".to_string());

        // Output format: "flag_key ENABLED" or "flag_key DISABLED" on first line
        let enabled = stdout
            .lines()
            .next()
            .map(|first_line| first_line.contains("ENABLED"))
            .unwrap_or(false);

        Ok(FlagInfo {
            key: flag_key,
            name,
            flag_type,
            enabled,
        })
    }

    /// Toggle a flag via CLI.
    pub fn flags_toggle(&self, key: &str) -> Result<bool, String> {
        let result = self.exec(&["flags", "toggle", key]);

        if result.failed() {
            return Err(format!("Flags toggle failed: {}", result.stderr()));
        }

        let stdout = result.stdout().to_lowercase();

        // Determine new state from output
        // Output is like: "Flag 'key' is now enabled in development"
        // or "Flag 'key' is now disabled in development"
        // Check for "disabled" first since "disabled" contains "enabled" substring
        let enabled = if stdout.contains("disabled") {
            false
        } else if stdout.contains("enabled") {
            true
        } else {
            // Fallback: check for other indicators
            stdout.contains("on") || stdout.contains("true")
        };

        Ok(enabled)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Environment Commands
    // ─────────────────────────────────────────────────────────────────────────

    /// List environments via CLI.
    pub fn envs_list(&self) -> Result<Vec<EnvInfo>, String> {
        let result = self.exec_json(&["envs", "list"]);

        if result.failed() {
            return Err(format!("Envs list failed: {}", result.stderr()));
        }

        let stdout = result.stdout();

        // Try JSON parsing
        if let Ok(envs) = serde_json::from_str::<Vec<EnvInfo>>(&stdout) {
            return Ok(envs);
        }

        // Fallback: parse table
        let mut envs = Vec::new();
        let lines: Vec<&str> = stdout.lines().collect();

        for line in lines.iter().skip(2) {
            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 2 {
                envs.push(EnvInfo {
                    name: parts[0].to_string(),
                    slug: parts.get(1).map(|s| s.to_string()).unwrap_or_default(),
                });
            }
        }

        Ok(envs)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Command Result
// ═══════════════════════════════════════════════════════════════════════════

/// Result of a CLI command execution.
pub struct CommandResult {
    output: Output,
}

impl CommandResult {
    pub fn new(output: Output) -> Self {
        Self { output }
    }

    /// Check if command succeeded and return stdout.
    pub fn success(self) -> Result<String, Box<dyn std::error::Error>> {
        if !self.output.status.success() {
            return Err(format!(
                "Command failed:\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&self.output.stdout),
                String::from_utf8_lossy(&self.output.stderr)
            )
            .into());
        }
        Ok(String::from_utf8_lossy(&self.output.stdout)
            .trim()
            .to_string())
    }

    /// Check if command succeeded, return error with context.
    pub fn success_or_err(self, context: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.output.status.success() {
            return Err(format!(
                "{} failed:\nstdout: {}\nstderr: {}",
                context,
                String::from_utf8_lossy(&self.output.stdout),
                String::from_utf8_lossy(&self.output.stderr)
            )
            .into());
        }
        Ok(())
    }

    /// Get stdout as string.
    pub fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.output.stdout)
            .trim()
            .to_string()
    }

    /// Get stderr as string.
    pub fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.output.stderr)
            .trim()
            .to_string()
    }

    /// Check if the command failed.
    pub fn failed(&self) -> bool {
        !self.output.status.success()
    }

    /// Check if the command succeeded.
    pub fn succeeded(&self) -> bool {
        self.output.status.success()
    }

    /// Get the exit code.
    pub fn exit_code(&self) -> Option<i32> {
        self.output.status.code()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Data Types
// ═══════════════════════════════════════════════════════════════════════════

/// Info returned from signup.
#[derive(Debug, Clone)]
pub struct SignupInfo {
    pub username: String,
    pub api_key: String,
}

/// Info returned from whoami.
#[derive(Debug, Clone)]
pub struct WhoamiInfo {
    pub username: String,
}

/// Project info parsed from CLI output.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub slug: String,
}

/// Flag info parsed from CLI output.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct FlagInfo {
    pub key: String,
    pub name: String,
    #[serde(alias = "type")]
    pub flag_type: String,
    pub enabled: bool,
}

/// Environment info parsed from CLI output.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct EnvInfo {
    pub name: String,
    pub slug: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════════

/// Find an available TCP port.
fn find_available_port() -> Result<u16, Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

/// Get paths to the flaglite-api and flaglite binaries.
fn get_binary_paths() -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    // Check environment variable overrides first
    if let (Ok(api), Ok(cli)) = (
        std::env::var("FLAGLITE_API_BIN"),
        std::env::var("FLAGLITE_CLI_BIN"),
    ) {
        let api_path = PathBuf::from(&api);
        let cli_path = PathBuf::from(&cli);
        if api_path.exists() && cli_path.exists() {
            return Ok((api_path.canonicalize()?, cli_path.canonicalize()?));
        }
    }

    // Get workspace root from CARGO_MANIFEST_DIR
    // The e2e-tests crate is at: workspace/apps/e2e-tests
    // So binaries are at: workspace/target/debug/
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).ok();

    let cwd = std::env::current_dir()?;

    // Build list of potential base directories
    let mut base_dirs: Vec<PathBuf> = Vec::new();

    // From manifest directory (apps/e2e-tests), go up to workspace root
    if let Some(ref manifest) = manifest_dir {
        // apps/e2e-tests -> apps -> workspace
        if let Some(workspace) = manifest.parent().and_then(|p| p.parent()) {
            base_dirs.push(workspace.to_path_buf());
        }
        base_dirs.push(manifest.clone());
    }

    // From current directory
    base_dirs.push(cwd.clone());

    // Also try going up from cwd (useful if cwd is apps/e2e-tests or workspace)
    if let Some(parent) = cwd.parent() {
        base_dirs.push(parent.to_path_buf());
        if let Some(grandparent) = parent.parent() {
            base_dirs.push(grandparent.to_path_buf());
        }
    }

    // Binary paths relative to workspace root
    let candidates = [
        ("target/debug/flaglite-api", "target/debug/flaglite"),
        ("target/release/flaglite-api", "target/release/flaglite"),
    ];

    for base in &base_dirs {
        for (api_rel, cli_rel) in &candidates {
            let api_path = base.join(api_rel);
            let cli_path = base.join(cli_rel);

            if api_path.exists() && cli_path.exists() {
                return Ok((api_path.canonicalize()?, cli_path.canonicalize()?));
            }
        }
    }

    // Build helpful error message
    let searched: Vec<String> = base_dirs
        .iter()
        .flat_map(|base| {
            candidates
                .iter()
                .map(move |(api, _)| format!("{}/{}", base.display(), api))
        })
        .collect();

    Err(format!(
        "Could not find flaglite-api and flaglite binaries.\n\
         Build them first with: cargo build --bins\n\
         Or set FLAGLITE_API_BIN and FLAGLITE_CLI_BIN environment variables.\n\
         Searched in:\n  {}",
        searched.join("\n  ")
    )
    .into())
}
