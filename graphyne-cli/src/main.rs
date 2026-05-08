//! Graphyne CLI - Command line interface for Graphyne services
//! 
//! Provides commands for search, document management, vector operations,
//! graph operations, memory management, and GraphRAG queries.

use clap::{Parser, Subcommand};
use graphyne_client::{ClientConfig, GraphyneGrpcClient, GraphyneHttpClient, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Graphyne CLI - Interact with Graphyne search and storage services
#[derive(Parser)]
#[command(
    name = "graphyne-cli",
    version,
    about = "CLI for Graphyne search and storage services",
    long_about = None
)]
struct Cli {
    /// Server endpoint (gRPC or HTTP)
    #[arg(short, long, default_value = "http://localhost:50051")]
    endpoint: String,
    
    /// Use HTTP API instead of gRPC
    #[arg(long)]
    http: bool,
    
    /// Timeout in seconds
    #[arg(short, long, default_value = "30")]
    timeout: u64,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for documents
    Search {
        /// Search query
        #[arg(short, long)]
        query: String,
        
        /// Collection name
        #[arg(short, long, default_value = "default")]
        collection: String,
        
        /// Bucket name
        #[arg(short, long, default_value = "default")]
        bucket: String,
        
        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: i32,
        
        /// Search mode: lexical, vector, graph, hybrid
        #[arg(short, long, default_value = "hybrid")]
        mode: String,
    },
    
    /// Push a document to the index
    Push {
        /// Path to JSON file containing document data
        #[arg(short, long)]
        file: PathBuf,
        
        /// Collection name
        #[arg(short, long, default_value = "default")]
        collection: String,
        
        /// Bucket name
        #[arg(short, long, default_value = "default")]
        bucket: String,
    },
    
    /// Vector operations
    Vector {
        #[command(subcommand)]
        command: VectorCommands,
    },
    
    /// Graph operations
    Graph {
        #[command(subcommand)]
        command: GraphCommands,
    },
    
    /// Memory operations (Agent & Memory Features)
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
    },
    
    /// GraphRAG operations for enhanced reasoning
    Rag {
        #[command(subcommand)]
        command: RagCommands,
    },

    
    /// Admin operations (Control & Observability)
    Admin {
        #[command(subcommand)]
        command: AdminCommands,
    },
}

#[derive(Subcommand)]
enum VectorCommands {
    /// Add an embedding vector
    Add {
        /// Document ID
        #[arg(short, long)]
        id: String,
        
        /// Embedding values as JSON array
        #[arg(short, long)]
        values: String,
        
        /// Metadata as JSON object
        #[arg(short, long)]
        metadata: Option<String>,
    },
}

#[derive(Subcommand)]
enum GraphCommands {
    /// Add a node to the graph
    AddNode {
        /// Node ID
        #[arg(short, long)]
        id: String,
        
        /// Node type
        #[arg(short, long)]
        node_type: String,
        
        /// Node label (optional)
        #[arg(long)]
        label: Option<String>,
        
        /// Properties as JSON object
        #[arg(short, long)]
        properties: Option<String>,
    },
    
    /// Add an edge to the graph
    AddEdge {
        /// Source node ID
        #[arg(long)]
        from: String,
        
        /// Target node ID
        #[arg(long)]
        to: String,
        
        /// Edge type
        #[arg(short, long)]
        edge_type: String,
        
        /// Properties as JSON object
        #[arg(short, long)]
        properties: Option<String>,
        
        /// Edge weight (default: 1.0)
        #[arg(long, default_value = "1.0")]
        weight: f32,
    },
    
    /// Find shortest path between two nodes
    ShortestPath {
        /// Source node ID
        #[arg(long)]
        from: String,
        
        /// Target node ID
        #[arg(long)]
        to: String,
    },
    
    /// Get node centrality score
    Centrality {
        /// Node ID
        #[arg(long)]
        node_id: String,
    },
    
    /// Get node recommendations
    Recommend {
        /// Starting node ID
        #[arg(long)]
        start: String,
        
        /// Maximum number of recommendations
        #[arg(long, default_value = "10")]
        limit: usize,
    },
}

