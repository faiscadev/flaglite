//! Authentication commands

use crate::config::Config;
use crate::output::Output;
use anyhow::Result;
use dialoguer::{Input, Password};
use flaglite_client::FlagLiteClient;

/// Sign up for FlagLite
pub async fn signup(
    config: &mut Config,
    output: &Output,
    cli_username: Option<String>,
    cli_password: Option<String>,
) -> Result<()> {
    // Determine if we're in interactive mode
    let is_interactive = cli_password.is_none();

    if is_interactive && output.is_json() {
        return Err(anyhow::anyhow!(
            "Interactive signup not supported with --format=json. Use --password for non-interactive mode."
        ));
    }

    let (username, password) = if is_interactive {
        // Interactive mode - prompt for input
        let username: String = Input::new()
            .with_prompt("Username (leave empty for auto-generated)")
            .allow_empty(true)
            .interact_text()?;

        let username = if username.trim().is_empty() {
            None
        } else {
            Some(username.trim().to_string())
        };

        let password: String = Password::new().with_prompt("Password").interact()?;
        let password_confirm: String =
            Password::new().with_prompt("Confirm password").interact()?;

        if password != password_confirm {
            return Err(anyhow::anyhow!("Passwords do not match"));
        }

        (username, password)
    } else {
        // Non-interactive mode - use CLI arguments
        let password = cli_password.unwrap();
        (cli_username, password)
    };

    if password.len() < 8 {
        return Err(anyhow::anyhow!("Password must be at least 8 characters"));
    }

    // Call signup endpoint
    let client = FlagLiteClient::new(&config.api_url);
    let response = client.signup(username.as_deref(), &password).await?;

    // Save credentials
    config.token = Some(response.token);
    config.api_key = Some(response.api_key.key.clone());
    config.username = Some(response.user.username.clone());
    
    // Save default project if provided
    if let Some(ref project) = response.project {
        config.project_id = Some(project.id.to_string());
    }
    
    config.save_credentials()?;

    if output.is_json() {
        // JSON output for scripting
        let json = serde_json::json!({
            "username": response.user.username,
            "api_key": response.api_key.key,
            "user_id": response.user.id,
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        output.success(&format!(
            "Account created successfully!\n  Username: {}\n  API Key: {}",
            response.user.username, response.api_key.key
        ));
    }

    Ok(())
}

/// Log in to FlagLite
pub async fn login(
    config: &mut Config,
    output: &Output,
    cli_username: Option<String>,
    cli_password: Option<String>,
) -> Result<()> {
    // Determine if we're in interactive mode
    let is_interactive = cli_username.is_none() || cli_password.is_none();

    if is_interactive && config.is_authenticated() && !output.is_json() {
        output.warn("You are already logged in. This will replace your current session.");
    }

    if is_interactive && output.is_json() {
        return Err(anyhow::anyhow!(
            "Interactive login not supported with --format=json. Use --username and --password."
        ));
    }

    let (username, password) = if is_interactive {
        // Interactive mode - prompt for input
        let username = if let Some(u) = cli_username {
            u
        } else {
            Input::new().with_prompt("Username").interact_text()?
        };

        let password = if let Some(p) = cli_password {
            p
        } else {
            Password::new().with_prompt("Password").interact()?
        };

        (username, password)
    } else {
        // Non-interactive mode
        (cli_username.unwrap(), cli_password.unwrap())
    };

    // Authenticate
    let client = FlagLiteClient::new(&config.api_url);
    let response = client.login(&username, &password).await?;

    // Save credentials
    config.token = Some(response.token);
    config.username = Some(response.user.username.clone());
    config.save_credentials()?;

    if output.is_json() {
        let json = serde_json::json!({
            "username": response.user.username,
            "user_id": response.user.id,
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        output.success(&format!("Logged in as {}", response.user.username));
    }

    Ok(())
}

/// Log out of FlagLite
pub async fn logout(config: &mut Config, output: &Output) -> Result<()> {
    if !config.is_authenticated() {
        output.info("You are not logged in.");
        return Ok(());
    }

    config.clear_auth();
    Config::delete_credentials()?;

    output.success("Logged out");

    Ok(())
}

/// Show current user
pub async fn whoami(config: &Config, output: &Output) -> Result<()> {
    let token = config.require_token()?;

    let client = if config.api_key.is_some() {
        FlagLiteClient::new(&config.api_url).with_api_key(token)
    } else {
        FlagLiteClient::new(&config.api_url).with_token(token)
    };

    let user = client.whoami().await?;

    output.print_user(&user)?;

    Ok(())
}
