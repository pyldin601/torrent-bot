use serde::Deserialize;

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
    #[serde(default, rename = "telegram_bot_chat_id")]
    pub(crate) bot_chat_id: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct TransmissionConfig {
    #[serde(rename = "trans_url")]
    pub(crate) url: String,
    #[serde(rename = "trans_download_directory")]
    pub(crate) username: Option<String>,
    #[serde(default, rename = "trans_password")]
    pub(crate) password: Option<String>,
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
    pub(crate) transmission: TransmissionConfig,
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
