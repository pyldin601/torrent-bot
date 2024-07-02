use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct TolokaCredentials {
    #[serde(rename = "toloka_username")]
    pub username: String,
    #[serde(rename = "toloka_password")]
    pub password: String,
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
    #[serde(default, rename = "trans_dry_run")]
    pub dry_run: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub storage_file: String,
    #[serde(flatten)]
    pub toloka: TolokaCredentials,
    #[serde(flatten)]
    pub transmission: TransmissionConfig,
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
