#[async_trait::async_trait]
pub trait BotCommandHandler<'a>: Send + Sync + 'a {
    async fn handle_search_command(&self, query: &str) {}

    async fn handle_add_command(&self, topic_id: &str) {}
}
