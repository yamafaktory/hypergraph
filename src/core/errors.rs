use thiserror::Error;

/// Enumeration of all the possible errors.
#[derive(Debug, Error)]
pub enum HypergraphError {
    /// Error when the path can't be created.
    #[error("Path can't be created")]
    PathCreation,
    /// Error when the path is not accessible.
    /// This could be the case when the listing permission is denied on one of
    /// the parent directories.
    #[error("Path is not accessible")]
    PathNotAccessible(#[source] std::io::Error, std::path::PathBuf),
    /// File error.
    #[error("File error")]
    File(#[source] std::io::Error),
    #[error("File error")]
    FileWithoutSource(),
    /// Serialization error.
    #[error("Serialization failed")]
    Serialization,
    /// Deserialization error.
    #[error("Deserialization failed")]
    Deserialization,
    /// Error when an entity is not found.
    #[error("Entity not found")]
    EntityNotFound,
    /// Error when an entity is created.
    #[error("Entity couldn't be created")]
    EntityCreation,
    /// Error when updating an entity.
    #[error("Entity couldn't be updated")]
    EntityUpdate,
    /// Processing error.
    #[error("Processing failed")]
    Processing,
}
