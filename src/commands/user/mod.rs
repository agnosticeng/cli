mod user;

use std::{error::Error, fs, sync::Arc};

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use clap::Subcommand;
use open::that;
use reqwest::Client;
use tokio::{net::TcpListener, sync::watch};

use crate::{
    commands::user::user::User,
    utils::{AppConfig, AuthTokens, ensure_valid_tokens},
};

#[derive(Subcommand, Debug)]
pub enum UserAction {
    Login,
    Logout,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShutdownSignal {
    NotTriggered,
    Triggered,
}

struct LoginAppState {
    config: AppConfig,
    shutdown_tx: watch::Sender<ShutdownSignal>,
}

impl UserAction {
    pub async fn handle(self, config: &AppConfig) {
        match self {
            Self::Login => self
                .handle_login(config)
                .await
                .expect("Unable to handle login command"),
            Self::Logout => self
                .handle_logout(config)
                .await
                .expect("Unable to handle logout command"),
            Self::Status => self
                .handle_status(config)
                .await
                .expect("Unable to handle status command"),
        }
    }

    async fn handle_login(self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        let (shutdown_tx, mut shutdown_rx) = watch::channel(ShutdownSignal::NotTriggered);

        let state = Arc::new(LoginAppState {
            shutdown_tx,
            config: config.clone(),
        });

        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let local_addr = listener.local_addr()?;
        let port = local_addr.port();

        let redirect_uri = format!("http://localhost:{}", port);
        let login_url = format!(
            "https://app.agnostic.tech/login?redirectTo={}",
            urlencoding::encode(&redirect_uri)
        );

        println!("Opening browser: {}", login_url);
        if let Err(e) = that(&login_url) {
            eprintln!("Failed to open browser: {}", e);
            eprintln!("Please manually open: {}", login_url);
        }

        // Build router with shutdown sender
        let app = Router::new()
            .route("/", post(handle_callback))
            .layer(tower_http::cors::CorsLayer::permissive())
            .with_state(state);

        tokio::select! {
            result = axum::serve(listener, app) => {
                if let Err(e) = result {
                    eprintln!("Server error: {}", e);
                }
            }
            _ = shutdown_rx.wait_for(|&signal| signal == ShutdownSignal::Triggered) => {
                println!("Authentication successful!");
            }
        }

        Ok(())
    }

    async fn handle_logout(self, config: &AppConfig) -> Result<(), Box<dyn Error>> {
        let auth_json = config.agnostic_dir.join("user/auth.json");
        if auth_json.try_exists()? {
            fs::remove_file(auth_json)?;
            println!("auth.json file removed");
        }

        println!("User logged out...");

        Ok(())
    }

    async fn handle_status(self, config: &AppConfig) -> Result<(), Box<dyn Error>> {
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

async fn handle_callback(
    State(state): State<Arc<LoginAppState>>,
    Json(payload): Json<AuthTokens>,
) -> impl IntoResponse {
    if payload.is_valid_token_type() {
        eprintln!("Invalid token_type: {}", payload.token_type());
        return StatusCode::BAD_REQUEST;
    }

    let auth_file = state.config.agnostic_dir.join("user/auth.json");
    if payload.save(&auth_file).is_ok() {
        println!("Tokens saved to auth.json");
        let _ = state.shutdown_tx.send(ShutdownSignal::Triggered);
        return StatusCode::NO_CONTENT;
    }

    StatusCode::INTERNAL_SERVER_ERROR
}
