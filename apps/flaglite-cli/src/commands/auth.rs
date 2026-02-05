//! Authentication commands

use crate::config::Config;
use crate::output::Output;
use anyhow::Result;
use dialoguer::{Input, Password};
use flaglite_client::FlagLiteClient;

/// Sign up for FlagLite
pub async fn signup(config: &mut Config, output: &Output) -> Result<()> {
    if output.is_json() {
        return Err(anyhow::anyhow!(
            "Interactive signup not supported with --format=json. Use the API directly."
        ));
    }

    // Get username (optional - auto-generated if empty)
    let username: String = Input::new()
        .with_prompt("Username (leave empty for auto-generated)")
        .allow_empty(true)
        .interact_text()?;

    let username = if username.trim().is_empty() {
        None
    } else {
        Some(username.trim().to_string())
    };

    // Get password
    let password: String = Password::new()
        .with_prompt("Password")
        .interact()?;

    // Confirm password
    let password_confirm: String = Password::new()
        .with_prompt("Confirm password")
        .interact()?;

    if password != password_confirm {
        return Err(anyhow::anyhow!("Passwords do not match"));
    }

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
    config.save_credentials()?;

    // Display the API key (truncated for display)
    let api_key_display = if response.api_key.key.len() > 16 {
        format!("{}...", &response.api_key.key[..16])
    } else {
        response.api_key.key.clone()
    };

    output.success(&format!(
        "Signed up as {}. API key: {}",
        response.user.username, api_key_display
    ));

    Ok(())
}

/// Log in to FlagLite
pub async fn login(config: &mut Config, output: &Output) -> Result<()> {
    if config.is_authenticated() && !output.is_json() {
        output.warn("You are already logged in. This will replace your current session.");
    }

    if output.is_json() {
        return Err(anyhow::anyhow!(
            "Interactive login not supported with --format=json. Use environment variables."
        ));
    }

    // Get username
    let username: String = Input::new()
        .with_prompt("Username")
        .interact_text()?;

    // Get password
    let password: String = Password::new()
        .with_prompt("Password")
        .interact()?;

    // Authenticate
    let client = FlagLiteClient::new(&config.api_url);
    let response = client.login(&username, &password).await?;

    // Save credentials
    config.token = Some(response.token);
    config.username = Some(response.user.username.clone());
    // Note: login doesn't return a new API key, but we keep any existing one
    config.save_credentials()?;

    output.success(&format!("Logged in as {}", response.user.username));

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
