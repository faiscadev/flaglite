//! Output formatting for FlagLite CLI

use crate::config::Config;
use anyhow::Result;
use colored::*;
use flaglite_shared::{Environment, Flag, FlagWithState, Project, User};
use serde::Serialize;
use std::str::FromStr;
use tabled::{Table, Tabled, settings::Style};

/// Output format
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    #[default]
    Pretty,
    Json,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pretty" | "table" => Ok(OutputFormat::Pretty),
            "json" => Ok(OutputFormat::Json),
            _ => Err(format!("Unknown format: {}. Use 'pretty' or 'json'.", s)),
        }
    }
}

/// Output handler
pub struct Output {
    format: OutputFormat,
}

impl Output {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    pub fn is_json(&self) -> bool {
        matches!(self.format, OutputFormat::Json)
    }

    /// Print a success message
    pub fn success(&self, message: &str) {
        if !self.is_json() {
            println!("{} {}", "✓".green().bold(), message);
        }
    }

    /// Print an info message
    pub fn info(&self, message: &str) {
        if !self.is_json() {
            println!("{} {}", "ℹ".blue().bold(), message);
        }
    }

    /// Print a warning message
    pub fn warn(&self, message: &str) {
        if !self.is_json() {
            println!("{} {}", "⚠".yellow().bold(), message);
        }
    }

    /// Print an error
    pub fn print_error(&self, error: &anyhow::Error) {
        if self.is_json() {
            let err = serde_json::json!({ "error": error.to_string() });
            println!("{}", serde_json::to_string_pretty(&err).unwrap());
        } else {
            eprintln!("{} {}", "✗".red().bold(), error);

            // Print chain
            for cause in error.chain().skip(1) {
                eprintln!("  {} {}", "caused by:".dimmed(), cause);
            }
        }
    }

    /// Print JSON output
    pub fn json<T: Serialize + ?Sized>(&self, value: &T) -> Result<()> {
        println!("{}", serde_json::to_string_pretty(value)?);
        Ok(())
    }

    /// Print user info
    pub fn print_user(&self, user: &User) -> Result<()> {
        if self.is_json() {
            return self.json(user);
        }

        println!("{}", "User Information".bold().underline());
        println!("  {} {}", "Email:".dimmed(), user.email.cyan());
        println!("  {} {}", "Name:".dimmed(), user.name);
        println!("  {} {}", "ID:".dimmed(), user.id.to_string().dimmed());
        println!(
            "  {} {}",
            "Member since:".dimmed(),
            user.created_at.format("%Y-%m-%d")
        );

        Ok(())
    }

    /// Print project list
    pub fn print_projects(&self, projects: &[Project], current: Option<&str>) -> Result<()> {
        if self.is_json() {
            return self.json(projects);
        }

        if projects.is_empty() {
            self.info("No projects found. Create one with 'flaglite projects create <name>'");
            return Ok(());
        }

        #[derive(Tabled)]
        struct ProjectRow {
            #[tabled(rename = "")]
            current: String,
            #[tabled(rename = "ID")]
            id: String,
            #[tabled(rename = "Name")]
            name: String,
            #[tabled(rename = "Slug")]
            slug: String,
            #[tabled(rename = "Created")]
            created: String,
        }

        let rows: Vec<_> = projects
            .iter()
            .map(|p| {
                let is_current = current.is_some_and(|c| c == p.id.to_string() || c == p.slug);
                ProjectRow {
                    current: if is_current { "→".green().to_string() } else { "".to_string() },
                    id: p.id.to_string()[..8].to_string(),
                    name: p.name.clone(),
                    slug: p.slug.clone(),
                    created: p.created_at.format("%Y-%m-%d").to_string(),
                }
            })
            .collect();

        let table = Table::new(rows).with(Style::rounded()).to_string();
        println!("{table}");

        Ok(())
    }

    /// Print a single project
    pub fn print_project(&self, project: &Project) -> Result<()> {
        if self.is_json() {
            return self.json(project);
        }

        println!("{}", "Project Created".bold().green());
        println!("  {} {}", "ID:".dimmed(), project.id.to_string().cyan());
        println!("  {} {}", "Name:".dimmed(), project.name);
        println!("  {} {}", "Slug:".dimmed(), project.slug);
        if let Some(desc) = &project.description {
            println!("  {} {}", "Description:".dimmed(), desc);
        }

        Ok(())
    }

