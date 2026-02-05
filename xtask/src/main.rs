//! Development tasks for the FlagLite workspace.
//!
//! Usage: `cargo xtask <command>`

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::process::{Command, ExitStatus};

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Development tasks for FlagLite workspace")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all checks: fmt, clippy, and test
    Check,

    /// Run cargo fmt on the workspace
    Fmt {
        /// Check formatting without modifying files
        #[arg(long)]
        check: bool,
    },

    /// Run clippy lints on the workspace
    Lint,

    /// Run tests for the workspace
    Test {
        /// Package to test (omit for all)
        #[arg(short, long)]
        package: Option<String>,
    },

    /// Run tests with coverage (requires cargo-llvm-cov)
    Coverage,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check => cmd_check(),
        Commands::Fmt { check } => cmd_fmt(check),
        Commands::Lint => cmd_lint(),
        Commands::Test { package } => cmd_test(package),
        Commands::Coverage => cmd_coverage(),
    }
}

/// Run all checks: fmt --check, clippy, and test
fn cmd_check() -> Result<()> {
    println!("🔍 Running all checks...\n");

    println!("📝 Checking formatting...");
    run_cargo(&["fmt", "--all", "--", "--check"])?;
    println!("✓ Formatting OK\n");

    println!("🔎 Running clippy...");
    run_cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ])?;
    println!("✓ Clippy OK\n");

    println!("🧪 Running tests...");
    run_cargo(&["test", "--workspace"])?;
    println!("✓ Tests OK\n");

    println!("✅ All checks passed!");
    Ok(())
}

/// Run cargo fmt
fn cmd_fmt(check: bool) -> Result<()> {
    if check {
        println!("📝 Checking formatting...");
        run_cargo(&["fmt", "--all", "--", "--check"])?;
        println!("✓ Formatting OK");
    } else {
        println!("📝 Formatting code...");
        run_cargo(&["fmt", "--all"])?;
        println!("✓ Code formatted");
    }
    Ok(())
}

/// Run clippy lints
fn cmd_lint() -> Result<()> {
    println!("🔎 Running clippy...");
    run_cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ])?;
    println!("✓ Clippy OK");
    Ok(())
}

/// Run tests
fn cmd_test(package: Option<String>) -> Result<()> {
    println!("🧪 Running tests...");

    let mut args = vec!["test"];

    if let Some(ref pkg) = package {
        args.push("--package");
        args.push(pkg);
    } else {
        args.push("--workspace");
    }

    run_cargo(&args)?;
    println!("✓ Tests OK");
    Ok(())
}

/// Run tests with coverage
fn cmd_coverage() -> Result<()> {
    println!("📊 Running tests with coverage...");
    println!("   (requires cargo-llvm-cov: cargo install cargo-llvm-cov)\n");

    // Check if cargo-llvm-cov is installed
    let check = Command::new("cargo")
        .args(["llvm-cov", "--version"])
        .output();

    if check.is_err() || !check.unwrap().status.success() {
        anyhow::bail!(
            "cargo-llvm-cov is not installed.\n\
             Install it with: cargo install cargo-llvm-cov"
        );
    }

    run_cargo(&["llvm-cov", "--workspace", "--html"])?;

    println!("\n✓ Coverage report generated");
    println!("  Open target/llvm-cov/html/index.html to view");
    Ok(())
}

/// Run a cargo command and check for success
fn run_cargo(args: &[&str]) -> Result<ExitStatus> {
    let status = Command::new("cargo")
        .args(args)
        .status()
        .with_context(|| format!("Failed to run: cargo {}", args.join(" ")))?;

    if !status.success() {
        anyhow::bail!("Command failed: cargo {}", args.join(" "));
    }

    Ok(status)
}
