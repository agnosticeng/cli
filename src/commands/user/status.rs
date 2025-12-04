use std::error::Error;

use crate::{
    commands::{UserAction, user::user::User},
    utils::{AppConfig, ensure_valid_tokens},
};
use reqwest::Client;

impl UserAction {
    pub(super) async fn handle_status(self, config: &AppConfig) -> Result<(), Box<dyn Error>> {
        let client = Client::new();
        let auth_tokens = match ensure_valid_tokens(config, &client).await {
            Ok(tokens) => tokens,
            Err(e) => {
                if config.verbose {
                    eprintln!("{}", e)
                }
                println!("Authentication required. Please run `user login` first.");
                return Ok(());
            }
        };

        let response = client
            .get("https://app.agnostic.tech/api/user")
            .bearer_auth(auth_tokens.id_token())
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            println!("Authentication failed. Please try to log in again.");
            return Ok(());
        }

        let user: User = response.json().await?;

        println!("User Status");
        println!("=============");
        println!();
        println!("User logged in as:");
        println!("  id: {}", user.id());
        println!("  email: {}", user.email());
        println!("  username: {}", user.username());

        Ok(())
    }
}