#[derive(Subcommand)]
enum MemoryCommands {
    /// Store a memory entry
    Store {
        /// Memory type: Working, Episodic, Semantic, Procedural
        #[arg(long)]
        r#type: String,
        
        /// Memory content
        #[arg(long)]
        content: String,
        
        /// Importance score (0.0 to 1.0)
        #[arg(long, default_value = "0.5")]
        importance: f32,
        
        /// Memory space (default: "default")
        #[arg(long)]
        space: Option<String>,
        
        /// Metadata as JSON object
        #[arg(long)]
        metadata: Option<String>,
    },
    
    /// Recall memories based on query
    Recall {
        /// Search query
        #[arg(long)]
        query: Option<String>,
        
        /// Memory type filter: Working, Episodic, Semantic, Procedural
        #[arg(long)]
        r#type: Option<String>,
        
        /// Minimum importance score (0.0 to 1.0)
        #[arg(long)]
        min_importance: Option<f32>,
        
        /// Maximum number of results
        #[arg(long, default_value = "10")]
        limit: i32,
        
        /// Memory space (default: "default")
        #[arg(long)]
        space: Option<String>,
    },
    
    /// List memory spaces
    Spaces,
    
    /// Update an existing memory
    Update {
        /// Memory ID
        #[arg(long)]
        id: String,
        
        /// New content (optional)
        #[arg(long)]
        content: Option<String>,
        
        /// New importance score (optional)
        #[arg(long)]
        importance: Option<f32>,
        
        /// New metadata as JSON object (optional)
        #[arg(long)]
        metadata: Option<String>,
    },
    
    /// Delete a memory
    Delete {
        /// Memory ID
        #[arg(long)]
        id: String,
    },
}


    
#[derive(Subcommand)]
enum AdminCommands {
    /// Show admin statistics
    Stats {
        /// Use HTTP API instead of gRPC
        #[arg(long)]
        http: bool,
    },
    
    /// Check health status
    Health {
        /// Use HTTP API instead of gRPC
        #[arg(long)]
        http: bool,
    },
    
    /// Flush all data to disk
    Flush {
        /// Use HTTP API instead of gRPC
        #[arg(long)]
        http: bool,
    },
    
    /// Create a backup
    Backup {
        /// Backup file path
        #[arg(long)]
        path: String,
        
        /// Use HTTP API instead of gRPC
        #[arg(long)]
        http: bool,
    },
    
    /// Restore from backup
    Restore {
        /// Backup file path
        #[arg(long)]
        path: String,
        
        /// Use HTTP API instead of gRPC
        #[arg(long)]
        http: bool,
    },
    
