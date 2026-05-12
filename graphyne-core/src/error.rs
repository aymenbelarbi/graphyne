use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphyneError {
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Sled database error: {0}")]
    Sled(#[from] sled::Error),
    
    #[error("Lexical index error: {0}")]
    Lexical(String),
    
    #[error("Vector index error: {0}")]
    Vector(String),
    
    #[error("Graph error: {0}")]
    Graph(String),
}

pub type Result<T> = std::result::Result<T, GraphyneError>;
