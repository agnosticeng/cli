use std::{error::Error, fmt::Display, fs, path::PathBuf};

use clap::Subcommand;
use inquire::Select;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::utils::{AppConfig, AuthTokens, ensure_valid_tokens};

#[derive(Subcommand, Debug)]
pub enum TeamAction {
    List,
    Select,
}

/// Agnostic Team entity
#[derive(Serialize, Deserialize)]
pub struct Team {
    id: u8,
    name: String,
    slug: String,
    #[serde(rename = "type")]
    variant: String,
}

impl Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// List teams JSON Response body
#[derive(Serialize, Deserialize)]
struct ListTeamsResponse {
    teams: Vec<Team>,
}

impl TeamAction {
    pub async fn handle(self, config: &AppConfig) {
        let client = Client::new();
        let auth_tokens = match ensure_valid_tokens(config, &client).await {
            Ok(tokens) => tokens,
            Err(e) => {
                if config.verbose {
                    eprintln!("{}", e)
                }
                println!("Authentication required. Please run `user login` first.");
                return;
            }
        };

        match self {
            Self::List => self
                .handle_list(&client, &auth_tokens, config)
                .await
                .expect("Unable to handle list command"),
            Self::Select => self
                .handle_select(&client, &auth_tokens, config)
                .await
                .expect("Unable to handle select command"),
        }
    }

    async fn handle_list(
        self,
        client: &Client,
        tokens: &AuthTokens,
        config: &AppConfig,
    ) -> Result<(), Box<dyn Error>> {
        let response = client
            .get("https://app.agnostic.tech/api/teams")
            .bearer_auth(tokens.id_token())
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            println!("Authentication failed. Please try to log in again.");
            return Ok(());
        }

        let body: ListTeamsResponse = response.json().await?;

        println!("Teams");
        println!("=============");
        println!();

        let current = get_current_team(config);
        for team in body.teams {
            if current.as_ref().map(|p| p.id) == Some(team.id) {
                println!("> {} (current)", team);
            } else {
                println!("> {}", team);
            }
        }
        println!();

        Ok(())
    }

    async fn handle_select(
        self,
        client: &Client,
        tokens: &AuthTokens,
        config: &AppConfig,
    ) -> Result<(), Box<dyn Error>> {
        let response = client
            .get("https://app.agnostic.tech/api/teams")
            .bearer_auth(tokens.id_token())
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            println!("Authentication failed. Please try to log in again.");
            return Ok(());
        }

        let previous = get_current_team(config);

        let body: ListTeamsResponse = response.json().await?;

        let options: Vec<String> = body
            .teams
            .iter()
            .map(|t| {
                if previous.as_ref().map(|p| p.id) == Some(t.id) {
                    format!("{} (current)", t)
                } else {
                    t.name.clone()
                }
            })
            .collect();

        let selected_option = Select::new("Select a team:", options.clone())
            .with_help_message("Use ↑↓, type to filter")
            .prompt()?;

        let index = options.iter().position(|o| *o == selected_option).unwrap();
        let selected = body.teams.get(index).unwrap();

        println!();

        fs::write(
            get_team_json_path(config),
            &serde_json::to_string_pretty(selected)?,
        )?;

        println!("Choice saved.");

        Ok(())
    }
}

pub fn get_current_team(config: &AppConfig) -> Option<Team> {
    fs::read_to_string(get_team_json_path(config))
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
}

fn get_team_json_path(config: &AppConfig) -> PathBuf {
    config.agnostic_dir.join("user/team.json")
}