    /// Compact storage
    Compact {
        /// Use HTTP API instead of gRPC
        #[arg(long)]
        http: bool,
    },
}


    
#[derive(Subcommand)]
enum RagCommands {
    /// Query the knowledge graph using GraphRAG
    Query {
        /// Query string
        #[arg(short, long)]
        query: String,
        
        /// Maximum number of hops to traverse
        #[arg(long, default_value = "2")]
        hops: usize,
        
        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    
    /// Get a subgraph around specific nodes
    Subgraph {
        /// Node IDs (comma-separated)
        #[arg(long)]
        node_ids: String,
        
        /// Maximum number of hops from the nodes
        #[arg(long, default_value = "2")]
        hops: usize,
    },
    
    /// Expand a node with its neighbors
    Expand {
        /// Node ID to expand
        #[arg(long)]
        node_id: String,
        
        /// Depth of expansion (1 = direct neighbors)
        #[arg(long, default_value = "1")]
        depth: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    // Create client config
    let config = ClientConfig {
        endpoint: cli.endpoint.clone(),
        timeout: Duration::from_secs(cli.timeout),
    };
    
    // Execute command
    match cli.command {
        Commands::Search { query, collection, bucket, limit, mode } => {
            if cli.http {
                search_http(&config, &query, &collection, &bucket, limit, &mode).await?;
            } else {
                search_grpc(&config, &query, &collection, &bucket, limit, &mode).await?;
            }
        }
        
        Commands::Push { file, collection, bucket } => {
            if cli.http {
                push_http(&config, &file, &collection, &bucket).await?;
            } else {
                push_grpc(&config, &file, &collection, &bucket).await?;
            }
        }
        
        Commands::Vector { command } => {
            if cli.http {
                vector_http(&config, command).await?;
            } else {
                vector_grpc(&config, command).await?;
            }
        }
        
        Commands::Graph { command } => {
            if cli.http {
                graph_http(&config, command).await?;
            } else {
                graph_grpc(&config, command).await?;
            }
        }
        
        Commands::Memory { command } => {
            if cli.http {
                memory_http(&config, command).await?;
            } else {
                memory_grpc(&config, command).await?;
            }
        }
        
        Commands::Rag { command } => {
            if cli.http {
                rag_http(&config, command).await?;
            } else {
                rag_grpc(&config, command).await?;
            }
        }
        
        Commands::Admin { command } => {
            if cli.http {
                admin_http(&config, command).await?;
            } else {
                admin_grpc(&config, command).await?;
            }
        }
    }
    
    Ok(())
}

/// Search using gRPC
async fn search_grpc(
    config: &ClientConfig,
    query: &str,
    collection: &str,
    bucket: &str,
    limit: i32,
    mode: &str,
) -> Result<()> {
    let mut client = GraphyneGrpcClient::connect(config.clone()).await?;
    let results = client.search(collection, bucket, query, limit, mode).await?;
    
    println!("Search results ({} found):", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("{}. [{}] {} (score: {})", i + 1, result.id, result.data, result.score);
    }
    
    Ok(())
}

/// Search using HTTP
async fn search_http(
    config: &ClientConfig,
    query: &str,
    collection: &str,
    bucket: &str,
    limit: i32,
    mode: &str,
) -> Result<()> {
    let client = GraphyneHttpClient::new(config.clone());
    let response = client.search(collection, bucket, query, limit, mode).await?;
    
    println!("Search results ({} found):", response.results.len());
    for (i, result) in response.results.iter().enumerate() {
        println!("{}. [{}] {} (score: {})", i + 1, result.id, result.data, result.score);
    }
    
    Ok(())
}

/// Push document using gRPC
async fn push_grpc(
    config: &ClientConfig,
    file: &PathBuf,
    collection: &str,
    bucket: &str,
) -> Result<()> {
    let content = std::fs::read_to_string(file)
        .map_err(|e| graphyne_client::ClientError::Config(e.to_string()))?;
    
    let doc: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| graphyne_client::ClientError::Config(e.to_string()))?;
    
    let id = doc["id"].as_str().unwrap_or("unknown").to_string();
    let content = doc["content"].as_str().unwrap_or("").to_string();
    let metadata = HashMap::new(); // TODO: Extract from JSON
    
    let mut client = GraphyneGrpcClient::connect(config.clone()).await?;
    let success = client.push_document(collection, bucket, &id, &content, metadata).await?;
    
    if success {
        println!("Document pushed successfully");
    } else {
        println!("Failed to push document");
    }
    
    Ok(())
}

/// Push document using HTTP
async fn push_http(
    config: &ClientConfig,
    file: &PathBuf,
    collection: &str,
    bucket: &str,
) -> Result<()> {
    let content = std::fs::read_to_string(file)
        .map_err(|e| graphyne_client::ClientError::Config(e.to_string()))?;
    
    let doc: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| graphyne_client::ClientError::Config(e.to_string()))?;
    
    let id = doc["id"].as_str().unwrap_or("unknown").to_string();
    let content = doc["content"].as_str().unwrap_or("").to_string();
    let metadata = HashMap::new(); // TODO: Extract from JSON
    
    let client = GraphyneHttpClient::new(config.clone());
    let response = client.push_document(collection, bucket, &id, &content, metadata).await?;
    
    if response.success {
        println!("Document pushed successfully: {}", response.message);
    } else {
        println!("Failed to push document: {}", response.message);
    }
    
    Ok(())
}

