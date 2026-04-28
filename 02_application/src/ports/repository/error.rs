#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Not found")]
    NotFound,
    #[error("Already exists")]
    AlreadyExists,
    #[error("Persistence Error: {0}")]
    PersistenceError(String),
}

