use crate::ports::repository::error::RepositoryError;
use async_trait::async_trait;
use domain::chat::Chat;
use domain::peer::PeerId;

#[async_trait]
pub trait ChatRepository {
    async fn get(&self, peer: &PeerId) -> Result<Option<Chat>, RepositoryError>;

    async fn save(&self, chat: Chat) -> Result<(), RepositoryError>;

    async fn list(&self) -> Result<Vec<Chat>, RepositoryError>;
}
