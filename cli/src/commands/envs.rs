//! Environment management commands

use crate::config::Config;
use crate::output::Output;
use anyhow::Result;
use flaglite_shared::FlagLiteClient;

/// List all environments
pub async fn list(config: &Config, output: &Output) -> Result<()> {
    let token = config.require_token()?;
    let project_id = config.require_project()?;

    let client = FlagLiteClient::new(&config.api_url).with_token(token);
    let envs = client.list_environments(project_id).await?;

    output.print_environments(&envs, config.environment.as_deref())?;

    Ok(())
}

/// Set the default environment
pub async fn use_env(config: &mut Config, output: &Output, name: String) -> Result<()> {
    let token = config.require_token()?;
    let project_id = config.require_project()?;

    // Verify environment exists
    let client = FlagLiteClient::new(&config.api_url).with_token(token);
    let envs = client.list_environments(project_id).await?;

    let found = envs.iter().find(|e| e.name == name || e.slug == name);

    match found {
        Some(e) => {
            config.environment = Some(e.slug.clone());
            config.save()?;
            output.success(&format!("Now using environment: {}", e.name));
        }
        None => {
            return Err(anyhow::anyhow!(
                "Environment '{}' not found. Run 'flaglite envs list' to see available environments.",
                name
            ));
        }
    }

    Ok(())
}
