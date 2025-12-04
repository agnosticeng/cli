use std::{error::Error, fs};

use crate::{commands::UserAction, utils::AppConfig};

impl UserAction {
    pub(super) async fn handle_logout(self, config: &AppConfig) -> Result<(), Box<dyn Error>> {
        let auth_json = config.agnostic_dir.join("user/auth.json");
        if auth_json.try_exists()? {
            fs::remove_file(auth_json)?;
            println!("auth.json file removed");
        }

        println!("User logged out...");

        Ok(())
    }
}
