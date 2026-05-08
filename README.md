![Graphyne Logo](graphyne.png)

# Graphyne 🚀

*A high-performance, agent-native search and knowledge graph engine*

[![Rust](https://img.shields.io/badge/Rust-1.75+-dea584?logo=rust)]()
[![License](https://img.shields.io/badge/License-Apache%202.0-blue)]()

---

## 🎯 What is Graphyne?

Graphyne is a **high-performance, agent-native search and knowledge graph engine** written in Rust. It's designed for modern AI applications that need:

- **Hybrid retrieval** (lexical + vector + graph)
- **Agent memory management** (working, episodic, semantic, procedural)
- **GraphRAG** for enhanced reasoning
- **Production-ready** observability

---

## ✨ Core Features

### 🔍 Hybrid Search Engine

| Feature | Description | Technology |
|---------|-------------|------------|
| **Lexical Search** | BM25 scoring with FST indexing | `fst` crate |
| **Vector Search** | Approximate Nearest Neighbor (ANN) | `hnsw-rs` |
| **Graph Search** | Typed property graph with traversal | `petgraph` |
| **Hybrid Scoring** | Weighted combination of all methods | Custom scorer |

**Example Search:**
```bash
cargo run --package graphyne-cli -- search --query "AI agents" --mode hybrid --limit 10
```

### 🧠 Agent Memory System

Graphyne treats agent memory as a first-class citizen:

| Memory Type | Purpose | Retention |
|------------|---------|-----------|
| **Working** | Short-term context | Session-based |
| **Episodic** | Event-based memories | Time-based |
| **Semantic** | Factual knowledge | Permanent |
| **Procedural** | How-to knowledge | Permanent |

**Memory Features:**
- ✅ Automatic token budgeting for LLM context
- ✅ Configurable retention policies
- ✅ Importance scoring
- ✅ Graph-based memory relationships

**Store Memory:**
```bash
cargo run --package graphyne-cli -- memory store --type semantic --content "Rust is memory-safe" --importance 0.9
```

### 🕸️ Knowledge Graph & GraphRAG

| Feature | Description |
|---------|-------------|
| **GraphRAG Engine** | Generate LLM context from subgraphs |
| **Multi-hop Traversal** | Find relationships across nodes |
| **Shortest Path** | Dijkstra's algorithm implementation |
| **Centrality Analysis** | Node importance ranking |
| **Recommendations** | Graph-based suggestions |

**GraphRAG Query:**
```bash
cargo run --package graphyne-cli -- rag query --query "machine learning" --hops 2
```

### 🌐 API & Interfaces

| Interface | Port | Protocol |
|-----------|------|----------|
| **gRPC** | 50051 | Protobuf via `tonic` |
| **HTTP/JSON** | 8080 | REST via `axum` |
| **CLI** | - | `clap`-based |
| **Web UI** | 8080/ | Simple SPA |

**HTTP API Examples:**
```bash
# Search
curl "http://localhost:8080/v1/search?q=hello&mode=hybrid"

# Store memory
curl -X POST http://localhost:8080/v1/memory/store -d '{"type":"working","content":"..."}'

# Health check
curl http://localhost:8080/health
```

### 📊 Observability & Control

| Feature | Endpoint | Format |
|---------|----------|--------|
| **Metrics** | `GET /metrics` | Prometheus |
| **Health** | `GET /health` | JSON |
| **Admin Stats** | `GET /admin/stats` | JSON |
| **Logging** | `stderr` | `tracing` |

**Available Admin Operations:**
- `POST /admin/flush` - Flush data to disk
- `POST /admin/backup` - Create backup
- `POST /admin/restore` - Restore from backup
- `POST /admin/compact` - Compact storage

### 🔌 Extensions (Optional)

| Extension | Feature Flag | Description |
|-----------|--------------|-------------|
| **Embeddings** | `embeddings` | Local embedding generation with `candle` |
| **Plugins** | `plugins` | Dynamic plugin system with hooks |
| **Web UI** | `web-ui` | Browser-based interface |

---

## 🚀 Quick Start

### Installation
```bash
git clone https://github.com/aymenbelarbi/graphyne.git
cd graphyne
cargo build --release
```

### Running the Server
```bash
cargo run --release --package graphyne-server
```

### Using the CLI
```bash
# Search
cargo run --package graphyne-cli -- search --query "your search"

# Store memory
cargo run --package graphyne-cli -- memory store --type working --content "Remember this"
```

---

## 📚 Documentation

- **Architecture Plan**: [`plans/comprehensive-architectural-plan.md`](plans/comprehensive-architectural-plan.md)
- **API Documentation**: Available at `http://localhost:8080/` when server is running
- **gRPC Proto**: [`graphyne-proto/proto/graphyne.proto`](graphyne-proto/proto/graphyne.proto)

---


---

## 🎨 Web UI (New!)

Graphyne now includes a modern web interface built with **React + Vite + shadcn/ui** components.

### Technology Stack

| Technology | Purpose |
|------------|---------|
| **React 18** | UI framework |
| **Vite** | Build tool & dev server |
| **TypeScript** | Type-safe development |
| **Tailwind CSS** | Utility-first styling |
| **shadcn/ui** | Reusable component library |

### Features

#### 🔍 Search Page
- Hybrid search with mode selection (hybrid, lexical, vector, graph)
- Real-time search results
- Responsive design with dark theme

#### 🧠 Memory Browser
- Store memories with type selection (working, episodic, semantic, procedural)
- View stored memories
- Configure retention policies and importance scoring

#### 📊 Dashboard
- System statistics at a glance
- Stat cards: Total Searches, Total Memories, Storage Size, Uptime
- Visual indicators and trends

#### ⚙️ Settings
- Server configuration
- API endpoint management
- Feature toggles

### Quick Start

#### Development Mode
```bash
cd graphyne-web
npm install
npm run dev
```
Access at: `http://localhost:5173`

#### Production Build
```bash
cd graphyne-web
npm run build
```

The build output will be in `graphyne-web/dist/`.

#### Integration with Graphyne Server

The server automatically serves the React build when available:

```bash
# Build the web UI
./build.sh

# Start the server (serves UI at http://localhost:8080)
cargo run --package graphyne-server
```

The server's `http.rs` is configured to serve static files from `graphyne-web/dist/`.

### Build Script

Use the provided `build.sh` script for easy deployment:

```bash
chmod +x build.sh
./build.sh              # Build web UI only
./build.sh --with-server  # Build web UI + Rust server
```

### UI Components

The UI is built with shadcn/ui components:
- **Cards** - Dashboard stat cards, content containers
- **Buttons** - Actions and submissions  
- **Inputs** - Text inputs with labels
- **Textarea** - Multi-line text input
- **Tabs** - Page navigation
- **Select** - Dropdown selections
- **Badge** - Status indicators
- **Label** - Form labels

All components are styled with Tailwind CSS using a dark gradient theme (`slate-900` to `purple-900`).

---

## 🏗️ Project Structure

```
graphyne/
├── graphyne-core/        # Core engine (search, memory, graph)
├── graphyne-server/      # API server (gRPC + HTTP)
├── graphyne-client/      # Client SDK
├── graphyne-cli/         # Command-line tool
├── graphyne-proto/       # Protobuf definitions
├── graphyne-web/         # Web UI (optional)
└── README.md
```

---

## 📄 License

Licensed under the Apache License 2.0 - see [LICENSE](LICENSE) for details.

---

## 🌟 GitHub

[![GitHub stars](https://img.shields.io/github/stars/aymenbelarbi/graphyne?style=social)](https://github.com/aymenbelarbi/graphyne)
[![GitHub forks](https://img.shields.io/github/forks/aymenbelarbi/graphyne?style=social)](https://github.com/aymenbelarbi/graphyne)
