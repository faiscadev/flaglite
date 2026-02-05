//! FlagLite CLI - Feature Flag Management
//!
//! A command-line tool for managing feature flags with FlagLite.

mod commands;
mod config;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::{auth, envs, flags, projects};

#[derive(Parser)]
#[command(
    name = "flaglite",
    author = "Faísca <team@faisca.dev>",
    version,
    about = "Feature flag management CLI",
    long_about = "FlagLite CLI - Manage feature flags from your terminal.\n\n\
                  Get started:\n  \
                  flaglite signup\n  \
                  flaglite projects list\n  \
                  flaglite flags list"
)]
struct Cli {
    /// Output format
    #[arg(long, global = true, default_value = "pretty")]
    format: output::OutputFormat,

    /// API base URL (overrides config)
    #[arg(long, global = true, env = "FLAGLITE_API_URL")]
    api_url: Option<String>,

    /// API key for authentication
    #[arg(long, global = true, env = "FLAGLITE_API_KEY")]
    api_key: Option<String>,

    /// Project ID (overrides config)
    #[arg(long, short = 'p', global = true, env = "FLAGLITE_PROJECT")]
    project: Option<String>,

    /// Environment (overrides config)
    #[arg(long, short = 'e', global = true, env = "FLAGLITE_ENV")]
    env: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new FlagLite account
    Signup {
        /// Username (optional, auto-generated if not provided)
        #[arg(long, short)]
        username: Option<String>,
        /// Password (for non-interactive use)
        #[arg(long)]
        password: Option<String>,
    },

    /// Authenticate with FlagLite
    Login {
        /// Username
        #[arg(long, short)]
        username: Option<String>,
        /// Password (for non-interactive use)
        #[arg(long)]
        password: Option<String>,
    },

    /// Clear stored authentication
    Logout,

    /// Show current user information
    Whoami,

    /// Manage projects
    #[command(subcommand)]
    Projects(ProjectsCommands),

    /// Manage feature flags
    #[command(subcommand)]
    Flags(FlagsCommands),

    /// Manage environments
    #[command(subcommand)]
    Envs(EnvsCommands),

    /// Show or edit configuration
    Config {
        /// Show config file path
        #[arg(long)]
        path: bool,
    },
}

#[derive(Subcommand)]
enum ProjectsCommands {
    /// List all projects
    List,
    /// Create a new project
    Create {
        /// Project name
        name: String,
        /// Project description
        #[arg(long, short)]
        description: Option<String>,
    },
    /// Set the default project
    Use {
        /// Project ID or slug
        project: String,
    },
}

#[derive(Subcommand)]
enum FlagsCommands {
    /// List all flags in the current project
    List,
    /// Create a new flag
    Create {
        /// Flag key (unique identifier)
        key: String,
        /// Display name
        #[arg(long, short)]
        name: Option<String>,
        /// Description
        #[arg(long, short)]
        description: Option<String>,
        /// Flag type (boolean, string, number, json)
        #[arg(long, short = 't', default_value = "boolean")]
        flag_type: String,
        /// Enable flag immediately
        #[arg(long)]
        enabled: bool,
    },
    /// Get details for a specific flag
    Get {
        /// Flag key
        key: String,
    },
    /// Toggle a flag on/off
    Toggle {
        /// Flag key
        key: String,
    },
    /// Delete a flag
    Delete {
        /// Flag key
        key: String,
        /// Skip confirmation
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum EnvsCommands {
    /// List all environments
    List,
    /// Set the default environment
    Use {
        /// Environment name or slug
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let output = output::Output::new(cli.format);

    // Load config
    let mut config = config::Config::load()?;

    // Apply CLI overrides
    if let Some(url) = cli.api_url {
        config.api_url = url;
    }
    if let Some(key) = cli.api_key {
        config.api_key = Some(key);
    }
    if let Some(project) = cli.project {
        config.project_id = Some(project);
    }
    if let Some(env) = cli.env {
        config.environment = Some(env);
    }

    let result = match cli.command {
        Commands::Signup { username, password } => {
            auth::signup(&mut config, &output, username, password).await
        }
        Commands::Login { username, password } => {
            auth::login(&mut config, &output, username, password).await
        }
        Commands::Logout => auth::logout(&mut config, &output).await,
        Commands::Whoami => auth::whoami(&config, &output).await,

        Commands::Projects(cmd) => match cmd {
            ProjectsCommands::List => projects::list(&config, &output).await,
            ProjectsCommands::Create { name, description } => {
                projects::create(&config, &output, name, description).await
            }
            ProjectsCommands::Use { project } => {
                projects::use_project(&mut config, &output, project).await
            }
        },

        Commands::Flags(cmd) => match cmd {
            FlagsCommands::List => flags::list(&config, &output).await,
            FlagsCommands::Create {
                key,
                name,
                description,
                flag_type,
                enabled,
            } => flags::create(&config, &output, key, name, description, flag_type, enabled).await,
            FlagsCommands::Get { key } => flags::get(&config, &output, key).await,
            FlagsCommands::Toggle { key } => flags::toggle(&config, &output, key).await,
            FlagsCommands::Delete { key, yes } => flags::delete(&config, &output, key, yes).await,
        },

        Commands::Envs(cmd) => match cmd {
            EnvsCommands::List => envs::list(&config, &output).await,
            EnvsCommands::Use { name } => envs::use_env(&mut config, &output, name).await,
        },

        Commands::Config { path } => {
            if path {
                println!("{}", config::Config::config_path()?.display());
            } else {
                output.print_config(&config)?;
            }
            Ok(())
        }
    };

    if let Err(e) = result {
        output.print_error(&e);
        std::process::exit(1);
    }

    Ok(())
}
