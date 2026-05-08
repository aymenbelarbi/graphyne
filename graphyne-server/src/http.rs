//! HTTP/JSON REST API implementation using axum

use axum::{
    extract::{Query, Path, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;

// Import graphyne-core types
use graphyne_core::memory::{
    MemoryStore, MemoryEntry, MemoryType, MemoryUpdate, MemoryQuery,
    MemorySpace, ScoringConfig,
};
use graphyne_core::graph::rag::GraphRAGResult;

/// Query parameters for search endpoint
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub collection: String,
    pub bucket: String,
    pub query: String,
    pub limit: Option<i32>,
    pub mode: Option<String>,
}

/// Request body for push document endpoint
#[derive(Debug, Deserialize)]
pub struct PushDocumentBody {
    pub collection: String,
    pub bucket: String,
    pub id: String,
    pub content: String,
    pub metadata: Option<HashMap<String, String>>,
}

/// Request body for add embedding endpoint
#[derive(Debug, Deserialize)]
pub struct AddEmbeddingBody {
    pub id: String,
    pub values: Vec<f32>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Request body for add node endpoint
#[derive(Debug, Deserialize)]
pub struct AddNodeBody {
    pub id: String,
    pub node_type: String,
    pub label: Option<String>,
    pub properties: Option<HashMap<String, serde_json::Value>>,
    pub embedding: Option<Vec<f32>>,
}

/// Request body for add edge endpoint
#[derive(Debug, Deserialize)]
pub struct AddEdgeBody {
    pub from_id: String,
    pub to_id: String,
    pub edge_type: String,
    pub properties: Option<HashMap<String, serde_json::Value>>,
    pub weight: Option<f32>,
}

/// Query parameters for recall memory endpoint
#[derive(Debug, Deserialize)]
pub struct RecallMemoryParams {
    pub query: Option<String>,
    pub memory_type: Option<String>,
    pub min_importance: Option<f32>,
    pub limit: Option<i32>,
    pub space: Option<String>,
}

/// Request body for store memory endpoint
#[derive(Debug, Deserialize)]
pub struct StoreMemoryBody {
    pub memory_type: String,  // "Working", "Episodic", "Semantic", "Procedural"
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub importance: Option<f32>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub space: Option<String>,
}

/// Request body for update memory endpoint
#[derive(Debug, Deserialize)]
pub struct UpdateMemoryBody {
    pub content: Option<String>,
    pub importance: Option<f32>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub embedding: Option<Vec<f32>>,
}

/// GraphRAG query request body
#[derive(Debug, Deserialize)]
pub struct GraphRAGQueryBody {
    pub query: String,
    pub max_hops: Option<usize>,
    pub limit: Option<usize>,
}

/// Subgraph request body
#[derive(Debug, Deserialize)]
pub struct SubgraphBody {
    pub node_ids: Vec<String>,
    pub max_hops: Option<usize>,
}

/// Expand node request body
#[derive(Debug, Deserialize)]
pub struct ExpandNodeBody {
    pub depth: Option<usize>,
}

/// Generic success response
#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}

/// Search result response
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

/// Search result item
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub data: String,
}

/// Document response
#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

/// Memory response
#[derive(Debug, Serialize)]
pub struct MemoryResponse {
    pub id: String,
    pub memory_type: String,
    pub content: String,
    pub importance: f32,
    pub created_at: String,
    pub last_accessed: String,
    pub access_count: u32,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Memory list response
#[derive(Debug, Serialize)]
pub struct MemoryListResponse {
    pub memories: Vec<MemoryResponse>,
    pub scores: Option<Vec<f32>>,
}

/// Memory space response
#[derive(Debug, Serialize)]
pub struct MemorySpaceResponse {
    pub name: String,
    pub memory_types: Vec<String>,
    pub max_entries: Option<usize>,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// GraphRAG query response
#[derive(Debug, Serialize)]
pub struct GraphRAGResponse {
    pub success: bool,
    pub message: String,
    pub context: Option<String>,
    pub nodes: Option<Vec<serde_json::Value>>,
    pub edges: Option<Vec<serde_json::Value>>,
    pub confidence: Option<f32>,
    pub explanation: Option<String>,
}

/// App state holding the memory store
#[derive(Clone)]
pub struct AppState {
    pub memory_store: Arc<tokio::sync::Mutex<MemoryStore>>,
}

/// Create the HTTP router with all endpoints
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // Search endpoint
        .route("/v1/search", get(search_handler))
        
        // Document endpoints
        .route("/v1/documents", post(push_document_handler))
        
        // Vector endpoints
        .route("/v1/vectors", post(add_embedding_handler))
        
        // Graph endpoints
        .route("/v1/graph/nodes", post(add_node_handler))
        .route("/v1/graph/edges", post(add_edge_handler))
        
        // Memory endpoints
        .route("/v1/memory/store", post(store_memory_handler))
        .route("/v1/memory/recall", get(recall_memory_handler))
        .route("/v1/memory/spaces", get(list_memory_spaces_handler))
        .route("/v1/memory/:id", post(update_memory_handler).delete(delete_memory_handler))
        
        // GraphRAG endpoints
        .route("/v1/graph/rag/query", post(graphrag_query_handler))
        .route("/v1/graph/subgraph", post(subgraph_handler))
        .route("/v1/graph/expand/:node_id", post(expand_node_handler))
        
        // Add tracing layer
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Health check handler
async fn health_check() -> Json<HealthResponse> {
    info!("Health check request");
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Search handler
async fn search_handler(Query(params): Query<SearchParams>) -> Json<SearchResponse> {
    info!(
        "Search request: collection={}, bucket={}, query={}, mode={:?}",
        params.collection, params.bucket, params.query, params.mode
    );
    
    // TODO: Implement actual search using graphyne-core
    
    // Placeholder response
    let results = vec![
        SearchResult {
            id: "doc1".to_string(),
            score: 0.95,
            data: "Sample search result".to_string(),
        }
    ];
    
    Json(SearchResponse { results })
}

/// Push document handler
async fn push_document_handler(Json(body): Json<PushDocumentBody>) -> Json<SuccessResponse> {
    info!(
        "Push document: collection={}, bucket={}, id={}",
        body.collection, body.bucket, body.id
    );
    
    // TODO: Implement actual document storage using graphyne-core
    
    Json(SuccessResponse {
        success: true,
        message: "Document pushed successfully".to_string(),
    })
}

/// Add embedding handler
async fn add_embedding_handler(Json(body): Json<AddEmbeddingBody>) -> Json<SuccessResponse> {
    info!(
        "Add embedding: id={}, dimensions={}",
        body.id, body.values.len()
    );
    
    // TODO: Implement actual vector storage using graphyne-core
    
    Json(SuccessResponse {
        success: true,
        message: "Embedding added successfully".to_string(),
    })
}

/// Add node handler
async fn add_node_handler(Json(body): Json<AddNodeBody>) -> Json<SuccessResponse> {
    info!(
        "Add node: id={}, type={}",
        body.id, body.node_type
    );
    
    // TODO: Implement actual graph node addition using graphyne-core
    
    Json(SuccessResponse {
        success: true,
        message: "Node added successfully".to_string(),
    })
}

/// Add edge handler
async fn add_edge_handler(Json(body): Json<AddEdgeBody>) -> Json<SuccessResponse> {
    info!(
        "Add edge: from={}, to={}, type={}",
        body.from_id, body.to_id, body.edge_type
    );
    
    // TODO: Implement actual graph edge addition using graphyne-core
    
    Json(SuccessResponse {
        success: true,
        message: "Edge added successfully".to_string(),
    })
}

/// Store memory handler
async fn store_memory_handler(
    State(state): State<AppState>,
    Json(body): Json<StoreMemoryBody>,
) -> Json<serde_json::Value> {
    info!(
        "Store memory: type={}, space={:?}",
        body.memory_type, body.space
    );
    
    let memory_type = match body.memory_type.as_str() {
        "Working" => MemoryType::Working,
        "Episodic" => MemoryType::Episodic,
        "Semantic" => MemoryType::Semantic,
        "Procedural" => MemoryType::Procedural,
        _ => {
            return Json(serde_json::json!({
                "success": false,
                "message": format!("Invalid memory type: {}", body.memory_type)
            }));
        }
    };
    
    let mut entry = MemoryEntry::new(memory_type, body.content, body.importance.unwrap_or(0.5));
    entry.embedding = body.embedding;
    if let Some(metadata) = body.metadata {
        entry.metadata = serde_json::to_value(metadata).unwrap_or(serde_json::Value::Null);
    }
    
    let mut store = state.memory_store.lock().await;
    match store.store_memory(entry, body.space.as_deref()) {
        Ok(memory_id) => Json(serde_json::json!({
            "success": true,
            "message": "Memory stored successfully",
            "memory_id": memory_id
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to store memory: {}", e)
        })),
    }
}

/// Recall memory handler
async fn recall_memory_handler(
    State(state): State<AppState>,
    Query(params): Query<RecallMemoryParams>,
) -> Json<serde_json::Value> {
    info!(
        "Recall memory: query={:?}, memory_type={:?}",
        params.query, params.memory_type
    );
    
    let memory_types = if let Some(ref mt) = params.memory_type {
        let mtype = match mt.as_str() {
            "Working" => Some(MemoryType::Working),
            "Episodic" => Some(MemoryType::Episodic),
            "Semantic" => Some(MemoryType::Semantic),
            "Procedural" => Some(MemoryType::Procedural),
            _ => None,
        };
        mtype.map(|t| vec![t])
    } else {
        None
    };
    
    let query = MemoryQuery {
        query_text: params.query,
        embedding: None,
        memory_types,
        time_range: None,
        min_importance: params.min_importance,
        limit: params.limit.unwrap_or(10) as usize,
        include_metadata: false,
    };
    
    let store = state.memory_store.lock().await;
    match store.recall(&query) {
        Ok(results) => {
            let memories: Vec<MemoryResponse> = results.iter().map(|(entry, _)| MemoryResponse {
                id: entry.id.clone(),
                memory_type: format!("{:?}", entry.memory_type),
                content: entry.content.clone(),
                importance: entry.importance,
                created_at: entry.created_at.to_rfc3339(),
                last_accessed: entry.last_accessed.to_rfc3339(),
                access_count: entry.access_count,
                metadata: if let serde_json::Value::Object(map) = &entry.metadata {
                    map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                } else {
                    HashMap::new()
                },
            }).collect();
            
            let scores: Vec<f32> = results.iter().map(|(_, score)| *score).collect();
            
            Json(serde_json::json!({
                "success": true,
                "message": format!("Found {} memories", memories.len()),
                "memories": memories,
                "scores": scores
            }))
        }
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to recall memories: {}", e)
        })),
    }
}

/// List memory spaces handler
async fn list_memory_spaces_handler(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    info!("List memory spaces");
    
    let store = state.memory_store.lock().await;
    let spaces = store.list_spaces();
    
    let spaces_response: Vec<MemorySpaceResponse> = spaces.iter().map(|space| MemorySpaceResponse {
        name: space.name.clone(),
        memory_types: space.memory_types.iter().map(|t| format!("{:?}", t)).collect(),
        max_entries: space.max_entries,
    }).collect();
    
    Json(serde_json::json!({
        "success": true,
        "message": format!("Found {} memory spaces", spaces_response.len()),
        "spaces": spaces_response
    }))
}

/// Update memory handler
async fn update_memory_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateMemoryBody>,
) -> Json<serde_json::Value> {
    info!("Update memory: id={}", id);
    
    let update = MemoryUpdate {
        content: body.content,
        importance: body.importance,
        metadata: body.metadata.map(|m| serde_json::to_value(m).unwrap_or(serde_json::Value::Null)),
        embedding: body.embedding,
    };
    
    let mut store = state.memory_store.lock().await;
    match store.update_memory(&id, update) {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "message": "Memory updated successfully"
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to update memory: {}", e)
        })),
    }
}

