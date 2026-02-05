//! Project management commands

use crate::config::Config;
use crate::output::Output;
use anyhow::Result;
use flaglite_client::{CreateProjectRequest, FlagLiteClient};

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

/// List all projects
pub async fn list(config: &Config, output: &Output) -> Result<()> {
    let client = client_from_config(config)?;
    let projects = client.list_projects().await?;

    output.print_projects(&projects, config.project_id.as_deref())?;

    Ok(())
}

/// Create a new project
pub async fn create(
    config: &Config,
    output: &Output,
    name: String,
    description: Option<String>,
) -> Result<()> {
    let client = client_from_config(config)?;

    let req = CreateProjectRequest { name, description };
    let project = client.create_project(req).await?;

    output.print_project(&project)?;

    if !output.is_json() {
        output.info(&format!(
            "Set as default with: flaglite projects use {}",
            project.slug
        ));
    }

    Ok(())
}

/// Set the default project
pub async fn use_project(config: &mut Config, output: &Output, project: String) -> Result<()> {
    let client = client_from_config(config)?;
    let projects = client.list_projects().await?;

    let found = projects.iter().find(|p| {
        p.id.to_string() == project || p.slug == project || p.id.to_string().starts_with(&project)
    });

    match found {
        Some(p) => {
            config.project_id = Some(p.id.to_string());
            config.save()?;
            output.success(&format!("Now using project: {} ({})", p.name, p.slug));
        }
        None => {
            return Err(anyhow::anyhow!(
                "Project '{project}' not found. Run 'flaglite projects list' to see available projects.",
            ));
        }
    }

    Ok(())
}