/// Vector operations using gRPC
async fn vector_grpc(config: &ClientConfig, command: VectorCommands) -> Result<()> {
    let mut client = GraphyneGrpcClient::connect(config.clone()).await?;
    
    match command {
        VectorCommands::Add { id, values, metadata } => {
            let values: Vec<f32> = serde_json::from_str(&values)
                .map_err(|e| graphyne_client::ClientError::Config(e.to_string()))?;
            
            let metadata = metadata
                .map(|m| serde_json::from_str(&m).unwrap_or_default())
                .unwrap_or_default();
            
            let success = client.add_embedding(&id, values, metadata).await?;
            
            if success {
                println!("Embedding added successfully");
            } else {
                println!("Failed to add embedding");
            }
        }
    }
    
    Ok(())
}

/// Vector operations using HTTP
async fn vector_http(config: &ClientConfig, command: VectorCommands) -> Result<()> {
    let client = GraphyneHttpClient::new(config.clone());
    
    match command {
        VectorCommands::Add { id, values, metadata } => {
            let values: Vec<f32> = serde_json::from_str(&values)
                .map_err(|e| graphyne_client::ClientError::Config(e.to_string()))?;
            
            let metadata = metadata
                .map(|m| serde_json::from_str(&m).unwrap_or_default())
                .unwrap_or_default();
            
            let response = client.add_embedding(&id, values, metadata).await?;
            
            if response.success {
                println!("Embedding added successfully: {}", response.message);
            } else {
                println!("Failed to add embedding: {}", response.message);
            }
        }
    }
    
    Ok(())
}

/// Graph operations using gRPC
async fn graph_grpc(config: &ClientConfig, command: GraphCommands) -> Result<()> {
    let mut client = GraphyneGrpcClient::connect(config.clone()).await?;
    
    match command {
        GraphCommands::AddNode { id, node_type, label, properties } => {
            let properties = properties
                .map(|p| serde_json::from_str(&p).unwrap_or_default())
                .unwrap_or_default();
            
            let success = client.add_node(&id, &node_type, label.as_deref(), properties).await?;
            
            if success {
                println!("Node added successfully");
            } else {
                println!("Failed to add node");
            }
        }
        
        GraphCommands::AddEdge { from, to, edge_type, properties, weight } => {
            let properties = properties
                .map(|p| serde_json::from_str(&p).unwrap_or_default())
                .unwrap_or_default();
            
            let success = client.add_edge(&from, &to, &edge_type, properties, weight).await?;
            
            if success {
                println!("Edge added successfully");
            } else {
                println!("Failed to add edge");
            }
        }
        
        GraphCommands::ShortestPath { from, to } => {
            println!("Finding shortest path from {} to {}...", from, to);
            // TODO: Implement shortest path via gRPC
            println!("Shortest path functionality not yet implemented via gRPC");
        }
        
        GraphCommands::Centrality { node_id } => {
            println!("Calculating centrality for node {}...", node_id);
            // TODO: Implement centrality via gRPC
            println!("Centrality functionality not yet implemented via gRPC");
        }
        
        GraphCommands::Recommend { start, limit } => {
            println!("Getting recommendations for node {} (limit: {})...", start, limit);
            // TODO: Implement recommendations via gRPC
            println!("Recommendation functionality not yet implemented via gRPC");
        }
    }
    
    Ok(())
}

/// Graph operations using HTTP
async fn graph_http(config: &ClientConfig, command: GraphCommands) -> Result<()> {
    let client = GraphyneHttpClient::new(config.clone());
    
    match command {
        GraphCommands::AddNode { id, node_type, label, properties } => {
            let properties = properties
                .map(|p| serde_json::from_str(&p).unwrap_or_default())
                .unwrap_or_default();
            
            let response = client.add_node(&id, &node_type, label.as_deref(), properties).await?;
            
            if response.success {
                println!("Node added successfully: {}", response.message);
            } else {
                println!("Failed to add node: {}", response.message);
            }
        }
        
        GraphCommands::AddEdge { from, to, edge_type, properties, weight } => {
            // TODO: Implement add_edge for HTTP client
            println!("Add edge via HTTP not yet implemented");
        }
        
        GraphCommands::ShortestPath { from, to } => {
            println!("Finding shortest path from {} to {}...", from, to);
            // TODO: Implement shortest path via HTTP
            println!("Shortest path functionality not yet implemented via HTTP");
        }
        
        GraphCommands::Centrality { node_id } => {
            println!("Calculating centrality for node {}...", node_id);
            // TODO: Implement centrality via HTTP
            println!("Centrality functionality not yet implemented via HTTP");
        }
        
        GraphCommands::Recommend { start, limit } => {
            println!("Getting recommendations for node {} (limit: {})...", start, limit);
            // TODO: Implement recommendations via HTTP
            println!("Recommendation functionality not yet implemented via HTTP");
        }
    }
    
    Ok(())
}