/// Delete memory handler
async fn delete_memory_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    info!("Delete memory: id={}", id);
    
    let mut store = state.memory_store.lock().await;
    match store.delete_memory(&id) {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "message": "Memory deleted successfully"
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "message": format!("Failed to delete memory: {}", e)
        })),
    }
}

/// GraphRAG query handler
async fn graphrag_query_handler(
    State(state): State<AppState>,
    Json(body): Json<GraphRAGQueryBody>,
) -> Json<GraphRAGResponse> {
    info!(
        "GraphRAG query: query={}, max_hops={:?}, limit={:?}",
        body.query, body.max_hops, body.limit
    );
    
    let store = state.memory_store.lock().await;
    
    // Use GraphRAG for enhanced recall if available
    match store.recall_with_graph(&body.query, body.max_hops, body.limit) {
        Ok(rag_result) => {
            // Convert nodes and edges to JSON
            let nodes_json: Vec<serde_json::Value> = rag_result.nodes.iter().map(|n| {
                serde_json::json!({
                    "id": n.id,
                    "node_type": n.node_type,
                    "label": n.label,
                    "properties": n.properties,
                })
            }).collect();
            
            let edges_json: Vec<serde_json::Value> = rag_result.edges.iter().map(|e| {
                serde_json::json!({
                    "id": e.id,
                    "from": e.from,
                    "to": e.to,
                    "edge_type": e.edge_type,
                    "properties": e.properties,
                    "weight": e.weight,
                })
            }).collect();
            
            Json(GraphRAGResponse {
                success: true,
                message: "GraphRAG query successful".to_string(),
                context: Some(rag_result.context),
                nodes: Some(nodes_json),
                edges: Some(edges_json),
                confidence: Some(rag_result.confidence),
                explanation: Some(rag_result.explanation),
            })
        }
        Err(e) => {
            // Fall back to regular recall if GraphRAG not initialized
            Json(GraphRAGResponse {
                success: false,
                message: format!("GraphRAG query failed: {}", e),
                context: None,
                nodes: None,
                edges: None,
                confidence: None,
                explanation: None,
            })
        }
    }
}

