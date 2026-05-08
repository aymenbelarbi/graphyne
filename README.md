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
│   │   ├── lib.rs         # Library root
│   │   ├── error.rs       # Error types (GraphyneError)
│   │   └── storage/       # Storage abstraction layer
│   │       └── mod.rs     # StorageBackend trait
│   └── Cargo.toml
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

Phase 1 implementation includes:
- ✅ Cargo workspace initialization
- ✅ graphyne-core crate with error handling (`GraphyneError`)
- ✅ Storage abstraction traits (`StorageBackend`)
- ✅ graphyne-server skeleton binary

## License

Apache License 2.0

## Status

✅ **Phase 1 Complete** - Foundations implemented (Cargo workspace, core crate, server skeleton)  
🚧 **Next**: Phase 2 - Core implementation (storage backends, query engine)
