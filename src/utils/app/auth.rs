use std::{
    collections::HashMap,
    error::Error,
    fs,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::dangerous::insecure_decode;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::utils::AppConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthTokens {
    access_token: String,
    id_token: String,
    token_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
}

impl AuthTokens {
    pub fn load_from_config(config: &AppConfig) -> Result<Option<Self>, Box<dyn Error>> {
        let auth_json = config.agnostic_dir.join("user/auth.json");
        if !auth_json.try_exists()? {
            return Ok(None);
        }

        let tokens = AuthTokens::load(auth_json)?;

        Ok(Some(tokens))
    }

    fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let tokens = serde_json::from_str(&content)?;
        Ok(tokens)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(path, &json)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn id_token(&self) -> &str {
        &self.id_token
    }

    pub fn expires_at(&self) -> Result<SystemTime, AuthTokenError> {
        let token_data = insecure_decode::<IdTokenClaims>(&self.id_token)
            .map_err(AuthTokenError::DecodeFailed)?;
        let claims = token_data.claims;
        let exp = claims.exp.ok_or(AuthTokenError::AlreadyExpired)?;
        let expiration = UNIX_EPOCH + Duration::from_secs(exp);
        Ok(expiration)
    }

    pub fn token_type(&self) -> &str {
        &self.token_type
    }

    pub fn is_valid_token_type(&self) -> bool {
        self.token_type.to_lowercase() == "bearer"
    }

    pub fn needs_refresh(&self, threshold: Duration) -> Result<bool, AuthTokenError> {
        let expires_at = self.expires_at()?;
        let now = SystemTime::now();
        Ok(now + threshold >= expires_at)
    }

    pub async fn refresh(&mut self, client: &Client) -> Result<(), AuthTokenError> {
        let refresh_token = self
            .refresh_token
            .as_ref()
            .ok_or(AuthTokenError::NoRefreshToken)?;

        let mut body = HashMap::new();
        body.insert("refresh_token", refresh_token);

        let response = client
            .post("https://app.agnostic.tech/api/refresh_token")
            .json(&body)
            .send()
            .await?;

        let new_tokens: AuthTokens = response
            .json()
            .await
            .map_err(|e| AuthTokenError::InvalidResponse(e.to_string()))?;

        *self = new_tokens;

        Ok(())
    }
}

/// check if needs refresh soon (5 min)
pub async fn ensure_valid_tokens(
    config: &AppConfig,
    client: &Client,
) -> Result<AuthTokens, AuthTokenError> {
    let result = AuthTokens::load_from_config(config).map_err(move |e| {
        if config.verbose {
            eprintln!("{}", e);
        }
        AuthTokenError::NoAuthTokens
    })?;

    let mut tokens = result.ok_or(AuthTokenError::NoAuthTokens)?;

    if tokens.needs_refresh(Duration::from_secs(5 * 60))? {
        tokens.refresh(client).await?;
        tokens
            .save(config.agnostic_dir.join("user/auth.json"))
            .map_err(|e| AuthTokenError::InvalidResponse(e.to_string()))?;
    }

    Ok(tokens)
}

#[derive(Debug, Serialize, Deserialize)]
struct IdTokenClaims {
    exp: Option<u64>,
    iat: Option<u64>,
    sub: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthTokenError {
    #[error("Missing auth tokens")]
    NoAuthTokens,
    #[error("Missing refresh token")]
    NoRefreshToken,
    #[error("JWT decode failed: {0}")]
    DecodeFailed(#[from] jsonwebtoken::errors::Error),
    #[error("Token already expired")]
    AlreadyExpired,
    #[error("Refresh request failed: {0}")]
    HttpFailed(#[from] reqwest::Error),
    #[error("Invalid refresh response: {0}")]
    InvalidResponse(String),
}
