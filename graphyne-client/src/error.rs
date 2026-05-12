use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("gRPC transport error: {0}")]
    GrpcTransport(#[from] tonic::transport::Error),
    
    #[error("gRPC status error: {0}")]
    GrpcStatus(#[from] tonic::Status),
    
    #[error("HTTP request error: {0}")]
    HttpRequest(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("URI parse error: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),
    
    #[error("Timeout error")]
    Timeout,
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, ClientError>;
