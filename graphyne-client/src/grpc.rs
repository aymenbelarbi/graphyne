//! gRPC client implementation for Graphyne services

use crate::ClientConfig;
use graphyne_proto::graphyne::{
    search_service_client::SearchServiceClient,
    document_service_client::DocumentServiceClient,
    vector_service_client::VectorServiceClient,
    graph_service_client::GraphServiceClient,
    memory_service_client::MemoryServiceClient,
    SearchRequest, PushDocumentRequest, AddEmbeddingRequest, 
    AddNodeRequest, AddEdgeRequest, StoreMemoryRequest,
    SearchVectorRequest, TraverseGraphRequest, RecallMemoryRequest,
};
use crate::ClientError;
use crate::Result;

/// gRPC client for Graphyne services
pub struct GraphyneGrpcClient {
    search_client: SearchServiceClient<tonic::transport::Channel>,
    document_client: DocumentServiceClient<tonic::transport::Channel>,
    vector_client: VectorServiceClient<tonic::transport::Channel>,
    graph_client: GraphServiceClient<tonic::transport::Channel>,
    memory_client: MemoryServiceClient<tonic::transport::Channel>,
}

impl GraphyneGrpcClient {
    /// Create a new gRPC client connected to the specified endpoint
    pub async fn connect(config: ClientConfig) -> Result<Self> {
        let channel = tonic::transport::Channel::from_shared(config.endpoint)?
            .timeout(config.timeout)
            .connect()
            .await?;
        
        Ok(Self {
            search_client: SearchServiceClient::new(channel.clone()),
            document_client: DocumentServiceClient::new(channel.clone()),
            vector_client: VectorServiceClient::new(channel.clone()),
            graph_client: GraphServiceClient::new(channel.clone()),
            memory_client: MemoryServiceClient::new(channel),
        })
    }
    
    /// Search using the specified mode
    pub async fn search(
        &mut self,
        collection: &str,
        bucket: &str,
        query: &str,
        limit: i32,
        mode: &str,
    ) -> Result<Vec<graphyne_proto::graphyne::SearchResult>> {
        let request = tonic::Request::new(SearchRequest {
            collection: collection.to_string(),
            bucket: bucket.to_string(),
            query: query.to_string(),
            limit,
            mode: mode.to_string(),
        });
        
        let response = self.search_client.search(request).await?;
        Ok(response.into_inner().results)
    }
    
    /// Push a document to the index
    pub async fn push_document(
        &mut self,
        collection: &str,
        bucket: &str,
        id: &str,
        content: &str,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<bool> {
        let request = tonic::Request::new(PushDocumentRequest {
            collection: collection.to_string(),
            bucket: bucket.to_string(),
            id: id.to_string(),
            content: content.to_string(),
            metadata,
        });
        
        let response = self.document_client.push_document(request).await?;
        Ok(response.into_inner().success)
    }
    
    /// Add embedding vector
    pub async fn add_embedding(
        &mut self,
        id: &str,
        values: Vec<f32>,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<bool> {
        let request = tonic::Request::new(AddEmbeddingRequest {
            id: id.to_string(),
            values,
            metadata,
        });
        
        let response = self.vector_client.add_embedding(request).await?;
        Ok(response.into_inner().success)
    }
    
    /// Add a node to the graph
    pub async fn add_node(
        &mut self,
        id: &str,
        node_type: &str,
        properties: std::collections::HashMap<String, String>,
    ) -> Result<bool> {
        let request = tonic::Request::new(AddNodeRequest {
            id: id.to_string(),
            node_type: node_type.to_string(),
            properties,
        });
        
        let response = self.graph_client.add_node(request).await?;
        Ok(response.into_inner().success)
    }
    
    /// Add an edge to the graph
    pub async fn add_edge(
        &mut self,
        from_id: &str,
        to_id: &str,
        edge_type: &str,
        properties: std::collections::HashMap<String, String>,
    ) -> Result<bool> {
        let request = tonic::Request::new(AddEdgeRequest {
            from_id: from_id.to_string(),
            to_id: to_id.to_string(),
            edge_type: edge_type.to_string(),
            properties,
        });
        
        let response = self.graph_client.add_edge(request).await?;
        Ok(response.into_inner().success)
    }
    
    /// Store memory
    pub async fn store_memory(
        &mut self,
        key: &str,
        value: &str,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<bool> {
        let request = tonic::Request::new(StoreMemoryRequest {
            key: key.to_string(),
            value: value.to_string(),
            metadata,
        });
        
        let response = self.memory_client.store_memory(request).await?;
        Ok(response.into_inner().success)
    }
    
    /// Recall memory
    pub async fn recall_memory(
        &mut self,
        key: &str,
        query: &str,
    ) -> Result<Option<graphyne_proto::graphyne::RecallMemoryResponse>> {
        let request = tonic::Request::new(RecallMemoryRequest {
            key: key.to_string(),
            query: query.to_string(),
        });
        
        let response = self.memory_client.recall_memory(request).await?;
        Ok(Some(response.into_inner()))
    }
}
