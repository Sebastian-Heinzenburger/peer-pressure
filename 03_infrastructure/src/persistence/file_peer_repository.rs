use application::ports::repository::error::RepositoryError;
use application::ports::repository::peer::PeerRepository;
use async_trait::async_trait;
use domain::peer::{Peer, PeerId};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

use super::dtos::PeerDto;

pub struct FilePeerRepository {
    data_dir: PathBuf,
    cache: RwLock<HashMap<String, Peer>>,
}

impl FilePeerRepository {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            cache: RwLock::new(HashMap::new()),
        }
    }

    fn peer_dir(&self, address: &str) -> PathBuf {
        self.data_dir.join("peers").join(address)
    }

    fn peer_file(&self, address: &str) -> PathBuf {
        self.peer_dir(address).join("peer.json")
    }

    pub async fn load(&self) -> Result<(), RepositoryError> {
        let peers_dir = self.data_dir.join("peers");
        if !peers_dir.exists() {
            return Ok(());
        }

        let mut entries = tokio::fs::read_dir(&peers_dir)
            .await
            .map_err(|e| RepositoryError::PersistenceError(e.to_string()))?;

        let mut cache = self.cache.write().await;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| RepositoryError::PersistenceError(e.to_string()))?
        {
            let peer_file = entry.path().join("peer.json");

            if !peer_file.exists() {
                continue;
            }

            let peer = Self::peer_from_file(&peer_file).await?;
            cache.insert(peer.address().to_string(), peer);
        }

        Ok(())
    }

    async fn peer_from_file(peer_file: &PathBuf) -> Result<Peer, RepositoryError> {
        let contents = tokio::fs::read_to_string(&peer_file)
            .await
            .map_err(|e| RepositoryError::PersistenceError(e.to_string()))?;

        let dto = serde_json::from_str::<PeerDto>(&contents)
            .map_err(|e| RepositoryError::PersistenceError(e.to_string()))?;

        let peer = Peer::from(dto);
        Ok(peer)
    }

    async fn write_to_disk(&self, address_str: &str, peer: &Peer) -> Result<(), RepositoryError> {
        let dir = self.peer_dir(address_str);
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|e| RepositoryError::PersistenceError(e.to_string()))?;

        let dto = PeerDto::from(peer.clone());
        let json = serde_json::to_string_pretty(&dto)
            .map_err(|e| RepositoryError::PersistenceError(e.to_string()))?;

        tokio::fs::write(self.peer_file(address_str), json)
            .await
            .map_err(|e| RepositoryError::PersistenceError(e.to_string()))?;
        Ok(())
    }

    async fn remove_from_disk(&self, address_str: &str) -> Result<(), RepositoryError> {
        let dir = self.peer_dir(address_str);
        if dir.exists() {
            tokio::fs::remove_dir_all(&dir)
                .await
                .map_err(|e| RepositoryError::PersistenceError(e.to_string()))?;
        }
        Ok(())
    }
}

#[async_trait]
impl PeerRepository for FilePeerRepository {
    async fn add(&self, peer: Peer) -> Result<(), RepositoryError> {
        let address_str = peer.address().to_string();
        self.write_to_disk(&address_str, &peer).await?;
        self.cache.write().await.insert(address_str, peer);
        Ok(())
    }

    async fn get(&self, id: &PeerId) -> Result<Option<Peer>, RepositoryError> {
        let cache = self.cache.read().await;
        Ok(cache.get(&id.to_string()).cloned())
    }

    async fn list(&self) -> Result<Vec<Peer>, RepositoryError> {
        let cache = self.cache.read().await;
        Ok(cache.values().cloned().collect())
    }

    async fn remove(&self, id: &PeerId) -> Result<(), RepositoryError> {
        let address_str = id.to_string();

        self.remove_from_disk(&address_str).await?;

        // Remove from cache
        self.cache.write().await.remove(&address_str);

        Ok(())
    }
}
