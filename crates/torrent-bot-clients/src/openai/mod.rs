pub struct OpenAiClient {
    api_key: &'static str,
}

impl OpenAiClient {
    fn create(api_key: &'static str) -> Self {
        Self { api_key }
    }
}
