//! Graphyne CLI - Command line interface for Graphyne services
//! 
//! Provides commands for search, document management, vector operations,
//! graph operations, and memory management.

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
    
    /// Memory operations
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
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
    },
}

#[derive(Subcommand)]
enum MemoryCommands {
    /// Store a memory
    Store {
        /// Memory key
        #[arg(short, long)]
        key: String,
        
        /// Memory value
        #[arg(short, long)]
        value: String,
        
        /// Metadata as JSON object
        #[arg(short, long)]
        metadata: Option<String>,
    },
    
    /// Recall a memory
    Recall {
        /// Memory key
        #[arg(short, long)]
        key: String,
        
        /// Optional query to filter results
        #[arg(short, long)]
        query: Option<String>,
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
        GraphCommands::AddNode { id, node_type, properties } => {
            let properties = properties
                .map(|p| serde_json::from_str(&p).unwrap_or_default())
                .unwrap_or_default();
            
            let success = client.add_node(&id, &node_type, properties).await?;
            
            if success {
                println!("Node added successfully");
            } else {
                println!("Failed to add node");
            }
        }
        
        GraphCommands::AddEdge { from, to, edge_type, properties } => {
            let properties = properties
                .map(|p| serde_json::from_str(&p).unwrap_or_default())
                .unwrap_or_default();
            
            let success = client.add_edge(&from, &to, &edge_type, properties).await?;
            
            if success {
                println!("Edge added successfully");
            } else {
                println!("Failed to add edge");
            }
        }
    }
    
    Ok(())
}

/// Graph operations using HTTP
async fn graph_http(config: &ClientConfig, command: GraphCommands) -> Result<()> {
    let client = GraphyneHttpClient::new(config.clone());
    
    match command {
        GraphCommands::AddNode { id, node_type, properties } => {
            let properties = properties
                .map(|p| serde_json::from_str(&p).unwrap_or_default())
                .unwrap_or_default();
            
            let response = client.add_node(&id, &node_type, properties).await?;
            
            if response.success {
                println!("Node added successfully: {}", response.message);
            } else {
                println!("Failed to add node: {}", response.message);
            }
        }
        
        GraphCommands::AddEdge { from, to, edge_type, properties } => {
            // TODO: Implement add_edge for HTTP client
            println!("Add edge via HTTP not yet implemented");
        }
    }
    
    Ok(())
}

/// Memory operations using gRPC
async fn memory_grpc(config: &ClientConfig, command: MemoryCommands) -> Result<()> {
    let mut client = GraphyneGrpcClient::connect(config.clone()).await?;
    
    match command {
        MemoryCommands::Store { key, value, metadata } => {
            let metadata = metadata
                .map(|m| serde_json::from_str(&m).unwrap_or_default())
                .unwrap_or_default();
            
            let success = client.store_memory(&key, &value, metadata).await?;
            
            if success {
                println!("Memory stored successfully");
            } else {
                println!("Failed to store memory");
            }
        }
        
        MemoryCommands::Recall { key, query } => {
            let query = query.unwrap_or_default();
            let result = client.recall_memory(&key, &query).await?;
            
            if let Some(memory) = result {
                println!("Key: {}", memory.key);
                println!("Value: {}", memory.value);
            } else {
                println!("Memory not found");
            }
        }
    }
    
    Ok(())
}

/// Memory operations using HTTP
async fn memory_http(_config: &ClientConfig, _command: MemoryCommands) -> Result<()> {
    println!("Memory operations via HTTP not yet implemented");
    Ok(())
}
