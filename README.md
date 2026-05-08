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
├── graphyne-server/        # Server binary (skeleton)
│   ├── src/
│   │   └── main.rs        # Basic startup message
│   └── Cargo.toml
├── graphyne-client/        # Client SDK and CLI (planned)
├── graphyne-cli/           # CLI tool (planned)
├── graphyne-proto/         # gRPC/protobuf schemas (planned)
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

## Getting Started

### Prerequisites

- Rust (edition 2021 or later)
- Cargo package manager

### Build Instructions

```bash
# Clone the repository
git clone https://github.com/yourusername/graphyne.git
cd graphyne/graphyne

# Build the workspace
cargo build

# Run the server (once implemented)
cargo run -p graphyne-server
```

### Current Status

**Phase 1** (Complete):
- ✅ Cargo workspace initialization
- ✅ graphyne-core crate with error handling (`GraphyneError`)
- ✅ Storage abstraction traits (`StorageBackend`)
- ✅ graphyne-server skeleton binary

**Phase 2** (In Progress):
- ✅ Lexical search engine with BM25 scoring and FST indexing
- ✅ Vector search engine with HNSW (hnsw-rs) for ANN
- ✅ Graph storage with petgraph for typed property graph
- ✅ Hybrid scoring to combine retrieval results
- ✅ Unit tests for core components

## License

Apache License 2.0

## Status

✅ **Phase 1 Complete** - Foundations implemented (Cargo workspace, core crate, server skeleton)  
🔄 **Phase 2 In Progress** - Core retrieval engines (lexical, vector, graph, hybrid scoring)