    /// Print environment list
    pub fn print_environments(&self, envs: &[Environment], current: Option<&str>) -> Result<()> {
        if self.is_json() {
            return self.json(envs);
        }

        if envs.is_empty() {
            self.info("No environments found.");
            return Ok(());
        }

        #[derive(Tabled)]
        struct EnvRow {
            #[tabled(rename = "")]
            current: String,
            #[tabled(rename = "Name")]
            name: String,
            #[tabled(rename = "Slug")]
            slug: String,
            #[tabled(rename = "Production")]
            production: String,
        }

        let rows: Vec<_> = envs
            .iter()
            .map(|e| {
                let is_current = current.is_some_and(|c| c == e.name || c == e.slug);
                EnvRow {
                    current: if is_current { "→".green().to_string() } else { "".to_string() },
                    name: e.name.clone(),
                    slug: e.slug.clone(),
                    production: if e.is_production {
                        "●".red().to_string()
                    } else {
                        "".to_string()
                    },
                }
            })
            .collect();

        let table = Table::new(rows).with(Style::rounded()).to_string();
        println!("{table}");

        Ok(())
    }

    /// Print flag list
    pub fn print_flags(&self, flags: &[FlagWithState]) -> Result<()> {
        if self.is_json() {
            return self.json(flags);
        }

        if flags.is_empty() {
            self.info("No flags found. Create one with 'flaglite flags create <key>'");
            return Ok(());
        }

        #[derive(Tabled)]
        struct FlagRow {
            #[tabled(rename = "Status")]
            status: String,
            #[tabled(rename = "Key")]
            key: String,
            #[tabled(rename = "Name")]
            name: String,
            #[tabled(rename = "Type")]
            flag_type: String,
            #[tabled(rename = "Updated")]
            updated: String,
        }

        let rows: Vec<_> = flags
            .iter()
            .map(|f| FlagRow {
                status: if f.enabled {
                    "●".green().to_string()
                } else {
                    "○".dimmed().to_string()
                },
                key: f.flag.key.clone(),
                name: f.flag.name.clone(),
                flag_type: f.flag.flag_type.to_string(),
                updated: f.flag.updated_at.format("%Y-%m-%d %H:%M").to_string(),
            })
            .collect();

        let table = Table::new(rows).with(Style::rounded()).to_string();
        println!("{table}");

        Ok(())
    }

    /// Print flag details
    pub fn print_flag(&self, flag: &FlagWithState) -> Result<()> {
        if self.is_json() {
            return self.json(flag);
        }

        let status = if flag.enabled {
            "ENABLED".green().bold()
        } else {
            "DISABLED".red().bold()
        };

        println!("{} {}", flag.flag.key.bold(), status);
        println!();
        println!("  {} {}", "Name:".dimmed(), flag.flag.name);
        println!("  {} {}", "Type:".dimmed(), flag.flag.flag_type);

        if let Some(desc) = &flag.flag.description {
            println!("  {} {}", "Description:".dimmed(), desc);
        }

        if let Some(value) = &flag.value {
            println!(
                "  {} {}",
                "Value:".dimmed(),
                serde_json::to_string(value).unwrap_or_default().cyan()
            );
        }

        println!("  {} {}", "ID:".dimmed(), flag.flag.id.to_string().dimmed());
        println!(
            "  {} {}",
            "Created:".dimmed(),
            flag.flag.created_at.format("%Y-%m-%d %H:%M")
        );
        println!(
            "  {} {}",
            "Updated:".dimmed(),
            flag.flag.updated_at.format("%Y-%m-%d %H:%M")
        );

        Ok(())
    }

    /// Print a single flag (without state)
    pub fn print_flag_created(&self, flag: &Flag) -> Result<()> {
        if self.is_json() {
            return self.json(flag);
        }

        println!("{}", "Flag Created".bold().green());
        println!("  {} {}", "Key:".dimmed(), flag.key.cyan());
        println!("  {} {}", "Name:".dimmed(), flag.name);
        println!("  {} {}", "Type:".dimmed(), flag.flag_type);
        if let Some(desc) = &flag.description {
            println!("  {} {}", "Description:".dimmed(), desc);
        }

        Ok(())
    }

    /// Print config
    pub fn print_config(&self, config: &Config) -> Result<()> {
        if self.is_json() {
            // Don't expose token in JSON output
            let safe = serde_json::json!({
                "api_url": config.api_url,
                "project_id": config.project_id,
                "environment": config.environment,
                "authenticated": config.is_authenticated(),
            });
            return self.json(&safe);
        }

        println!("{}", "Configuration".bold().underline());
        println!("  {} {}", "API URL:".dimmed(), config.api_url.cyan());
        println!(
            "  {} {}",
            "Authenticated:".dimmed(),
            if config.is_authenticated() {
                "Yes".green()
            } else {
                "No".red()
            }
        );
        println!(
            "  {} {}",
            "Project:".dimmed(),
            config.project_id.as_deref().unwrap_or("-").to_string()
        );
        println!(
            "  {} {}",
            "Environment:".dimmed(),
            config.environment.as_deref().unwrap_or("development")
        );
        println!();
        println!(
            "  {} {}",
            "Config file:".dimmed(),
            Config::config_path()?.display()
        );

        Ok(())
    }
}
