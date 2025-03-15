use serde::Deserialize;

use crate::serde_helpers::deserialize_i64;

fn default_bind_address() -> String {
    "0.0.0.0:8080".to_string()
}

fn default_shutdown_timeout() -> u64 {
    30
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct TolokaCredentials {
    #[serde(rename = "toloka_username")]
    pub(crate) username: String,
    #[serde(rename = "toloka_password")]
    pub(crate) password: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct TelegramCredentials {
    #[serde(default, rename = "telegram_bot_token")]
    pub(crate) bot_token: String,
    #[serde(
        default,
        rename = "telegram_bot_chat_id",
        deserialize_with = "deserialize_i64"
    )]
    pub(crate) bot_chat_id: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Config {
    #[serde(default = "default_bind_address")]
    pub(crate) bind_address: String,
    #[serde(default = "default_shutdown_timeout")]
    pub(crate) shutdown_timeout: u64,
    #[serde(flatten)]
    pub(crate) toloka: TolokaCredentials,
    #[serde(flatten)]
    pub(crate) telegram: TelegramCredentials,
}

impl Config {
    pub fn init_dotenv() {
        dotenv::dotenv().ok();
    }

    pub fn from_env() -> Self {
        match envy::from_env::<Self>() {
            Ok(config) => config,
            Err(error) => panic!("Missing environment variable: {:#?}", error),
        }
    }
}
