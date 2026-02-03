//! Authentication commands

use crate::config::Config;
use crate::output::Output;
use anyhow::Result;
use dialoguer::{Input, Password};
use flaglite_shared::FlagLiteClient;

/// Log in to FlagLite
pub async fn login(config: &mut Config, output: &Output) -> Result<()> {
    if config.is_authenticated() && !output.is_json() {
        output.warn("You are already logged in. This will replace your current session.");
    }

    // Get credentials interactively
    let email: String = if output.is_json() {
        return Err(anyhow::anyhow!(
            "Interactive login not supported with --format=json. Use environment variables."
        ));
    } else {
        Input::new()
            .with_prompt("Email")
            .interact_text()?
    };

    let password: String = Password::new()
        .with_prompt("Password")
        .interact()?;

    // Authenticate
    let client = FlagLiteClient::new(&config.api_url);
    let response = client.login(&email, &password).await?;

    // Save token
    config.token = Some(response.token);
    config.save()?;

    output.success(&format!("Logged in as {}", response.user.email));

    Ok(())
}

/// Log out of FlagLite
pub async fn logout(config: &mut Config, output: &Output) -> Result<()> {
    if !config.is_authenticated() {
        output.info("You are not logged in.");
        return Ok(());
    }

    config.clear_auth();
    config.save()?;

    output.success("Logged out successfully.");

    Ok(())
}

/// Show current user
pub async fn whoami(config: &Config, output: &Output) -> Result<()> {
    let token = config.require_token()?;

    let client = FlagLiteClient::new(&config.api_url).with_token(token);
    let user = client.whoami().await?;

    output.print_user(&user)?;

    Ok(())
}
