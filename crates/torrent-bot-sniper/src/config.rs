use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct TolokaCredentials {
    #[serde(rename = "toloka_username")]
    pub(crate) username: String,
    #[serde(rename = "toloka_password")]
    pub(crate) password: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) storage_file: String,
    pub(crate) server_endpoint: String,
    pub(crate) openai_api_key: String,
    #[serde(flatten)]
    pub(crate) toloka: TolokaCredentials,
}
