# Graphyne

A high-performance, agent-native search and knowledge graph engine written in Rust.

## Overview

Graphyne is designed for hybrid lexical, vector, and graph retrieval with agent-first architecture. It provides:

- **Hybrid Retrieval**: Unifies lexical (BM25), vector (ANN), and graph search
- **Agent-Native**: Built-in memory spaces (working, episodic, semantic, procedural)
- **Knowledge Graph**: Advanced graph reasoning with GraphRAG for enhanced LLM context
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
│   │   │   ├── mod.rs     # Typed property graph with multi-hop traversal
│   │   │   └── rag.rs      # GraphRAG engine for LLM context generation
│   │   ├── scoring/       # Hybrid scoring system
│   │   │   └── mod.rs     # Combine lexical, vector, and graph scores
│   │   └── memory/        # Agent & Memory system (Phase 4)
│   │       ├── mod.rs      # Memory module root
│   │       ├── types.rs    # Memory types: Working, Episodic, Semantic, Procedural
│   │       ├── retention.rs # Retention policies for memory management
│   │       ├── store.rs    # MemoryStore with CRUD and integration
│   │       ├── context.rs  # ContextPacker for LLM token budgeting
│   │       └── scoring.rs  # MemoryScorer with configurable weights
│   └── Cargo.toml         # Dependencies: fst, sled, hnsw-rs, petgraph, chrono, etc.
├── graphyne-server/        # Server binary with gRPC and HTTP APIs
│   ├── src/
│   │   ├── main.rs        # Server startup with gRPC and HTTP servers
│   │   ├── grpc.rs        # gRPC service implementations (including GraphRAG)
│   │   └── http.rs        # HTTP/JSON REST API with axum (including GraphRAG)
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
│   │   └── main.rs        # CLI commands: search, push, vector, graph, memory, rag
│   └── Cargo.toml
├── graphyne-proto/         # gRPC/protobuf schemas
│   ├── proto/
│   │   └── graphyne.proto # Service definitions (Search, Document, Vector, Graph, Memory, GraphRAG)
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

## Agent & Memory Features (Phase 4)

Graphyne's agent-first memory system is the core differentiator from traditional search engines. It implements a sophisticated memory architecture inspired by human memory systems.

### Memory Types

Graphyne supports four types of memory:

1. **Working Memory**: Short-term, immediate context (e.g., current conversation state)
2. **Episodic Memory**: Event-based memories with timestamps (e.g., "User asked about X at 3pm")
3. **Semantic Memory**: Factual knowledge (e.g., "User prefers dark mode")
4. **Procedural Memory**: How-to knowledge and skills (e.g., "Steps to configure server")

### Memory Store

The `MemoryStore` integrates with all three core retrieval engines:
- **Lexical Index**: Full-text search across memory content
- **Vector Index**: Semantic search using embeddings
- **Graph Store**: Relationship mapping between memories

### Retention Policies

Configurable retention policies control memory lifecycle:
- **Max Age**: Automatically archive/delete memories older than N days
- **Max Entries**: Limit memory space size
- **Importance Threshold**: Keep only memories above a certain importance score
- **Auto-archive**: Move old memories to archive instead of deleting

### Memory Scoring

The `MemoryScorer` combines multiple factors:
- **Recency**: More recent memories score higher
- **Importance**: User-defined importance (0.0 to 1.0)
- **Access Frequency**: Frequently accessed memories boost
- **Relevance**: Query relevance from lexical/vector search

Formula: `score = w1*recency + w2*importance + w3*access_freq + w4*relevance`

### Context Packing

The `ContextPacker` prepares memories for LLM context:
- **Token Budgeting**: Respects LLM token limits (default: 4096 tokens)
- **Truncation Strategies**: Head (keep end), Tail (keep beginning), Middle (keep both ends)
- **Score-based Prioritization**: Higher-scoring memories included first

### Memory Spaces

Isolated memory environments for different agents or contexts:
- Each space has its own retention policy
- Configurable scoring weights per space
- Independent memory types per space

## Knowledge Graph & GraphRAG (Phase 5)

Phase 5 introduces advanced Knowledge Graph capabilities and GraphRAG (Graph-based Retrieval-Augmented Generation) for enhanced agent reasoning.

### Enhanced Graph Data Model

The graph data model has been enhanced with rich node and edge types:

```rust
pub struct GraphNode {
    pub id: String,
    pub node_type: String,
    pub label: String,
    pub properties: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct GraphEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub edge_type: String,
    pub properties: serde_json::Value,
    pub weight: f32,
}
```

### GraphRAG Engine

