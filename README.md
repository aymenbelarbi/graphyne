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
├── graphyne-core/      # Core engine (query, scoring, storage)
├── graphyne-server/    # API binary, config, runtime
├── graphyne-client/    # Client SDK and CLI
├── graphyne-proto/     # gRPC/protobuf schemas
├── graphyne-mcp/       # MCP integration for agent tooling
└── README.md
```

## Getting Started

[To be filled as implementation progresses]

## License

Apache License 2.0

## Status

🚧 **Under Active Development** - Phase 1: Project Scaffolding
