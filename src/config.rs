use serde::{de, Deserialize};

fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;

    match s.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(de::Error::unknown_variant(&s, &["true", "false"])),
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TolokaCredentials {
    #[serde(rename = "toloka_username")]
    pub username: String,
    #[serde(rename = "toloka_password")]
    pub password: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TelegramCredentials {
    #[serde(default, rename = "telegram_bot_token")]
    pub bot_token: Option<String>,
    #[serde(default, rename = "telegram_bot_chat_id")]
    pub bot_chat_id: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TransmissionConfig {
    #[serde(rename = "trans_url")]
    pub url: String,
    #[serde(rename = "trans_download_directory")]
    pub download_directory: String,
    #[serde(default, rename = "trans_username")]
    pub username: Option<String>,
    #[serde(default, rename = "trans_password")]
    pub password: Option<String>,
    #[serde(
        default,
        rename = "trans_dry_run",
        deserialize_with = "deserialize_bool"
    )]
    pub dry_run: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub storage_file: String,
    #[serde(default)]
    pub wipeout_mode: bool,
    #[serde(flatten)]
    pub toloka: TolokaCredentials,
    #[serde(flatten)]
    pub transmission: TransmissionConfig,
    #[serde(flatten)]
    pub telegram: TelegramCredentials,
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
