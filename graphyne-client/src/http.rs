//! HTTP/JSON client implementation for Graphyne services

use crate::{ClientConfig, ClientError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP client for Graphyne REST API
pub struct GraphyneHttpClient {
    client: reqwest::Client,
    base_url: String,
}

impl GraphyneHttpClient {
    /// Create a new HTTP client
    pub fn new(config: ClientConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");
        
        // Extract base URL from endpoint (remove grpc port, use 8080 for HTTP)
        let base_url = if config.endpoint.contains("50051") {
            config.endpoint.replace("50051", "8080")
        } else {
            config.endpoint
        };
        
        Self { client, base_url }
    }
    
    /// Search using HTTP API
    pub async fn search(
        &self,
        collection: &str,
        bucket: &str,
        query: &str,
        limit: i32,
        mode: &str,
    ) -> Result<SearchResponse> {
        let url = format!("{}/v1/search", self.base_url);
        
        let request = SearchRequest {
            collection: collection.to_string(),
            bucket: bucket.to_string(),
            query: query.to_string(),
            limit,
            mode: mode.to_string(),
        };
        
        let response = self.client
            .get(&url)
            .query(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(ClientError::InvalidResponse(
                format!("HTTP {}", response.status())
            ))
        }
    }
    
    /// Push a document via HTTP API
    pub async fn push_document(
        &self,
        collection: &str,
        bucket: &str,
        id: &str,
        content: &str,
        metadata: HashMap<String, String>,
    ) -> Result<DocumentResponse> {
        let url = format!("{}/v1/documents", self.base_url);
        
        let request = PushDocumentRequest {
            collection: collection.to_string(),
            bucket: bucket.to_string(),
            id: id.to_string(),
            content: content.to_string(),
            metadata,
        };
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(ClientError::InvalidResponse(
                format!("HTTP {}", response.status())
            ))
        }
    }
    
    /// Add embedding vector via HTTP API
    pub async fn add_embedding(
        &self,
        id: &str,
        values: Vec<f32>,
        metadata: HashMap<String, String>,
    ) -> Result<VectorResponse> {
        let url = format!("{}/v1/vectors", self.base_url);
        
        let request = AddEmbeddingRequest {
            id: id.to_string(),
            values,
            metadata,
        };
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(ClientError::InvalidResponse(
                format!("HTTP {}", response.status())
            ))
        }
    }
    
    /// Add a node to the graph via HTTP API
    pub async fn add_node(
        &self,
        id: &str,
        node_type: &str,
        properties: HashMap<String, String>,
    ) -> Result<GraphResponse> {
        let url = format!("{}/v1/graph/nodes", self.base_url);
        
        let request = AddNodeRequest {
            id: id.to_string(),
            node_type: node_type.to_string(),
            properties,
        };
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(ClientError::InvalidResponse(
                format!("HTTP {}", response.status())
            ))
        }
    }
}

// Request/Response types for HTTP API

#[derive(Serialize, Deserialize)]
pub struct SearchRequest {
    pub collection: String,
    pub bucket: String,
    pub query: String,
    pub limit: i32,
    pub mode: String,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub data: String,
}

#[derive(Serialize, Deserialize)]
pub struct PushDocumentRequest {
    pub collection: String,
    pub bucket: String,
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct DocumentResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddEmbeddingRequest {
    pub id: String,
    pub values: Vec<f32>,
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct VectorResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddNodeRequest {
    pub id: String,
    pub node_type: String,
    pub properties: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct GraphResponse {
    pub success: bool,
    pub message: String,
}
