# Graphyne

A high-performance, agent-native search and knowledge graph engine written in Rust.

## Overview

Graphyne is designed for hybrid lexical, vector, and graph retrieval with agent-first architecture. It provides:

- **Hybrid Retrieval**: Unifies lexical (BM25), vector (ANN), and graph search
- **Agent-Native**: Built-in memory spaces (working, episodic, semantic, procedural)
- **High Performance**: Single-digit ms p99 latency for common operations
- **Multiple APIs**: gRPC, HTTP/JSON, and CLI interfaces

## Architecture

Graphyne uses a four-layer architecture:

1. **API Layer**: gRPC, HTTP/JSON, CLI
2. **Query Layer**: Lexical, vector, graph engines + hybrid scorer
3. **Data Layer**: Embedded Rust KV store, WAL, index storage
4. **Control & Observability**: Metrics, health checks, admin operations

## Project Structure

```
graphyne/
├── Cargo.toml              # Workspace configuration
├── graphyne-core/          # Core library with error handling and storage traits
│   ├── src/
│   │   ├── lib.rs         # Library root with module declarations
│   │   ├── error.rs       # Error types (GraphyneError)
│   │   ├── storage/       # Storage abstraction layer
│   │   │   └── mod.rs     # StorageBackend trait
│   │   ├── lexical/       # Lexical search engine (BM25, FST)
│   │   │   └── mod.rs     # FST-based term indexing with BM25 scoring
│   │   ├── vector/        # Vector search engine (HNSW, ANN)
│   │   │   └── mod.rs     # HNSW index for approximate nearest neighbor search
│   │   ├── graph/         # Graph storage (petgraph, typed edges)
│   │   │   └── mod.rs     # Typed property graph with multi-hop traversal
│   │   └── scoring/       # Hybrid scoring system
│   │       └── mod.rs     # Combine lexical, vector, and graph scores
│   └── Cargo.toml         # Dependencies: fst, sled, hnsw-rs, petgraph, etc.
├── graphyne-server/        # Server binary with gRPC and HTTP APIs
│   ├── src/
│   │   ├── main.rs        # Server startup with gRPC and HTTP servers
│   │   ├── grpc.rs        # gRPC service implementations
│   │   └── http.rs        # HTTP/JSON REST API with axum
│   └── Cargo.toml
├── graphyne-client/        # Client SDK for gRPC and HTTP
│   ├── src/
│   │   ├── lib.rs         # Client SDK library root
│   │   ├── grpc.rs        # gRPC client implementation
│   │   ├── http.rs        # HTTP client implementation
│   │   └── error.rs       # Client error types
│   └── Cargo.toml
├── graphyne-cli/           # CLI tool with clap
│   ├── src/
│   │   └── main.rs        # CLI commands: search, push, vector, graph, memory
│   └── Cargo.toml
├── graphyne-proto/         # gRPC/protobuf schemas
│   ├── proto/
│   │   └── graphyne.proto # Service definitions (Search, Document, Vector, Graph, Memory)
│   ├── src/
│   │   └── lib.rs         # Generated code exports
│   ├── build.rs           # tonic-build configuration
│   └── Cargo.toml
├── graphyne-mcp/           # MCP integration for agent tooling (planned)
└── README.md
```

## Core Retrieval Engines (Phase 2)

### Lexical Search Engine
- **FST-based Term Indexing**: Uses Finite State Transducers for fast term lookup
- **BM25 Scoring**: Implements the BM25 ranking algorithm for relevance scoring
- **Text Tokenization**: Unicode-aware tokenization with language detection (via whatlang)
- **Prefix and Fuzzy Matching**: Fast prefix search using FST structure

### Vector Search Engine
- **HNSW Algorithm**: Hierarchical Navigable Small World graphs for ANN search
- **Cosine Similarity**: Default distance metric for vector comparison
- **Embedding Storage**: Persistent storage of embeddings using sled KV store
- **Configurable Parameters**: Adjust M, ef_construction, and ef for performance tuning

### Graph Storage
- **Typed Property Graph**: Nodes and edges with types and arbitrary properties
- **petgraph Integration**: Uses petgraph for in-memory graph operations
- **Multi-hop Traversal**: Breadth-first search with configurable hop limits
- **Persistent Storage**: Graph state persisted in sled with automatic reload

### Hybrid Scoring
- **Configurable Weights**: Adjust lexical, vector, and graph score contributions
- **Score Normalization**: Automatic normalization to 0-1 range
- **Flexible Combination**: Combine any subset of retrieval methods

## API Layer (Phase 3)

### gRPC API

Graphyne provides a gRPC API using tonic with the following services:

#### SearchService
- `Search(SearchRequest) -> SearchResponse` - Hybrid search across collections
- `Suggest(SuggestRequest) -> SuggestResponse` - Query suggestions

#### DocumentService
- `PushDocument(PushDocumentRequest) -> PushDocumentResponse` - Index documents
- `PopDocument(PopDocumentRequest) -> PopDocumentResponse` - Remove documents
- `GetDocument(GetDocumentRequest) -> GetDocumentResponse` - Retrieve documents

#### VectorService
- `AddEmbedding(AddEmbeddingRequest) -> AddEmbeddingResponse` - Store embeddings
- `SearchVector(SearchVectorRequest) -> SearchVectorResponse` - Vector similarity search

#### GraphService
- `AddNode(AddNodeRequest) -> AddNodeResponse` - Add graph nodes
- `AddEdge(AddEdgeRequest) -> AddEdgeResponse` - Add graph edges
- `TraverseGraph(TraverseGraphRequest) -> TraverseGraphResponse` - Graph traversal

