use application::ports::repository::chat::ChatRepository;
use application::ports::repository::error::RepositoryError;
use async_trait::async_trait;
use domain::chat::Chat;
use domain::message::ChatMessage;
use domain::peer::{PeerAddress, PeerId};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::DirEntry;
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
            if !messages_file.exists() {
                continue;
            }

            let (messages, peer_address) = Self::messages_from_file(entry, &messages_file).await?;
            cache.insert(peer_address, messages);
        }

        Ok(())
    }

    async fn messages_from_file(
        entry: DirEntry,
        messages_file: &PathBuf,
    ) -> Result<(Vec<ChatMessage>, String), RepositoryError> {
        let contents = tokio::fs::read_to_string(&messages_file)
            .await
            .map_err(|e| RepositoryError::PersistenceError(e.to_string()))?;

        let dto = serde_json::from_str::<MessageVecDto>(&contents)
            .map_err(|e| RepositoryError::PersistenceError(e.to_string()))?;

        let messages: Vec<ChatMessage> = dto
            .messages
            .into_iter()
            .filter_map(|m| ChatMessage::try_from(m).ok())
            .collect();

        let peer_id = entry.file_name().to_string_lossy().to_string();
        Ok((messages, peer_id))
    }

    async fn write_to_disk(
        &self,
        peer_address: &String,
        chat: &Chat,
    ) -> Result<(), RepositoryError> {
        let dir = self.data_dir.join("peers").join(peer_address);
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|e| RepositoryError::PersistenceError(e.to_string()))?;

        let dto = MessageVecDto::from(chat.messages.as_slice());
        let json = serde_json::to_string_pretty(&dto)
            .map_err(|e| RepositoryError::PersistenceError(e.to_string()))?;

        tokio::fs::write(self.messages_file(peer_address), json)
            .await
            .map_err(|e| RepositoryError::PersistenceError(e.to_string()))?;
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
        let peer_address = chat.peer.to_string();
        self.write_to_disk(&peer_address, &chat).await?;
        self.cache.write().await.insert(peer_address, chat.messages);
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