The GraphRAG engine combines graph traversal with vector similarity to generate rich context for LLM consumption:

1. **Lexical Search**: Find seed nodes using text search
2. **Graph Traversal**: Expand via multi-hop traversal from seed nodes
3. **Relevance Ranking**: Rank nodes by graph centrality + vector similarity
4. **Context Generation**: Format subgraph as text for LLM

```rust
pub struct GraphRAG {
    graph_store: GraphStore,
    vector_index: VectorIndex,
    lexical_index: LexicalIndex,
}

impl GraphRAG {
    pub fn query(&self, query: &str, max_hops: usize, limit: usize) -> Result<GraphRAGResult> {
        // Returns context string, nodes, edges, and confidence score
    }
}
```

### Advanced Graph Traversal

New graph traversal algorithms have been added:

- **Shortest Path (Dijkstra)**: Find optimal paths between nodes using weighted edges
- **Node Centrality**: Calculate degree centrality for node importance
- **Graph Recommendations**: Recommend nodes based on graph structure and vector similarity
- **Neighbor Expansion**: Get all neighbors (incoming and outgoing edges)

### Graph-Memory Integration

The memory system is now fully integrated with the knowledge graph:

- **Memory-Graph Linking**: Memory entries are automatically linked to graph nodes
- **Relationship Tracking**: Store memory relationships in the graph
- **Graph-Enhanced Recall**: Use GraphRAG for enhanced memory recall
- **Automatic Node Creation**: Memory entries create corresponding graph nodes

```rust
impl MemoryStore {
    pub fn store_memory_with_graph(&mut self, entry: MemoryEntry, 
        relationships: Vec<(String, String, String)>) -> Result<String> {
        // Store memory and create graph relationships
    }
    
    pub fn recall_with_graph(&self, query: &str) -> Result<GraphRAGResult> {
        // Use GraphRAG for enhanced recall
    }
}
```

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
- `RecallMemory(RecallMemoryRequest) -> RecallMemoryResponse` - Recall memories with scoring
- `GetMemorySpaces(GetMemorySpacesRequest) -> GetMemorySpacesResponse` - List memory spaces
- `UpdateMemory(UpdateMemoryRequest) -> UpdateMemoryResponse` - Update existing memories
- `DeleteMemory(DeleteMemoryRequest) -> DeleteMemoryResponse` - Delete memories

#### GraphRAGService (NEW in Phase 5)
- `Query(GraphRAGQueryRequest) -> GraphRAGQueryResponse` - GraphRAG query for LLM context
- `GetSubgraph(SubgraphRequest) -> SubgraphResponse` - Get subgraph around nodes
- `ExpandNode(ExpandNodeRequest) -> ExpandNodeResponse` - Expand node with neighbors

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
  "label": "John Doe",
  "properties": {"name": "John", "age": "30"},
  "embedding": [0.1, 0.2, ...]
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
  "properties": {"since": "2023"},
  "weight": 1.0
}
```

#### Memory Store
```bash
POST /v1/memory/store
Content-Type: application/json

{
  "memory_type": "Semantic",
  "content": "User prefers dark mode",
  "importance": 0.8,
  "metadata": {"source": "user_reference"},
  "space": "default"
}
```

#### Memory Recall
```bash
GET /v1/memory/recall?query=preferences&memory_type=Semantic&min_importance=0.5&limit=10&space=default
```

#### Memory Spaces
```bash
GET /v1/memory/spaces
```

#### Update Memory
```bash
PUT /v1/memory/{memory_id}
Content-Type: application/json

{
  "content": "Updated: User prefers dark mode and high contrast",
  "importance": 0.9
}
```

#### GraphRAG Query (NEW in Phase 5)
```bash
POST /v1/graph/rag/query
Content-Type: application/json

{
  "query": "What does the user prefer?",
  "max_hops": 2,
  "limit": 10
}
```

Returns:
```json
{
  "success": true,
  "message": "GraphRAG query successful",
  "context": "# Knowledge Graph Context for Query: \"What does the user prefer?\"\n\n## Summary\n...",
  "nodes": [...],
  "edges": [...],
  "confidence": 0.85,
  "explanation": "Found 3 seed nodes from lexical search..."
}
```

#### Get Subgraph (NEW in Phase 5)
```bash
POST /v1/graph/subgraph
Content-Type: application/json

{
  "node_ids": ["node1", "node2"],
  "max_hops": 2
}
```

#### Expand Node (NEW in Phase 5)
```bash
POST /v1/graph/expand/{node_id}
Content-Type: application/json

{
  "depth": 1
}
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
graphyne-cli graph add-node --id "node1" --type "Person" --label "John Doe" --properties '{"name": "John"}'

