use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct TolokaCredentials {
    #[serde(rename = "toloka_username")]
    pub username: String,
    #[serde(rename = "toloka_password")]
    pub password: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub storage_file: String,
    #[serde(flatten)]
    pub toloka: TolokaCredentials,
}

impl Config {
    pub fn from_env() -> Self {
        match envy::from_env::<Self>() {
            Ok(config) => config,
            Err(error) => panic!("Missing environment variable: {:#?}", error),
        }
    }
}
