use application::ports::repository::chat::ChatRepository;
use application::ports::repository::error::RepositoryError;
use async_trait::async_trait;
use domain::chat::Chat;
use domain::message::ChatMessage;
use domain::peer::{PeerAddress, PeerId};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::dtos::MessageVecDto;

pub struct FileChatRepository {
    data_dir: PathBuf,
    cache: RwLock<HashMap<String, Vec<ChatMessage>>>,
}

impl FileChatRepository {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            cache: RwLock::new(HashMap::new()),
        }
    }

    fn messages_file(&self, peer_address: &str) -> PathBuf {
        self.data_dir
            .join("peers")
            .join(peer_address)
            .join("messages.json")
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
            let messages_file = entry.path().join("messages.json");
            if messages_file.exists() {
                if let Ok(contents) = tokio::fs::read_to_string(&messages_file).await {
                    if let Ok(dto) = serde_json::from_str::<MessageVecDto>(&contents) {
                        let messages: Vec<ChatMessage> = dto
                            .messages
                            .into_iter()
                            .filter_map(|m| ChatMessage::try_from(m).ok())
                            .collect();
                        let dir_name = entry
                            .file_name()
                            .to_string_lossy()
                            .to_string();
                        cache.insert(dir_name, messages);
                    }
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl ChatRepository for FileChatRepository {
    async fn get(&self, peer: &PeerId) -> Result<Option<Chat>, RepositoryError> {
        let key = peer.to_string();
        let cache = self.cache.read().await;
        match cache.get(&key) {
            Some(messages) => Ok(Some(Chat::new(peer.clone(), messages.clone()))),
            None => Ok(None),
        }
    }

    async fn save(&self, chat: Chat) -> Result<(), RepositoryError> {
        let key = chat.peer.to_string();

        // Write to disk
        let dir = self.data_dir.join("peers").join(&key);
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|e| RepositoryError::PersistenceError(e.to_string()))?;

        let dto = MessageVecDto::from(chat.messages.as_slice());
        let json = serde_json::to_string_pretty(&dto)
            .map_err(|e| RepositoryError::PersistenceError(e.to_string()))?;

        tokio::fs::write(self.messages_file(&key), json)
            .await
            .map_err(|e| RepositoryError::PersistenceError(e.to_string()))?;

        // Update cache
        self.cache.write().await.insert(key, chat.messages);

        Ok(())
    }

    async fn list(&self) -> Result<Vec<Chat>, RepositoryError> {
        let cache = self.cache.read().await;
        let chats = cache
            .iter()
            .map(|(peer_str, messages)| {
                let peer = PeerAddress::new(Arc::from(peer_str.as_str()));
                Chat::new(peer, messages.clone())
            })
            .collect();
        Ok(chats)
    }
}
