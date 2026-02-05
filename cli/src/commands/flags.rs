//! Flag management commands

use crate::config::Config;
use crate::output::Output;
use anyhow::Result;
use dialoguer::Confirm;
use flaglite_shared::{CreateFlagRequest, FlagLiteClient, FlagType};

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

/// List all flags in the current project
pub async fn list(config: &Config, output: &Output) -> Result<()> {
    let client = client_from_config(config)?;
    let project_id = config.require_project()?;
    let env = config.get_environment();

    let flags = client.list_flags(project_id, Some(env)).await?;

    if !output.is_json() {
        output.info(&format!("Flags in environment: {}", env));
    }

    output.print_flags(&flags)?;

    Ok(())
}

/// Create a new flag
pub async fn create(
    config: &Config,
    output: &Output,
    key: String,
    name: Option<String>,
    description: Option<String>,
    flag_type: String,
    enabled: bool,
) -> Result<()> {
    let client = client_from_config(config)?;
    let project_id = config.require_project()?;

    // Parse flag type
    let flag_type = match flag_type.to_lowercase().as_str() {
        "boolean" | "bool" => FlagType::Boolean,
        "string" | "str" => FlagType::String,
        "number" | "num" | "int" | "float" => FlagType::Number,
        "json" | "object" => FlagType::Json,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid flag type: '{}'. Use: boolean, string, number, or json",
                flag_type
            ));
        }
    };

    // Default name to key if not provided
    let name = name.unwrap_or_else(|| {
        // Convert key to title case: my_feature -> My Feature
        key.replace(['_', '-'], " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    });

    let req = CreateFlagRequest {
        key,
        name,
        description,
        flag_type,
        enabled,
    };

    let flag = client.create_flag(project_id, req).await?;

    output.print_flag_created(&flag)?;

    Ok(())
}

/// Get flag details
pub async fn get(config: &Config, output: &Output, key: String) -> Result<()> {
    let client = client_from_config(config)?;
    let project_id = config.require_project()?;
    let env = config.get_environment();

    let flag = client.get_flag(project_id, &key, Some(env)).await?;

    output.print_flag(&flag)?;

    Ok(())
}

/// Toggle a flag
pub async fn toggle(config: &Config, output: &Output, key: String) -> Result<()> {
    let client = client_from_config(config)?;
    let project_id = config.require_project()?;
    let env = config.get_environment();

    let flag = client.toggle_flag(project_id, &key, env).await?;

    let status = if flag.enabled { "enabled" } else { "disabled" };
    output.success(&format!("Flag '{}' is now {} in {}", key, status, env));

    Ok(())
}

/// Delete a flag
pub async fn delete(config: &Config, output: &Output, key: String, yes: bool) -> Result<()> {
    let client = client_from_config(config)?;
    let project_id = config.require_project()?;

    // Confirm deletion unless --yes flag is provided
    if !yes && !output.is_json() {
        let confirmed = Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete flag '{}'? This cannot be undone.",
                key
            ))
            .default(false)
            .interact()?;

        if !confirmed {
            output.info("Deletion cancelled.");
            return Ok(());
        }
    }

    client.delete_flag(project_id, &key).await?;

    output.success(&format!("Flag '{}' deleted.", key));

    Ok(())
}
