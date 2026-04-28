use crate::error::ApplicationError;
use crate::events::AppEvent;
use crate::ports::event_sender::EventSender;
use crate::ports::repository::peer::PeerRepository;
use domain::peer::{Peer, PeerAddress};
use std::sync::Arc;

pub struct AddPeer<P: PeerRepository, E: EventSender> {
    peer_repository: Arc<P>,
    event_sender: Arc<E>,
}

impl<P, E> AddPeer<P, E>
where
    P: PeerRepository,
    E: EventSender,
{
    pub fn new(peer_repository: Arc<P>, event_sender: Arc<E>) -> Self {
        Self {
            peer_repository,
            event_sender,
        }
    }

    pub async fn execute(&self, address: PeerAddress) -> Result<(), ApplicationError> {
        if let Ok(Some(_)) = self.peer_repository.get(&address).await {
            return Err(ApplicationError::PeerAlreadyExists);
        }

        let new_peer = Peer::new(address.clone());
        self.peer_repository.add(new_peer).await?;

        self.event_sender.send(AppEvent::PeerAdded(address)).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::test_helper::{MockEventSender, MockPeerRepository};

    #[tokio::test]
    async fn add_peer_success() {
        let mock_peer_repository = Arc::new(MockPeerRepository::new());
        let mock_event_sender = Arc::new(MockEventSender::new());
        let add_peer = AddPeer::new(mock_peer_repository.clone(), mock_event_sender.clone());
        let peer_address = PeerAddress::new("peer1".into());
        add_peer.execute(peer_address.clone()).await.unwrap();

        let retrieved_peer = mock_peer_repository
            .get(&peer_address)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved_peer.address(), peer_address);

        let events = mock_event_sender
            .get()
            .await
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(events, vec![AppEvent::PeerAdded(peer_address)]);
    }

    #[tokio::test]
    async fn add_peer_already_exists() {
        let mock_peer_repository = Arc::new(MockPeerRepository::new());
        let mock_event_sender = Arc::new(MockEventSender::new());
        let add_peer = AddPeer::new(mock_peer_repository.clone(), mock_event_sender.clone());
        let peer_address = PeerAddress::new("peer1".into());
        let result1 = add_peer.execute(peer_address.clone()).await;
        assert!(result1.is_ok());
        assert_eq!(mock_event_sender.get().await.len(), 1);
        let result2 = add_peer.execute(peer_address.clone()).await;
        assert!(result2.is_err());
        assert_eq!(mock_event_sender.get().await.len(), 1);
    }
}