#### MemoryService
- `StoreMemory(StoreMemoryRequest) -> StoreMemoryResponse` - Store agent memories
- `RecallMemory(RecallMemoryRequest) -> RecallMemoryResponse` - Recall memories

### HTTP/JSON REST API

The HTTP API runs on port 8080 with the following endpoints:

#### Search
```bash
GET /v1/search?collection=default&bucket=default&query=test&limit=10&mode=hybrid
```

#### Documents
```bash
POST /v1/documents
Content-Type: application/json

{
  "collection": "default",
  "bucket": "default",
  "id": "doc1",
  "content": "Document content here",
  "metadata": {"source": "user"}
}
```

#### Vectors
```bash
POST /v1/vectors
Content-Type: application/json

{
  "id": "vec1",
  "values": [0.1, 0.2, 0.3, ...],
  "metadata": {"type": "embedding"}
}
```

#### Graph Nodes
```bash
POST /v1/graph/nodes
Content-Type: application/json

{
  "id": "node1",
  "node_type": "Person",
  "properties": {"name": "John", "age": "30"}
}
```

#### Graph Edges
```bash
POST /v1/graph/edges
Content-Type: application/json

{
  "from_id": "node1",
  "to_id": "node2",
  "edge_type": "KNOWS",
  "properties": {"since": "2023"}
}
```

#### Memory Recall
```bash
GET /v1/memory/recall?key=user:123&query=preferences
```

### CLI Usage

The `graphyne-cli` tool provides command-line access to all APIs:

#### Search
```bash
graphyne-cli search --query "test query" --collection default --bucket default --limit 10 --mode hybrid
```

#### Push Document
```bash
graphyne-cli push --file document.json --collection default --bucket default
```

Example `document.json`:
```json
{
  "id": "doc1",
  "content": "This is a sample document"
}
```

#### Vector Operations
```bash
graphyne-cli vector add --id "vec1" --values "[0.1, 0.2, 0.3, 0.4, 0.5]"
```

#### Graph Operations
```bash
# Add node
graphyne-cli graph add-node --id "node1" --type "Person" --properties '{"name": "John"}'

# Add edge
graphyne-cli graph add-edge --from "node1" --to "node2" --type "KNOWS" --properties '{"since": "2023"}'
```

#### Memory Operations
```bash
# Store memory
graphyne-cli memory store --key "user:123" --value "User prefers dark mode"

# Recall memory
graphyne-cli memory recall --key "user:123"
```

### Using the Client SDK

#### gRPC Client (Rust)
```rust
use graphyne_client::{GraphyneGrpcClient, ClientConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClientConfig {
        endpoint: "http://localhost:50051".to_string(),
        timeout: Duration::from_secs(30),
    };
    
    let mut client = GraphyneGrpcClient::connect(config).await?;
    
    let results = client.search("default", "default", "query", 10, "hybrid").await?;
    println!("Found {} results", results.len());
    
    Ok(())
}
```

#### HTTP Client (Rust)
```rust
use graphyne_client::{GraphyneHttpClient, ClientConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClientConfig {
        endpoint: "http://localhost:8080".to_string(),
        timeout: Duration::from_secs(30),
    };
    
    let client = GraphyneHttpClient::new(config);
    
    let response = client.search("default", "default", "query", 10, "hybrid").await?;
    println!("Found {} results", response.results.len());
    
    Ok(())
}
```

## Getting Started

### Prerequisites

- Rust (edition 2021 or later)
- Cargo package manager
- protoc (for gRPC/protobuf compilation)

### Build Instructions

```bash
# Clone the repository
git clone https://github.com/yourusername/graphyne.git
cd graphyne/graphyne

# Build the workspace
cargo build

# Run the server
cargo run -p graphyne-server

# Use the CLI
cargo run -p graphyne-cli -- search --query "test"
```

### Server Ports

- **gRPC**: Port 50051
- **HTTP/JSON**: Port 8080
- **Health Check**: `GET http://localhost:8080/health`

## Development Status

### Phase 1 (Complete) ✅
- ✅ Cargo workspace initialization
- ✅ graphyne-core crate with error handling (`GraphyneError`)
- ✅ Storage abstraction traits (`StorageBackend`)
- ✅ graphyne-server skeleton binary

### Phase 2 (Complete) ✅
- ✅ Lexical search engine with BM25 scoring and FST indexing
- ✅ Vector search engine with HNSW (hnsw-rs) for ANN
- ✅ Graph storage with petgraph for typed property graph
- ✅ Hybrid scoring to combine retrieval results
- ✅ Unit tests for core components

### Phase 3 (Complete) ✅
- ✅ graphyne-proto crate with protobuf definitions
- ✅ gRPC server implementation using tonic
- ✅ HTTP/JSON REST API using axum
- ✅ graphyne-client SDK crate (gRPC and HTTP clients)
- ✅ graphyne-cli binary with clap-based CLI
- ✅ Integrated server startup (gRPC + HTTP)
- ✅ API documentation and examples

## License

Apache License 2.0

## Status

✅ **Phase 1 Complete** - Foundations implemented (Cargo workspace, core crate, server skeleton)  
✅ **Phase 2 Complete** - Core retrieval engines (lexical, vector, graph, hybrid scoring)  
✅ **Phase 3 Complete** - API Layer (gRPC, HTTP/JSON, CLI, Client SDK)