# Add edge
graphyne-cli graph add-edge --from "node1" --to "node2" --type "KNOWS" --properties '{"since": "2023"}' --weight 1.0

# Shortest path
graphyne-cli graph shortest-path --from "node1" --to "node2"

# Node centrality
graphyne-cli graph centrality --node-id "node1"

# Recommendations
graphyne-cli graph recommend --start "node1" --limit 10
```

#### Memory Operations
```bash
# Store memory
graphyne-cli memory store --type "Semantic" --content "User prefers dark mode" --importance 0.8

# Recall memories
graphyne-cli memory recall --query "preferences" --type "Semantic" --min-importance 0.5 --limit 10

# List memory spaces
graphyne-cli memory spaces

# Update memory
graphyne-cli memory update --id "mem_123" --content "Updated content" --importance 0.9

# Delete memory
graphyne-cli memory delete --id "mem_123"
```

#### GraphRAG Operations (NEW in Phase 5)
```bash
# GraphRAG query
graphyne-cli rag query --query "What does the user prefer?" --hops 2 --limit 10

# Get subgraph
graphyne-cli rag subgraph --node-ids "node1,node2" --hops 2

# Expand node
graphyne-cli rag expand --node-id "node1" --depth 1
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

## GraphRAG Examples

### Example 1: Basic GraphRAG Query

```bash
# Query the knowledge graph for context
curl -X POST http://localhost:8080/v1/graph/rag/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "machine learning frameworks",
    "max_hops": 2,
    "limit": 10
  }'
```

### Example 2: Building a Knowledge Graph

```bash
# Add nodes
curl -X POST http://localhost:8080/v1/graph/nodes \
  -H "Content-Type: application/json" \
  -d '{"id": "pytorch", "node_type": "Framework", "label": "PyTorch", "properties": {"year": "2016"}}'

curl -X POST http://localhost:8080/v1/graph/nodes \
  -H "Content-Type: application/json" \
  -d '{"id": "tensorflow", "node_type": "Framework", "label": "TensorFlow", "properties": {"year": "2015"}}'

# Add relationships
curl -X POST http://localhost:8080/v1/graph/edges \
  -H "Content-Type: application/json" \
  -d '{"from_id": "pytorch", "to_id": "tensorflow", "edge_type": "SIMILAR_TO", "weight": 0.8}'

# Query the graph
curl -X POST http://localhost:8080/v1/graph/rag/query \
  -H "Content-Type: application/json" \
  -d '{"query": "deep learning frameworks", "max_hops": 2}'
```

### Example 3: Memory with Graph Relationships

```bash
# Store memory with graph relationships
cargo run -p graphyne-cli -- memory store \
  --type "Semantic" \
  --content "PyTorch is a machine learning framework" \
  --importance 0.9

# Use GraphRAG for enhanced recall
cargo run -p graphyne-cli -- rag query \
  --query "What machine learning frameworks are mentioned?" \
  --hops 2
```

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

### Phase 4 (Complete) ✅
- ✅ Memory types: Working, Episodic, Semantic, Procedural
- ✅ MemoryStore with integration to lexical, vector, and graph stores
- ✅ Retention policies (max age, max entries, importance threshold)
- ✅ Token budgeting for LLM context (ContextPacker)
- ✅ Memory scoring (recency, importance, access frequency, relevance)
- ✅ Updated gRPC, HTTP APIs and CLI with memory commands
- ✅ Agent-first architecture now fully functional

### Phase 5 (Complete) ✅
- ✅ Enhanced graph data model with rich node/edge types
- ✅ GraphRAG engine for LLM context generation
- ✅ Advanced graph traversal (shortest path, centrality, recommendations)
- ✅ Graph-memory integration for relationship tracking
- ✅ GraphRAGService added to gRPC proto and server
- ✅ HTTP endpoints for GraphRAG queries (`/v1/graph/rag/query`, `/v1/graph/subgraph`, `/v1/graph/expand`)
- ✅ CLI commands for graph reasoning (`graph rag query`, `graph rag subgraph`, `graph rag expand`)
- ✅ Graph-based agent memory recall


### Phase 6 (Complete) ✅
- ✅ Prometheus-compatible metrics (search, memory, storage, API metrics)
- ✅ Structured logging with tracing (log_search, log_memory, log_storage macros)
- ✅ Health checks with detailed status (storage, memory, overall status)
- ✅ Admin operations (stats, flush, backup, restore, compact)
- ✅ AdminService added to gRPC proto and server
- ✅ HTTP endpoints for control & observability:
  - `GET /health` - Health check
  - `GET /metrics` - Prometheus metrics
  - `GET /admin/stats` - Admin statistics
  - `POST /admin/flush` - Flush data
  - `POST /admin/backup` - Create backup
  - `POST /admin/restore` - Restore from backup
  - `POST /admin/compact` - Compact storage
