use std::sync::Arc;

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use open::that;
use tokio::{net::TcpListener, sync::watch};

use crate::{commands::UserAction, utils::AppConfig, utils::AuthTokens};

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
    pub(super) async fn handle_login(
        self,
        config: &AppConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
            if config.verbose {
                eprintln!("Failed to open browser: {}", e);
            }
            eprintln!("Please manually open: {}", login_url);
        }

        // Build router with shutdown sender
        let app = Router::new()
            .route("/", post(handle_callback))
            .layer(tower_http::cors::CorsLayer::permissive())
            .with_state(state);

        if config.verbose {
            println!("HTTP server listening at {}", redirect_uri);
        }

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

        if config.verbose {
            println!("Shutting down HTTP server.");
        }

        Ok(())
    }
}

async fn handle_callback(
    State(state): State<Arc<LoginAppState>>,
    Json(payload): Json<AuthTokens>,
) -> impl IntoResponse {
    if !payload.is_valid_token_type() {
        eprintln!("Invalid token_type: {}", payload.token_type());
        return StatusCode::BAD_REQUEST;
    }

    let auth_file = state.config.agnostic_dir.join("user/auth.json");
    if payload.save(&auth_file).is_ok() {
        if state.config.verbose {
            println!("Tokens saved to {:?}", auth_file);
        }
        let _ = state.shutdown_tx.send(ShutdownSignal::Triggered);
        return StatusCode::NO_CONTENT;
    }

    StatusCode::INTERNAL_SERVER_ERROR
}
