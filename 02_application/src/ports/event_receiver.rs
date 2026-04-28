use crate::events::AppEvent;
use tokio::sync::broadcast;

pub trait EventReceiverFactory: Send + Sync {
    fn subscribe(&self) -> broadcast::Receiver<AppEvent>;
}
