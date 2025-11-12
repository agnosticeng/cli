use std::error::Error;

use crate::{
    commands::{UserAction, user::user::User},
    utils::{AppConfig, AuthTokens, ensure_valid_tokens},
};
use reqwest::Client;

impl UserAction {
    pub(super) async fn handle_status(self, config: &AppConfig) -> Result<(), Box<dyn Error>> {
        println!("User Status");
        println!("=============");
        println!();

        let Some(mut auth_tokens) = AuthTokens::load_from_config(config)? else {
            println!("User not logged in");
            return Ok(());
        };

        let client = Client::new();
        ensure_valid_tokens(&config, &mut auth_tokens, &client).await?;

        let response = client
            .get("https://app.agnostic.tech/api/user")
            .bearer_auth(&auth_tokens.id_token())
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            println!("User not logged in");
            return Ok(());
        }

        let user: User = response.json().await?;

        println!("User logged in as:");
        println!("  id: {}", user.id());
        println!("  email: {}", user.email());
        println!("  username: {}", user.username());

        Ok(())
    }
}