/// Subgraph handler
async fn subgraph_handler(
    State(state): State<AppState>,
    Json(body): Json<SubgraphBody>,
) -> Json<GraphRAGResponse> {
    info!(
        "Get subgraph: node_ids={:?}, max_hops={:?}",
        body.node_ids, body.max_hops
    );
    
    let store = state.memory_store.lock().await;
    
    // TODO: Implement actual subgraph retrieval using graphyne-core
    // For now, return placeholder
    
    Json(GraphRAGResponse {
        success: true,
        message: "Subgraph retrieved successfully".to_string(),
        context: Some("Sample subgraph context".to_string()),
        nodes: Some(vec![]),
        edges: Some(vec![]),
        confidence: Some(0.5),
        explanation: Some("Placeholder implementation".to_string()),
    })
}

/// Expand node handler
async fn expand_node_handler(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
    Json(body): Json<ExpandNodeBody>,
) -> Json<GraphRAGResponse> {
    info!(
        "Expand node: node_id={}, depth={:?}",
        node_id, body.depth
    );
    
    let store = state.memory_store.lock().await;
    
    // TODO: Implement actual node expansion using graphyne-core
    // For now, return placeholder
    
    Json(GraphRAGResponse {
        success: true,
        message: "Node expanded successfully".to_string(),
        context: Some(format!("Expanded node: {}", node_id)),
        nodes: Some(vec![]),
        edges: Some(vec![]),
        confidence: Some(0.5),
        explanation: Some("Placeholder implementation".to_string()),
    })
}

/// Start the HTTP server
pub async fn start_http_server(addr: SocketAddr, state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_router(state);
    
    info!("HTTP server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
