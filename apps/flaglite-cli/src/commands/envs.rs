//! Environment management commands

use crate::config::Config;
use crate::output::Output;
use anyhow::Result;
use flaglite_client::FlagLiteClient;

/// Create an authenticated client from config
fn client_from_config(config: &Config) -> Result<FlagLiteClient> {
    let client = FlagLiteClient::new(&config.api_url);

    // Prefer API key over token
    if let Some(api_key) = &config.api_key {
        Ok(client.with_api_key(api_key))
    } else if let Some(token) = &config.token {
        Ok(client.with_token(token))
    } else {
        Err(anyhow::anyhow!(
            "Not logged in. Run `flaglite signup` or `flaglite login`"
        ))
    }
}

/// List all environments
pub async fn list(config: &Config, output: &Output) -> Result<()> {
    let client = client_from_config(config)?;
    let project_id = config.require_project()?;

    let envs = client.list_environments(project_id).await?;

    output.print_environments(&envs, config.environment.as_deref())?;

    Ok(())
}

/// Set the default environment
pub async fn use_env(config: &mut Config, output: &Output, name: String) -> Result<()> {
    let client = client_from_config(config)?;
    let project_id = config.require_project()?;

    // Verify environment exists
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
