use application::events::AppEvent;
use application::ports::event_receiver::EventReceiverFactory;
use application::ports::event_sender::EventSender;
use async_trait::async_trait;
use tokio::sync::broadcast;

pub struct BroadcastEventBus {
    sender: broadcast::Sender<AppEvent>,
}

impl BroadcastEventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }
}

#[async_trait]
impl EventSender for BroadcastEventBus {
    async fn send(&self, event: AppEvent) {
        let _ = self.sender.send(event);
    }
}

impl EventReceiverFactory for BroadcastEventBus {
    fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }
}