- ✅ CLI admin commands (stats, health, flush, backup, restore, compact)
- ✅ Logging initialization in main.rs

## Control & Observability

Graphyne now includes production-ready observability features:

### Metrics (Prometheus-compatible)
- Search metrics: `graphyne_search_requests_total`, `graphyne_search_duration_seconds`, `graphyne_search_results_count`
- Memory metrics: `graphyne_memory_stored_total`, `graphyne_memory_recalled_total`, `graphyne_memory_entries_current`
- Storage metrics: `graphyne_storage_operations_total`, `graphyne_storage_operation_duration_seconds`
- API metrics: `graphyne_grpc_requests_total`, `graphyne_http_requests_total`

Access metrics at: `GET /metrics` (Prometheus format)

### Structured Logging
- Uses `tracing` crate with configurable log levels
- Initialize with: `graphyne_core::logging::init_logging("info")`

### Health Checks
- Overall status: `healthy`, `degraded`, `unhealthy`
- Checks storage accessibility and memory store operational status
- Detailed checks available with `?detailed=true` parameter
- Access at: `GET /health` or `GET /admin/health`

### Admin Operations
- **Stats**: Get system statistics (uptime, searches, memories, storage size)
- **Flush**: Flush all buffers to disk
- **Backup**: Create backup of data to specified path
- **Restore**: Restore data from backup
- **Compact**: Compact storage for better performance

### CLI Admin Commands
```bash
# Show admin statistics
cargo run --package graphyne-cli -- admin stats

# Check health status
cargo run --package graphyne-cli -- admin health

# Flush data to disk
cargo run --package graphyne-cli -- admin flush

# Create backup
cargo run --package graphyne-cli -- admin backup --path /path/to/backup

# Restore from backup
cargo run --package graphyne-cli -- admin restore --path /path/to/backup

# Compact storage
cargo run --package graphyne-cli -- admin compact
```


### Phase 7 (Complete) ✅ - Optional Extensions

Phase 7 implements optional extensions that enhance Graphyne's capabilities with feature-gated components.

#### Embedded Embedding Models
- ✅ Feature-gated embedding generation with `#[cfg(feature = "embeddings")]`
- ✅ Fallback hash-based embeddings when `candle` is not enabled
- ✅ Support for local inference using `candle` or `burn` (when feature enabled)
- ✅ Simple bag-of-characters embedding as fallback method
- ✅ Batch embedding generation support

**Usage:**
```rust
use graphyne_core::embeddings::EmbeddingGenerator;

// Create with default fallback method
let generator = EmbeddingGenerator::new(None)?;

// Generate embedding
let embedding = generator.generate("hello world")?;

// Batch generation
let texts = vec!["hello", "world"];
let embeddings = generator.generate_batch(&texts)?;
```

**Enable candle embeddings:**
```toml
[dependencies]
graphyne-core = { version = "0.1", features = ["embeddings"] }
```

#### Plugin System (Basic)
- ✅ `GraphynePlugin` trait definition with lifecycle hooks
- ✅ `PluginManager` for dynamic library loading (dlopen)
- ✅ Plugin lifecycle management (load, unload, register)
- ✅ Pre/post search hooks for extending functionality
- ✅ Support for statically linked plugins via `register_plugin()`

**Plugin Trait:**
```rust
pub trait GraphynePlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn on_load(&mut self) -> Result<()>;
    fn on_unload(&mut self) -> Result<()>;
    fn pre_search(&self, query: &str) -> Result<()>;
    fn post_search(&self, results: &mut Vec<(String, f32)>) -> Result<()>;
}
```

**Usage:**
```rust
use graphyne_core::plugins::{PluginManager, GraphynePlugin};

let mut manager = PluginManager::new(PathBuf::from("./plugins"));

// Register a static plugin
manager.register_plugin(Box::new(MyPlugin::new()))?;

// Load dynamic plugin from file
manager.load_plugin("my_plugin")?;

// Use hooks
manager.pre_search_all("query")?;
```

#### Basic Web UI (Optional)
- ✅ Simple single-page web interface at `graphyne-web/index.html`
- ✅ Search interface with mode selection (hybrid, lexical, vector, graph)
- ✅ Memory browser for storing and viewing memories
- ✅ System statistics dashboard
- ✅ Served automatically by the server at `/`

**Access the Web UI:**
```
http://localhost:8080/
```