/// Memory operations using gRPC
async fn memory_grpc(config: &ClientConfig, command: MemoryCommands) -> Result<()> {
    let mut client = GraphyneGrpcClient::connect(config.clone()).await?;
    
    match command {
        MemoryCommands::Store { r#type, content, importance, space, metadata } => {
            let metadata = metadata
                .map(|m| serde_json::from_str(&m).unwrap_or_default())
                .unwrap_or_default();
            
            // TODO: Update gRPC client to support new memory store API
            println!("Memory store via gRPC not yet fully implemented with new API");
            println!("Type: {}, Content: {}, Importance: {}", r#type, content, importance);
        }
        
        MemoryCommands::Recall { query, r#type, min_importance, limit, space } => {
            // TODO: Update gRPC client to support new memory recall API
            println!("Memory recall via gRPC not yet fully implemented with new API");
        }
        
        MemoryCommands::Spaces => {
            // TODO: Implement list memory spaces via gRPC
            println!("List memory spaces via gRPC not yet implemented");
        }
        
        MemoryCommands::Update { id, content, importance, metadata } => {
            // TODO: Implement update memory via gRPC
            println!("Update memory via gRPC not yet implemented");
        }
        
        MemoryCommands::Delete { id } => {
            // TODO: Implement delete memory via gRPC
            println!("Delete memory via gRPC not yet implemented");
        }
    }
    
    Ok(())
}

/// Memory operations using HTTP
async fn memory_http(config: &ClientConfig, command: MemoryCommands) -> Result<()> {
    let client = GraphyneHttpClient::new(config.clone());
    
    match command {
        MemoryCommands::Store { r#type, content, importance, space, metadata } => {
            let metadata_map: HashMap<String, serde_json::Value> = metadata
                .map(|m| serde_json::from_str(&m).unwrap_or_default())
                .unwrap_or_default();
            
            // Convert metadata to proper format
            let metadata_value = if metadata_map.is_empty() {
                None
            } else {
                Some(metadata_map)
            };
            
            // TODO: Implement store_memory in HTTP client
            println!("Storing memory via HTTP...");
            println!("Type: {}", r#type);
            println!("Content: {}", content);
            println!("Importance: {}", importance);
            println!("Space: {:?}", space);
        }
        
        MemoryCommands::Recall { query, r#type, min_importance, limit, space } => {
            // TODO: Implement recall_memory in HTTP client
            println!("Recalling memories via HTTP...");
            println!("Query: {:?}", query);
            println!("Type: {:?}", r#type);
            println!("Min Importance: {:?}", min_importance);
            println!("Limit: {}", limit);
            println!("Space: {:?}", space);
        }
        
        MemoryCommands::Spaces => {
            // TODO: Implement list_spaces in HTTP client
            println!("Listing memory spaces via HTTP...");
        }
        
        MemoryCommands::Update { id, content, importance, metadata } => {
            // TODO: Implement update_memory in HTTP client
            println!("Updating memory via HTTP...");
            println!("ID: {}", id);
        }
        
        MemoryCommands::Delete { id } => {
            // TODO: Implement delete_memory in HTTP client
            println!("Deleting memory via HTTP...");
            println!("ID: {}", id);
        }
    }
    
    Ok(())
}

/// GraphRAG operations using gRPC
async fn rag_grpc(config: &ClientConfig, command: RagCommands) -> Result<()> {
    let mut client = GraphyneGrpcClient::connect(config.clone()).await?;
    
    match command {
        RagCommands::Query { query, hops, limit } => {
            println!("GraphRAG query: \"{}\" (hops: {}, limit: {})", query, hops, limit);
            // TODO: Implement GraphRAG query via gRPC
            println!("GraphRAG query via gRPC not yet implemented");
        }
        
        RagCommands::Subgraph { node_ids, hops } => {
            let ids: Vec<&str> = node_ids.split(',').map(|s| s.trim()).collect();
            println!("Getting subgraph for nodes: {:?} (hops: {})", ids, hops);
            // TODO: Implement subgraph via gRPC
            println!("Subgraph via gRPC not yet implemented");
        }
        
        RagCommands::Expand { node_id, depth } => {
            println!("Expanding node: {} (depth: {})", node_id, depth);
            // TODO: Implement expand node via gRPC
            println!("Expand node via gRPC not yet implemented");
        }
    }
    
    Ok(())
}

/// GraphRAG operations using HTTP
async fn rag_http(config: &ClientConfig, command: RagCommands) -> Result<()> {
    let client = GraphyneHttpClient::new(config.clone());
    
    match command {
        RagCommands::Query { query, hops, limit } => {
            println!("GraphRAG query: \"{}\" (hops: {}, limit: {})", query, hops, limit);
            // TODO: Implement GraphRAG query via HTTP
            // This would call the /v1/graph/rag/query endpoint
            println!("GraphRAG query via HTTP not yet implemented");
            println!("Would call: POST /v1/graph/rag/query with query=\"{}\", max_hops={}, limit={}", query, hops, limit);
        }
        
        RagCommands::Subgraph { node_ids, hops } => {
            let ids: Vec<&str> = node_ids.split(',').map(|s| s.trim()).collect();
            println!("Getting subgraph for nodes: {:?} (hops: {})", ids, hops);
            // TODO: Implement subgraph via HTTP
            // This would call the /v1/graph/subgraph endpoint
            println!("Subgraph via HTTP not yet implemented");
            println!("Would call: POST /v1/graph/subgraph with node_ids={:?}, max_hops={}", ids, hops);
        }
        
        RagCommands::Expand { node_id, depth } => {
            println!("Expanding node: {} (depth: {})", node_id, depth);
            // TODO: Implement expand node via HTTP
            // This would call the /v1/graph/expand/{node_id} endpoint
            println!("Expand node via HTTP not yet implemented");
            println!("Would call: POST /v1/graph/expand/{} with depth={}", node_id, depth);
        }
    }
    
    Ok(())
}

/// Admin operations using gRPC
async fn admin_grpc(config: &ClientConfig, command: AdminCommands) -> Result<()> {
    let mut client = GraphyneGrpcClient::connect(config.clone()).await?;
    
    match command {
        AdminCommands::Stats { .. } => {
            let response = client.get_admin_stats().await?;
            println!("Admin Statistics:");
            println!("  Uptime: {} seconds", response.uptime_seconds);
            println!("  Total Searches: {}", response.total_searches);
            println!("  Total Memories: {}", response.total_memories);
            println!("  Storage Size: {} bytes", response.storage_size_bytes);
        }
        
        AdminCommands::Health { .. } => {
            let response = client.get_admin_health().await?;
            println!("Health Status: {}", response.status);
            println!("Version: {}", response.version);
            println!("Uptime: {} seconds", response.uptime_seconds);
            for (name, check) in response.checks {
                println!("  {}: {} - {}", name, check.status, check.message);
            }
        }
        
        AdminCommands::Flush { .. } => {
            client.admin_flush().await?;
            println!("Flush completed successfully");
        }
        
        AdminCommands::Backup { path, .. } => {
            client.admin_backup(&path).await?;
            println!("Backup created at: {}", path);
        }
        
        AdminCommands::Restore { path, .. } => {
            client.admin_restore(&path).await?;
            println!("Restore completed from: {}", path);
        }
        
        AdminCommands::Compact { .. } => {
            client.admin_compact().await?;
            println!("Compaction completed successfully");
        }
    }
    
    Ok(())
}

/// Admin operations using HTTP
async fn admin_http(config: &ClientConfig, command: AdminCommands) -> Result<()> {
    let client = GraphyneHttpClient::new(config.clone());
    
    match command {
        AdminCommands::Stats { .. } => {
            let stats = client.admin_stats().await?;
            println!("Admin Statistics:");
            println!("  Uptime: {} seconds", stats.uptime_seconds);
            println!("  Total Searches: {}", stats.total_searches);
            println!("  Total Memories: {}", stats.total_memories);
            println!("  Storage Size: {} bytes", stats.storage_size_bytes);
        }
        
        AdminCommands::Health { .. } => {
            let health = client.admin_health().await?;
            println!("Health Status: {}", health.status);
            println!("Version: {}", health.version);
            println!("Uptime: {} seconds", health.uptime_seconds);
            for (name, check) in health.checks {
                println!("  {}: {} - {}", name, check.status, check.message);
            }
        }
        
        AdminCommands::Flush { .. } => {
            client.admin_flush().await?;
            println!("Flush completed successfully");
        }
        
        AdminCommands::Backup { path, .. } => {
            client.admin_backup(&path).await?;
            println!("Backup created at: {}", path);
        }
        
        AdminCommands::Restore { path, .. } => {
            client.admin_restore(&path).await?;
            println!("Restore completed from: {}", path);
        }
        
        AdminCommands::Compact { .. } => {
            client.admin_compact().await?;
            println!("Compaction completed successfully");
        }
    }
    
    Ok(())
}

/// Admin operations using gRPC
async fn admin_grpc(config: &ClientConfig, command: AdminCommands) -> Result<()> {
    let mut client = GraphyneGrpcClient::connect(config.clone()).await?;
    
    match command {
        AdminCommands::Stats { .. } => {
            let response = client.get_admin_stats().await?;
            println!("Admin Statistics:");
            println!("  Uptime: {} seconds", response.uptime_seconds);
            println!("  Total Searches: {}", response.total_searches);
            println!("  Total Memories: {}", response.total_memories);
            println!("  Storage Size: {} bytes", response.storage_size_bytes);
        }
        
        AdminCommands::Health { .. } => {
            let response = client.get_admin_health().await?;
            println!("Health Status: {}", response.status);
            println!("Version: {}", response.version);
            println!("Uptime: {} seconds", response.uptime_seconds);
            for (name, check) in response.checks {
                println!("  {}: {} - {}", name, check.status, check.message);
            }
        }
        
        AdminCommands::Flush { .. } => {
            client.admin_flush().await?;
            println!("Flush completed successfully");
        }
        
        AdminCommands::Backup { path, .. } => {
            client.admin_backup(&path).await?;
            println!("Backup created at: {}", path);
        }
        
        AdminCommands::Restore { path, .. } => {
            client.admin_restore(&path).await?;
            println!("Restore completed from: {}", path);
        }
        
        AdminCommands::Compact { .. } => {
            client.admin_compact().await?;
            println!("Compaction completed successfully");
        }
    }
    
    Ok(())
}

/// Admin operations using HTTP
async fn admin_http(config: &ClientConfig, command: AdminCommands) -> Result<()> {
    let client = GraphyneHttpClient::new(config.clone());
    
    match command {
        AdminCommands::Stats { .. } => {
            let stats = client.admin_stats().await?;
            println!("Admin Statistics:");
            println!("  Uptime: {} seconds", stats.uptime_seconds);
            println!("  Total Searches: {}", stats.total_searches);
            println!("  Total Memories: {}", stats.total_memories);
            println!("  Storage Size: {} bytes", stats.storage_size_bytes);
        }
        
        AdminCommands::Health { .. } => {
            let health = client.admin_health().await?;
            println!("Health Status: {}", health.status);
            println!("Version: {}", health.version);
            println!("Uptime: {} seconds", health.uptime_seconds);
            for (name, check) in health.checks {
                println!("  {}: {} - {}", name, check.status, check.message);
            }
        }
        
        AdminCommands::Flush { .. } => {
            client.admin_flush().await?;
            println!("Flush completed successfully");
        }
        
        AdminCommands::Backup { path, .. } => {
            client.admin_backup(&path).await?;
            println!("Backup created at: {}", path);
        }
        
        AdminCommands::Restore { path, .. } => {
            client.admin_restore(&path).await?;
            println!("Restore completed from: {}", path);
        }
        
        AdminCommands::Compact { .. } => {
            client.admin_compact().await?;
            println!("Compaction completed successfully");
        }
    }
    
    Ok(())
}
