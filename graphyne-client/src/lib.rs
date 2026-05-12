//! Graphyne Client SDK
//!
//! Provides both gRPC and HTTP clients for interacting with Graphyne services.

pub mod grpc;
pub mod http;
pub mod error;

pub use error::ClientError;
pub use error::Result;
pub use grpc::GraphyneGrpcClient;
pub use http::GraphyneHttpClient;

use std::time::Duration;

/// Common configuration for Graphyne clients
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub endpoint: String,
    pub timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:50051".to_string(),
            timeout: Duration::from_secs(30),
        }
    }
}
