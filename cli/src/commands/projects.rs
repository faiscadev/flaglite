//! Project management commands

use crate::config::Config;
use crate::output::Output;
use anyhow::Result;
use flaglite_shared::{CreateProjectRequest, FlagLiteClient};

/// List all projects
pub async fn list(config: &Config, output: &Output) -> Result<()> {
    let token = config.require_token()?;

    let client = FlagLiteClient::new(&config.api_url).with_token(token);
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
    let token = config.require_token()?;

    let client = FlagLiteClient::new(&config.api_url).with_token(token);

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
    let token = config.require_token()?;

    // Verify project exists by listing projects
    let client = FlagLiteClient::new(&config.api_url).with_token(token);
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
                "Project '{}' not found. Run 'flaglite projects list' to see available projects.",
                project
            ));
        }
    }

    Ok(())
}
