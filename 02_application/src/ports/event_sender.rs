use crate::events::AppEvent;
use async_trait::async_trait;

#[async_trait]
pub trait EventSender: Send + Sync {
    async fn send(&self, event: AppEvent);
}
