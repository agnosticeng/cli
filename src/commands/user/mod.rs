mod login;
mod logout;
mod status;
mod user;

use clap::Subcommand;

use crate::utils::AppConfig;

#[derive(Subcommand, Debug)]
pub enum UserAction {
    Login,
    Logout,
    Status,
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
}