**Features:**
- Search with real-time results
- Store different memory types (working, episodic, semantic, procedural)
- View system statistics (searches, memories, storage size, uptime)
- Responsive design with gradient UI

#### Feature Flags
Graphyne now supports several feature flags for optional components:

| Feature | Crate | Description |
|---------|-------|-------------|
| `embeddings` | graphyne-core | Enable candle-based embedding generation |
| `plugins` | graphyne-core | Enable plugin system support |
| `web-ui` | graphyne-server | Enable web UI serving (default: enabled) |

**Enable features:**
```toml
[dependencies]
graphyne-core = { version = "0.1", features = ["embeddings", "plugins"] }
```



### Phase 7 (Complete) ✅ - Optional Extensions

Phase 7 implements optional extensions that enhance Graphyne's capabilities with feature-gated components.

#### Embedded Embedding Models
- ✅ Feature-gated embedding generation with `#[cfg(feature = "embeddings")]`
- ✅ Fallback hash-based embeddings when `candle` is not enabled
- ✅ Support for local inference using `candle` or `burn` (when feature enabled)
- ✅ Simple bag-of-characters embedding as fallback method
- ✅ Batch embedding generation support

**Usage:**
```rust
use graphyne_core::embeddings::EmbeddingGenerator;

// Create with default fallback method
let generator = EmbeddingGenerator::new(None)?;

// Generate embedding
let embedding = generator.generate("hello world")?;

// Batch generation
let texts = vec!["hello", "world"];
let embeddings = generator.generate_batch(&texts)?;
```

**Enable candle embeddings:**
```toml
[dependencies]
graphyne-core = { version = "0.1", features = ["embeddings"] }
```

#### Plugin System (Basic)
- ✅ `GraphynePlugin` trait definition with lifecycle hooks
- ✅ `PluginManager` for dynamic library loading (dlopen)
- ✅ Plugin lifecycle management (load, unload, register)
- ✅ Pre/post search hooks for extending functionality
- ✅ Support for statically linked plugins via `register_plugin()`

**Plugin Trait:**
```rust
pub trait GraphynePlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn on_load(&mut self) -> Result<()>;
    fn on_unload(&mut self) -> Result<()>;
    fn pre_search(&self, query: &str) -> Result<()>;
    fn post_search(&self, results: &mut Vec<(String, f32)>) -> Result<()>;
}
```

**Usage:**
```rust
use graphyne_core::plugins::{PluginManager, GraphynePlugin};

let mut manager = PluginManager::new(PathBuf::from("./plugins"));

// Register a static plugin
manager.register_plugin(Box::new(MyPlugin::new()))?;

// Load dynamic plugin from file
manager.load_plugin("my_plugin")?;

// Use hooks
manager.pre_search_all("query")?;
```

#### Basic Web UI (Optional)
- ✅ Simple single-page web interface at `graphyne-web/index.html`
- ✅ Search interface with mode selection (hybrid, lexical, vector, graph)
- ✅ Memory browser for storing and viewing memories
- ✅ System statistics dashboard
- ✅ Served automatically by the server at `/`

**Access the Web UI:**
```
http://localhost:8080/
```

**Features:**
- Search with real-time results
- Store different memory types (working, episodic, semantic, procedural)
- View system statistics (searches, memories, storage size, uptime)
- Responsive design with gradient UI

#### Feature Flags
Graphyne now supports several feature flags for optional components:

| Feature | Crate | Description |
|---------|-------|-------------|
| `embeddings` | graphyne-core | Enable candle-based embedding generation |
| `plugins` | graphyne-core | Enable plugin system support |
| `web-ui` | graphyne-server | Enable web UI serving (default: enabled) |

**Enable features:**
```toml
[dependencies]
graphyne-core = { version = "0.1", features = ["embeddings", "plugins"] }
```


## License

Apache License 2.0

## Status

✅ **Phase 1 Complete** - Foundations implemented (Cargo workspace, core crate, server skeleton)  
✅ **Phase 2 Complete** - Core retrieval engines (lexical, vector, graph, hybrid scoring)  
✅ **Phase 3 Complete** - API Layer (gRPC, HTTP/JSON, CLI, Client SDK)  
✅ **Phase 4 Complete** - Agent & Memory Features (memory types, retention, scoring, context packing)  
✅ **Phase 5 Complete** - Knowledge Graph & GraphRAG (enhanced graph model, GraphRAG engine, advanced traversal, graph-memory integration)
✅ **Phase 6 Complete** - Control & Observability (metrics, logging, health checks, admin operations)
✅ **Phase 7 Complete** - Optional Extensions (embedded embeddings, plugin system, web UI)