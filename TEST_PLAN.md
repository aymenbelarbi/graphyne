# Graphyne Test Infrastructure Improvement Plan

## 1. Current State Summary

### Existing Tests (32 unit tests, all passing)

| Module | File | Test Count | What's Tested |
|--------|------|------------|---------------|
| `memory/context.rs` | `graphyne-core/src/memory/context.rs` | 3 | Token estimation, packing within/over budget |
| `memory/retention.rs` | `graphyne-core/src/memory/retention.rs` | 3 | Importance threshold, age expiry, max-entries prune |
| `memory/scoring.rs` | `graphyne-core/src/memory/scoring.rs` | 4 | Combined score, recency, access frequency, weight normalization |
| `lexical/mod.rs` | `graphyne-core/src/lexical/mod.rs` | 3 | Tokenize, push+search, prefix search |
| `vector/mod.rs` | `graphyne-core/src/vector/mod.rs` | 4 | Add+search, dimension mismatch, cosine similarity, get embedding |
| `graph/mod.rs` | `graphyne-core/src/graph/mod.rs` | 6 | Add/get node, add edge, traverse, find by type, shortest path, centrality |
| `graph/rag.rs` | `graphyne-core/src/graph/rag.rs` | 1 | Empty query returns zero confidence |
| `embeddings/mod.rs` | `graphyne-core/src/embeddings/mod.rs` | 2 | Fallback embedding generation, batch generation |
| `health/mod.rs` | `graphyne-core/src/health/mod.rs` | 2 | Health checker creation, uptime measurement |
| `admin/mod.rs` | `graphyne-core/src/admin/mod.rs` | 1 | Admin stats retrieval |
| `scoring/mod.rs` | `graphyne-core/src/scoring/mod.rs` | 4 | Config validation, combine results, empty combine, normalize/invert scores |

### What Has NO Tests

- **graphyne-server**: Zero tests for HTTP handlers, gRPC services, or server startup
- **graphyne-client**: Zero tests for HTTP client, gRPC client, or error handling
- **graphyne-cli**: Zero tests for any CLI command parsing or execution
- **graphyne-proto**: Zero tests for protobuf serialization/deserialization
- **graphyne-web**: Zero tests — no test framework configured (no vitest/jest)
- **Integration tests**: None — no `tests/` directory at the workspace root or in any crate
- **E2E tests**: None
- **CI/CD**: No GitHub Actions or other CI pipeline
- **Coverage**: No coverage tooling configured
- **Benchmarks**: No performance benchmarks

---

## 2. Coverage Gap Analysis

### Critical Gaps (High Risk)

| Gap | Impact | Effort |
|-----|--------|--------|
| HTTP handler `store_memory_handler` — parses `MemoryType` from string, constructs `MemoryEntry`, calls `MemoryStore::store_memory` | **HIGH** — core memory write path is untested end-to-end | Medium |
| HTTP handler `recall_memory_handler` — builds `MemoryQuery`, calls `store.recall()` or `store.list_memories()` | **HIGH** — core memory read path is untested | Medium |
| HTTP handler `update_memory_handler` / `delete_memory_handler` | **HIGH** — mutation paths untested | Medium |
| `MemoryStore::store_memory` — integrates sled + lexical + vector + graph | **HIGH** — no integration test verifying all 4 subsystems are populated | Medium |
| `MemoryStore::recall` — multi-stage query with filters + scoring | **HIGH** — no test with real data flowing through lexical → vector → score pipeline | High |
| `MemoryStore::delete_memory` — removes from sled + graph but NOT lexical/vector | **MEDIUM** — potential stale-index bug, no test verifies behavior | Low |
| `MemoryStore::apply_retention` — prunes entries by policy | **MEDIUM** — no test with real data | Medium |
| `GraphRAG::query` — only tested with empty graph | **MEDIUM** — no test with seeded data + traversal + context generation | High |
| `GraphStore::shortest_path` — tested but only with 3-node chain | **MEDIUM** — no test with disconnected nodes, cycles, or weighted edges | Medium |
| `LexicalIndex::search` — BM25 scoring accuracy | **MEDIUM** — only tests that results are non-empty, not score correctness | Medium |
| `VectorIndex::search` — HNSW accuracy | **MEDIUM** — only tests that results are non-empty, not nearest-neighbor correctness | Medium |

### Important Gaps (Medium Risk)

| Gap | Impact | Effort |
|-----|--------|--------|
| `ContextPacker` with empty memories, single memory, exact boundary | **MEDIUM** — edge cases in token budget management | Low |
| `RetentionPolicy::is_expired` — never called in tests | **MEDIUM** — dead code or untested path | Low |
| `MemoryEntry::access()` — never directly tested | **LOW** — simple but should be covered | Low |
| `MemoryStore::create_space` — duplicate space error path | **MEDIUM** — error handling untested | Low |
| `MemoryStore::store_memory_with_graph` — never tested | **MEDIUM** — graph relationship creation untested | Medium |
| `MemoryStore::recall_with_graph` — never tested | **MEDIUM** — GraphRAG integration path untested | Medium |
| `GraphStore::recommend_nodes` — never tested | **MEDIUM** — recommendation algorithm untested | Medium |
| `GraphStore::get_neighbors` — never tested | **LOW** — basic graph operation | Low |
| `HybridScorer` with min_score threshold | **MEDIUM** — filtering behavior untested | Low |
| `AdminService::backup` / `restore` / `compact` / `flush` | **MEDIUM** — admin operations untested | Medium |
| `HealthChecker` with invalid storage path | **MEDIUM** — error path untested | Low |
| `PluginManager` — only registration/unload tested; `load_plugin`, `load_all_plugins`, hooks untested | **MEDIUM** — dynamic loading not testable, but hook chains are | Medium |
| `EmbeddingGenerator` with empty string, very long text | **LOW** — edge cases | Low |
| `LexicalIndex` with empty collection/bucket/doc_id (error paths) | **MEDIUM** — input validation untested | Low |
| `VectorIndex::remove_embedding` + `needs_rebuild` + `maybe_rebuild` | **MEDIUM** — rebuild cycle untested | Medium |

### Lower Priority Gaps

| Gap | Impact | Effort |
|-----|--------|--------|
| `StorageBackend` trait — no mock implementation tests | **LOW** — trait is defined but never implemented outside sled | Low |
| `LexicalIndex::get_terms` — never tested | **LOW** — utility function | Low |
| `GraphStore::find_edges_by_type` — never tested | **LOW** — utility function | Low |
| `GraphStore::iter_nodes` — never tested | **LOW** — utility function | Low |
| `GraphyneError` variants — no test for `Display` formatting | **LOW** — error message quality | Low |
| `ClientConfig::default` — trivial but untested | **LOW** | Low |

---

## 3. Phased Improvement Plan

### Phase 1: Unit Test Improvements (Quick Wins)

**Goal**: Cover edge cases, error paths, and boundary conditions in existing modules. No new files needed — add tests to existing `#[cfg(test)]` modules.

#### 1.1 Memory Module Enhancements

**File**: `graphyne-core/src/memory/store.rs`

Add integration-style unit tests that use a real `TempDir` + `sled::Db` to exercise the full `MemoryStore`:

```rust
// New tests to add:
#[cfg(test)]
mod store_integration_tests {
    // test_store_memory_populates_all_indices — verify entry appears in sled, lexical, vector, graph
    // test_store_memory_with_invalid_space — error path
    // test_store_memory_with_graph — verify edges created
    // test_get_memory_updates_access_count — verify access_count increments
    // test_get_memory_nonexistent — returns None
    // test_update_memory_content — verify re-indexing in lexical
    // test_update_memory_not_found — error path
    // test_delete_memory_removes_from_sled_and_graph
    // test_delete_memory_not_found — error path
    // test_recall_with_lexical_query — store text, search text, verify results
    // test_recall_with_vector_query — store embedding, search embedding, verify results
    // test_recall_with_type_filter — store mixed types, filter by type
    // test_recall_with_importance_filter — store mixed importance, filter
    // test_recall_with_time_range — store with different timestamps, filter
    // test_recall_combined_lexical_vector — verify score merging
    // test_recall_empty_store — returns empty results
    // test_list_memories_with_limit
    // test_create_space_duplicate — error path
    // test_apply_retention_prunes_low_importance
    // test_apply_retention_prunes_by_age
    // test_apply_retention_prunes_by_max_entries
    // test_recall_with_graph_when_uninitialized — error path
}
```

**File**: `graphyne-core/src/memory/context.rs`

```rust
// New tests:
// test_pack_empty_memories — returns empty string
// test_pack_single_memory — single entry fits
// test_pack_exact_budget_boundary — memory fits exactly
// test_pack_truncation_head — verify head truncation
// test_pack_truncation_tail — verify tail truncation
// test_pack_truncation_middle — verify middle truncation
// test_pack_truncation_summarize — verify summarize placeholder
// test_estimate_tokens_empty — empty string
// test_estimate_tokens_unicode — non-ASCII content
// test_set_max_tokens — mutator
// test_set_chars_per_token — mutator with clamping
```

**File**: `graphyne-core/src/memory/retention.rs`

```rust
// New tests:
// test_is_expired_within_threshold — not expired
// test_is_expired_beyond_threshold — expired
// test_is_expired_no_max_age — never expired when max_age is None
// test_should_retain_no_criteria — retains everything when all None
// test_prune_empty_entries — no panic on empty vec
// test_prune_preserves_order — verify sort stability
```

**File**: `graphyne-core/src/memory/scoring.rs`

```rust
// New tests:
// test_score_with_zero_relevance — boundary
// test_score_with_max_relevance — boundary
// test_score_old_memory_heavy_access — old but frequently accessed
// test_score_new_memory_no_access — new but never accessed
// test_compute_recency_exact_boundaries — 1h, 24h, 7d, 30d boundaries
// test_compute_access_frequency_zero — returns 0.0
// test_compute_access_frequency_one — returns 0.5
// test_update_weights_partial — only update some weights
// test_update_weights_zero_sum — edge case
```

#### 1.2 Graph Module Enhancements

**File**: `graphyne-core/src/graph/mod.rs`

```rust
// New tests:
// test_shortest_path_no_path — disconnected nodes
// test_shortest_path_same_node — from == to
// test_shortest_path_cycle — graph with cycles
// test_shortest_path_weighted — verify Dijkstra picks lowest weight
// test_remove_node_cascading — verify connected edges are removed
// test_remove_node_not_found — error path
// test_add_edge_missing_from — error path
// test_add_edge_missing_to — error path
// test_remove_edge_not_found — error path
// test_traverse_zero_hops — only start node
// test_traverse_disconnected — no neighbors
// test_traverse_cycle — graph with cycles
// test_recommend_nodes — verify scoring
// test_recommend_nodes_disconnected — no recommendations
// test_get_neighbors — verify neighbor list
// test_get_neighbors_not_found — error path
// test_find_edges_by_type
// test_iter_nodes
// test_node_count_empty
// test_edge_count_empty
```

#### 1.3 Lexical Module Enhancements

**File**: `graphyne-core/src/lexical/mod.rs`

```rust
// New tests:
// test_push_empty_collection — error path
// test_push_empty_bucket — error path
// test_push_empty_doc_id — error path
// test_search_empty_query — returns empty
// test_search_no_results — valid query, no matches
// test_search_case_insensitive — verify case folding
// test_search_multiple_terms — AND behavior
// test_search_relevance_ranking — verify BM25 ordering
// test_tokenize_empty — empty string
// test_tokenize_unicode — non-ASCII
// test_tokenize_single_char — filtered out
// test_tokenize_stop_words — single-char words removed
// test_prefix_search_no_match
// test_prefix_search_case_insensitive
// test_get_terms_empty_collection
// test_update_fst_multiple_pushes — verify FST grows correctly
```

#### 1.4 Vector Module Enhancements

**File**: `graphyne-core/src/vector/mod.rs`

```rust
// New tests:
// test_add_embedding_wrong_dimension — error path
// test_search_empty_index — error path (HNSW not initialized)
// test_search_accuracy — verify nearest neighbor is correct
// test_search_with_k_larger_than_n — request more than available
// test_remove_embedding — verify removal + rebuild flag
// test_remove_embedding_not_found — no error, just no-op
// test_maybe_rebuild — verify HNSW rebuilt after removal
// test_cosine_similarity_orthogonal — 0.0
// test_cosine_similarity_opposite — negative
// test_cosine_similarity_zero_vector — edge case
// test_get_embedding_nonexistent — returns None
// test_dimension_mismatch_on_search — query dim != index dim
```

#### 1.5 Other Module Enhancements

**File**: `graphyne-core/src/graph/rag.rs`

```rust
// New tests:
// test_graphrag_with_seeded_data — add nodes + edges, query, verify context
// test_graphrag_expand_node — verify neighbor retrieval
// test_graphrag_get_subgraph — verify subgraph extraction
// test_graphrag_empty_label_fallback — label match fallback
// test_graphrag_confidence_scoring — verify confidence increases with more data
// test_graphrag_explanation_format — verify explanation string
```

**File**: `graphyne-core/src/embeddings/mod.rs`

```rust
// New tests:
// test_fallback_embedding_empty_string
// test_fallback_embedding_long_text — 10KB+ text
// test_fallback_embedding_deterministic — same input → same output
// test_fallback_embedding_different_inputs — different inputs → different outputs
// test_fallback_embedding_normalized — unit length
```

**File**: `graphyne-core/src/health/mod.rs`

```rust
// New tests:
// test_health_with_valid_storage_path
// test_health_with_invalid_storage_path — unhealthy
// test_health_with_unreadable_path — permission error
// test_health_status_degraded — mixed healthy/unhealthy checks
```

**File**: `graphyne-core/src/admin/mod.rs`

```rust
// New tests:
// test_backup_success
// test_backup_creates_directory
// test_restore_success
// test_restore_nonexistent_path — error path
// test_compact_success
// test_flush_success
// test_calculate_directory_size_empty
// test_calculate_directory_size_nested
```

**File**: `graphyne-core/src/scoring/mod.rs`

```rust
// New tests:
// test_combine_with_min_score_threshold
// test_combine_lexical_vector_only
// test_combine_lexical_graph_only
// test_combine_vector_graph_only
// test_normalize_scores_identical — all same score
// test_normalize_scores_single — single element
// test_invert_scores_zero — zero distance
// test_set_config_invalid — error path
```

**File**: `graphyne-core/src/plugins/mod.rs`

```rust
// New tests:
// test_pre_search_all — verify hook chain
// test_post_search_all — verify hook chain
// test_load_plugin_not_found — error path
// test_unload_plugin_not_found — error path
// test_load_all_plugins_empty_dir
// test_is_plugin_file — verify extension detection
// test_extract_plugin_name
```

---

### Phase 2: Integration Tests

**Goal**: Test HTTP API endpoints, gRPC services, and full request/response flows with a real server and database.

#### 2.1 HTTP API Integration Tests

**New file**: `graphyne-server/tests/http_api_tests.rs`

```rust
// Test infrastructure:
// - Use tempfile for sled DB
// - Start axum server on random port using tokio::net::TcpListener
// - Use reqwest for HTTP client calls
// - Helper functions: setup_test_server(), teardown()

// Endpoints to test:
// GET  /health                    → 200, {"status":"ok"}
// GET  /metrics                   → 200, Prometheus format
// GET  /admin/stats               → 200, stats JSON
// GET  /admin/health              → 200, health JSON
// GET  /admin/health?detailed=true → 200, with checks array
// POST /admin/flush               → 200, success
// POST /admin/backup              → 200, creates backup dir
// POST /admin/restore             → 200/404 based on path
// POST /admin/compact             → 200, success
// GET  /v1/search                 → 200, results array
// POST /v1/documents              → 200, success
// POST /v1/vectors               → 200, success
// POST /v1/graph/nodes            → 200, success
// POST /v1/graph/edges            → 200, success
// POST /v1/memory/store           → 200, returns memory_id
// GET  /v1/memory/recall          → 200, returns memories array
// GET  /v1/memory/spaces          → 200, returns spaces array
// POST /v1/memory/:id             → 200, success
// DELETE /v1/memory/:id           → 200, success
// POST /v1/graph/rag/query        → 200, returns context
// POST /v1/graph/subgraph         → 200, returns nodes/edges
// POST /v1/graph/expand/:id       → 200, returns nodes/edges

// Error cases:
// POST /v1/memory/store with invalid memory_type → error JSON
// POST /v1/memory/store with missing content → error JSON
// POST /v1/memory/nonexistent → 404 or error JSON
// DELETE /v1/memory/nonexistent → error JSON
// POST /v1/graph/edges with missing node → error JSON
```

#### 2.2 Memory Store Integration Tests

**New file**: `graphyne-core/tests/memory_integration.rs`

```rust
// Full lifecycle tests:
// test_memory_store_recall_roundtrip — store N memories, recall, verify all returned
// test_memory_store_recall_by_type — store mixed types, filter by type
// test_memory_store_recall_by_importance — store mixed importance, filter
// test_memory_store_update_and_recall — update content, verify re-indexed
// test_memory_store_delete_and_recall — delete, verify not in results
// test_memory_store_with_embeddings — store with embedding, search by vector
// test_memory_store_graph_integration — verify graph nodes created for memories
// test_memory_store_retention_integration — apply retention, verify pruning
// test_memory_store_multiple_spaces — create spaces, store in different spaces
// test_memory_store_concurrent_access — multi-threaded store/recall
```

#### 2.3 Graph Integration Tests

**New file**: `graphyne-core/tests/graph_integration.rs`

```rust
// Complex graph scenarios:
// test_graph_large_dataset — 100+ nodes, verify traversal performance
// test_graph_dense_connections — fully connected subgraph
// test_graph_disconnected_components — multiple disconnected subgraphs
// test_graph_cyclic — cycles don't cause infinite loops
// test_graph_shortest_path_complex — multiple paths, verify optimal
// test_graph_recommendations_accuracy — verify recommendation scoring
// test_graph_persistence — save, reload from sled, verify integrity
```

#### 2.4 gRPC Integration Tests

**New file**: `graphyne-server/tests/grpc_api_tests.rs`

```rust
// Test gRPC service implementations:
// test_grpc_search — SearchService::search
// test_grpc_suggest — SearchService::suggest
// test_grpc_push_document — DocumentService::push_document
// test_grpc_pop_document — DocumentService::pop_document
// test_grpc_get_document — DocumentService::get_document
// test_grpc_add_embedding — VectorService::add_embedding
// test_grpc_search_vector — VectorService::search_vector
// test_grpc_add_node — GraphService::add_node
// test_grpc_add_edge — GraphService::add_edge
// test_grpc_traverse — GraphService::traverse_graph
// test_grpc_store_memory — MemoryService::store_memory
// test_grpc_recall_memory — MemoryService::recall_memory
// test_grpc_graphrag_query — GraphRAGService::query
// test_grpc_admin_stats — AdminService::get_stats
// test_grpc_admin_health — AdminService::get_health
```

#### 2.5 Client Integration Tests

**New file**: `graphyne-client/tests/client_integration.rs`

```rust
// Requires running server instance:
// test_http_client_search — full request/response cycle
// test_http_client_push_document
// test_http_client_add_embedding
// test_http_client_add_node
// test_grpc_client_search — full gRPC cycle
// test_grpc_client_push_document
// test_grpc_client_store_memory
// test_grpc_client_recall_memory
// test_client_error_handling — server unreachable, timeout
// test_client_config_default
// test_client_config_custom_endpoint
```

---

### Phase 3: End-to-End Tests (Web App)

**Goal**: Test the React web application UI flows using Playwright.

#### 3.1 Test Setup

**New files**:
- `graphyne-web/vitest.config.ts` — unit test configuration
- `graphyne-web/tests/e2e/playwright.config.ts` — E2E test configuration
- `graphyne-web/tests/e2e/helpers.ts` — test utilities

**Dependencies to add** (`graphyne-web/package.json`):
```json
{
  "devDependencies": {
    "@playwright/test": "^1.40.0",
    "@testing-library/react": "^14.0.0",
    "@testing-library/jest-dom": "^6.0.0",
    "@testing-library/user-event": "^14.0.0",
    "@vitejs/plugin-react": "^6.0.0",
    "jsdom": "^23.0.0",
    "vitest": "^1.0.0"
  },
  "scripts": {
    "test": "vitest run",
    "test:watch": "vitest",
    "test:e2e": "playwright test",
    "test:e2e:ui": "playwright test --ui"
  }
}
```

#### 3.2 Web App Unit Tests

**New file**: `graphyne-web/src/__tests__/utils.test.ts`

```typescript
// Test cn() utility function
// test_cn_with_empty_inputs
// test_cn_with_conflicting_classes — twMerge resolution
// test_cn_with_conditional_classes
```

**New file**: `graphyne-web/src/pages/__tests__/DashboardPage.test.tsx`

```typescript
// Test DashboardPage component:
// test_dashboard_loading_state — shows loading spinner
// test_dashboard_error_state — shows error message when server unreachable
// test_dashboard_stats_display — renders stats from API
// test_dashboard_auto_refresh — verifies 30s interval
// test_dashboard_format_bytes — 0 B, KB, MB, GB
// test_dashboard_format_uptime — seconds, minutes, hours
```

**New file**: `graphyne-web/src/pages/__tests__/SearchPage.test.tsx`

```typescript
// Test SearchPage component:
// test_search_empty_query — button disabled
// test_search_with_query — triggers API call
// test_search_results_display — renders results
// test_search_error_state — shows error on failure
// test_search_mode_switching — changes search mode
// test_search_no_results — shows empty state
// test_search_keyboard_submit — Enter key triggers search
```

**New file**: `graphyne-web/src/pages/__tests__/MemoryPage.test.tsx`

```typescript
// Test MemoryPage component:
// test_memory_store_form — fill and submit
// test_memory_type_select — change memory type
// test_memory_importance_select — change importance
// test_memory_list_display — renders stored memories
// test_memory_delete — click delete, verify removal
// test_memory_empty_state — shows empty state
// test_memory_error_handling — server error
// test_memory_success_message — shows success after store
```

**New file**: `graphyne-web/src/pages/__tests__/SettingsPage.test.tsx`

```typescript
// Test SettingsPage component:
// test_settings_server_url_input
// test_settings_save_button — saves to localStorage
// test_settings_saved_feedback — shows "Saved!" temporarily
// test_settings_load_saved_url — loads from localStorage on mount
```

**New file**: `graphyne-web/src/components/ui/__tests__/button.test.tsx`

```typescript
// Test Button component variants:
// test_button_default_render
// test_button_disabled_state
// test_button_with_icon
// test_button_click_handler
// test_button_variants — default, destructive, outline, secondary, ghost, link
// test_button_sizes — default, sm, lg, icon
```

#### 3.3 E2E Tests with Playwright

**New file**: `graphyne-web/tests/e2e/dashboard.spec.ts`

```typescript
// E2E test: Dashboard page
// test_dashboard_loads — page loads with stats cards
// test_dashboard_shows_error_when_server_down
// test_dashboard_navigation — click nav items, verify page changes
// test_dashboard_responsive — mobile viewport
```

**New file**: `graphyne-web/tests/e2e/search.spec.ts`

```typescript
// E2E test: Search page
// test_search_flow — enter query, click search, see results
// test_search_mode_switch — change mode, verify description updates
// test_search_empty_state — search with no results
```

**New file**: `graphyne-web/tests/e2e/memory.spec.ts`

```typescript
// E2E test: Memory page
// test_store_memory_flow — fill form, submit, verify in list
// test_delete_memory_flow — store, delete, verify removed
// test_memory_type_selection — change type, verify badge
// test_importance_selection — change importance
```

**New file**: `graphyne-web/tests/e2e/settings.spec.ts`

```typescript
// E2E test: Settings page
// test_settings_save — change URL, save, reload, verify persisted
// test_settings_about_section — verify about info displayed
```

**New file**: `graphyne-web/tests/e2e/navigation.spec.ts`

```typescript
// E2E test: Navigation
// test_sidebar_navigation — click each nav item
// test_mobile_sidebar_toggle — open/close on mobile
// test_active_state_highlighting — active tab is highlighted
// test_breadcrumb_updates — breadcrumb matches active page
```

---

### Phase 4: CI/CD Pipeline

**Goal**: Automated test runs on every push and PR.

#### 4.1 GitHub Actions Workflow

**New file**: `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # Job 1: Rust lint + unit tests
  rust-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Clippy lint
        run: cargo clippy --workspace --all-targets -- -D warnings

      - name: Run unit tests
        run: cargo test --workspace --lib --bins

      - name: Run integration tests
        run: cargo test --workspace --tests

  # Job 2: Build check (all crates compile)
  rust-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Build all crates
        run: cargo build --workspace

      - name: Build release
        run: cargo build --workspace --release

  # Job 3: Web app tests
  web-test:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: graphyne-web
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: graphyne-web/package-lock.json

      - name: Install dependencies
        run: npm ci

      - name: Type check
        run: npx tsc --noEmit

      - name: Lint
        run: npm run lint

      - name: Unit tests
        run: npm run test

      - name: Build
        run: npm run build

  # Job 4: E2E tests (requires running server)
  e2e-test:
    runs-on: ubuntu-latest
    needs: [rust-build]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Build server
        run: cargo build --bin graphyne-server

      - name: Install Playwright
        working-directory: graphyne-web
        run: |
          npm ci
          npx playwright install --with-deps chromium

      - name: Start server and run E2E tests
        working-directory: graphyne-web
        run: |
          cargo run --bin graphyne-server &
          sleep 3
          npx playwright test

  # Job 5: Coverage report
  coverage:
    runs-on: ubuntu-latest
    needs: [rust-test]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@cargo-tarpaulin

      - name: Generate coverage report
        run: cargo tarpaulin --workspace --out Xml --out Html

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          file: ./cobertura.xml
```

#### 4.2 Additional CI Workflows

**New file**: `.github/workflows/release.yml` — Build and publish releases on tag push

**New file**: `.github/workflows/security-audit.yml` — Run `cargo audit` weekly

---

### Phase 5: Test Data Management

**Goal**: Consistent, reusable test fixtures and factories.

#### 5.1 Rust Test Utilities

**New file**: `graphyne-core/src/test_utils.rs` (or `graphyne-core/tests/common/mod.rs`)

```rust
// Test utilities module:
pub mod fixtures {
    // fn temp_db() -> (sled::Db, TempDir) — create temp sled database
    // fn test_memory_entry() -> MemoryEntry — create a default MemoryEntry
    // fn test_memory_entry_with_type(MemoryType) -> MemoryEntry
    // fn test_graph_node(id: &str) -> GraphNode
    // fn test_graph_edge(from: &str, to: &str) -> GraphEdge
    // fn test_embedding(dim: usize) -> Vec<f32> — random normalized embedding
    // fn seeded_memory_store(n: usize) -> (MemoryStore, TempDir) — store with N entries
    // fn seeded_graph_store(n_nodes: usize, n_edges: usize) -> (GraphStore, TempDir)
}

pub mod assertions {
    // fn assert_memory_in_store(store: &MemoryStore, id: &str)
    // fn assert_memory_not_in_store(store: &MemoryStore, id: &str)
    // fn assert_graph_node_exists(store: &GraphStore, id: &str)
    // fn assert_graph_edge_exists(store: &GraphStore, from: &str, to: &str)
}
```

#### 5.2 Web Test Utilities

**New file**: `graphyne-web/tests/e2e/helpers.ts`

```typescript
// E2E test helpers:
// export const mockServerHealth() — mock /health endpoint
// export const mockServerStats() — mock /admin/stats endpoint
// export const mockMemoryStore() — mock /v1/memory/store endpoint
// export const mockMemoryRecall() — mock /v1/memory/recall endpoint
// export const waitForServerReady() — poll /health until 200
```

**New file**: `graphyne-web/src/__tests__/test-utils.tsx`

```typescript
// React Testing Library utilities:
// export function renderWithProviders(ui: ReactElement) — wrap with necessary providers
// export function mockFetchResponse(data: unknown) — mock global.fetch
// export function mockFetchError(message: string) — mock fetch failure
```

---

### Phase 6: Coverage Reporting

**Goal**: Track and improve code coverage over time.

#### 6.1 Rust Coverage

**Tool**: `cargo-tarpaulin` (already referenced in CI)

**Configuration**: `.tarpaulin.toml`

```toml
[default]
exclude-files = ["src/main.rs", "src/cli/*", "tests/*"]
output-dir = "target/tarpaulin"
out = ["Html", "Xml", "Json"]
timeout = 120

[default.features]
```

**Local usage**:
```bash
# Generate HTML report
cargo tarpaulin --workspace --out Html
open target/tarpaulin/tarpaulin-report.html

# Generate JSON for CI
cargo tarpaulin --workspace --out Xml
```

#### 6.2 Web Coverage

**Tool**: Vitest built-in coverage (via `@vitest/coverage-v8`)

**Configuration** (`graphyne-web/vitest.config.ts`):
```typescript
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/__tests__/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'json', 'lcov'],
      exclude: ['node_modules/', 'tests/', 'src/__tests__/'],
      thresholds: {
        branches: 70,
        functions: 70,
        lines: 70,
        statements: 70,
      },
    },
  },
})
```

**Local usage**:
```bash
cd graphyne-web
npm run test -- --coverage
```

---

### Phase 7: Performance Benchmarks

**Goal**: Track performance regressions and establish baselines.

#### 7.1 Rust Benchmarks

**New file**: `graphyne-core/benches/memory_bench.rs`

```rust
use criterion::{criterion_group, criterion_benchmark, Criterion};

fn bench_memory_store(c: &mut Criterion) {
    // bench_store_single_memory — store one memory
    // bench_store_100_memories — batch store
    // bench_recall_100_memories — recall from 100 entries
    // bench_recall_1000_memories — recall from 1000 entries
    // bench_recall_with_embedding — vector search performance
}

fn bench_graph_operations(c: &mut Criterion) {
    // bench_add_node — single node insertion
    // bench_add_edge — single edge insertion
    // bench_traverse_2_hops — 2-hop traversal
    // bench_shortest_path — Dijkstra performance
    // bench_recommend — recommendation scoring
}

fn bench_lexical_search(c: &mut Criterion) {
    // bench_index_document — push_text performance
    // bench_search_100_docs — search latency
    // bench_search_1000_docs — search latency
}

fn bench_vector_search(c: &mut Criterion) {
    // bench_add_embedding — single insertion
    // bench_search_100_vectors — ANN search latency
    // bench_search_1000_vectors — ANN search latency
}

criterion_group!(benches, bench_memory_store, bench_graph_operations, bench_lexical_search, bench_vector_search);
criterion_main!(benches);
```

**Add to `graphyne-core/Cargo.toml`**:
```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "memory_bench"
harness = false
```

#### 7.2 Web Performance Tests

**New file**: `graphyne-web/tests/performance/lighthouse.spec.ts`

```typescript
// Lighthouse CI tests for web app performance:
// test_dashboard_lighthouse_score — performance, accessibility, SEO
// test_search_lighthouse_score
// test_memory_lighthouse_score
```

**New file**: `.github/workflows/lighthouse.yml` — Lighthouse CI on PR

---

## 4. Priority Matrix

### High Impact + Low Effort (Do First)

| # | Task | File(s) | Est. Tests |
|---|------|---------|------------|
| 1 | Add error path tests to existing unit tests | `graphyne-core/src/*/mod.rs` | ~30 |
| 2 | Add boundary/edge case tests to existing unit tests | `graphyne-core/src/*/mod.rs` | ~20 |
| 3 | Set up web test framework (Vitest) | `graphyne-web/vitest.config.ts` | — |
| 4 | Write web component unit tests | `graphyne-web/src/**/__tests__/*.test.tsx` | ~25 |
| 5 | Create CI workflow for Rust tests | `.github/workflows/ci.yml` | — |

### High Impact + Medium Effort (Do Second)

| # | Task | File(s) | Est. Tests |
|---|------|---------|------------|
| 6 | Memory store integration tests | `graphyne-core/tests/memory_integration.rs` | ~15 |
| 7 | Graph integration tests | `graphyne-core/tests/graph_integration.rs` | ~10 |
| 8 | HTTP API integration tests | `graphyne-server/tests/http_api_tests.rs` | ~25 |
| 9 | Test data utilities module | `graphyne-core/src/test_utils.rs` | — |
| 10 | CI workflow for web tests | `.github/workflows/ci.yml` (web-test job) | — |

### High Impact + High Effort (Do Third)

| # | Task | File(s) | Est. Tests |
|---|------|---------|------------|
| 11 | gRPC integration tests | `graphyne-server/tests/grpc_api_tests.rs` | ~14 |
| 12 | Client integration tests | `graphyne-client/tests/client_integration.rs` | ~10 |
| 13 | E2E tests with Playwright | `graphyne-web/tests/e2e/*.spec.ts` | ~15 |
| 14 | Coverage reporting setup | `.tarpaulin.toml`, `vitest.config.ts` | — |
| 15 | Performance benchmarks | `graphyne-core/benches/memory_bench.rs` | — |

### Medium Impact (Do When Time Permits)

| # | Task | File(s) | Est. Tests |
|---|------|---------|------------|
| 16 | CLI integration tests | `graphyne-cli/tests/` | ~10 |
| 17 | Proto serialization tests | `graphyne-proto/tests/` | ~5 |
| 18 | Web performance (Lighthouse) | `graphyne-web/tests/performance/` | — |
| 19 | Security audit CI | `.github/workflows/security-audit.yml` | — |
| 20 | Release automation | `.github/workflows/release.yml` | — |

---

## 5. File Checklist

### New Files to Create

```
graphyne-core/src/test_utils.rs                    -- Test utilities and fixtures
graphyne-core/tests/memory_integration.rs          -- Memory store integration tests
graphyne-core/tests/graph_integration.rs           -- Graph integration tests
graphyne-core/benches/memory_bench.rs              -- Performance benchmarks
graphyne-server/tests/http_api_tests.rs            -- HTTP API integration tests
graphyne-server/tests/grpc_api_tests.rs           -- gRPC integration tests
graphyne-client/tests/client_integration.rs        -- Client integration tests
graphyne-web/vitest.config.ts                      -- Vitest configuration
graphyne-web/src/__tests__/setup.ts                -- Test setup (jest-dom import)
graphyne-web/src/__tests__/test-utils.tsx          -- React testing utilities
graphyne-web/src/__tests__/utils.test.ts           -- cn() utility tests
graphyne-web/src/pages/__tests__/DashboardPage.test.tsx
graphyne-web/src/pages/__tests__/SearchPage.test.tsx
graphyne-web/src/pages/__tests__/MemoryPage.test.tsx
graphyne-web/src/pages/__tests__/SettingsPage.test.tsx
graphyne-web/src/components/ui/__tests__/button.test.tsx
graphyne-web/tests/e2e/playwright.config.ts        -- Playwright configuration
graphyne-web/tests/e2e/helpers.ts                  -- E2E test helpers
graphyne-web/tests/e2e/dashboard.spec.ts
graphyne-web/tests/e2e/search.spec.ts
graphyne-web/tests/e2e/memory.spec.ts
graphyne-web/tests/e2e/settings.spec.ts
graphyne-web/tests/e2e/navigation.spec.ts
.tarpaulin.toml                                    -- Coverage configuration
.github/workflows/ci.yml                           -- Main CI workflow
.github/workflows/release.yml                      -- Release automation
.github/workflows/security-audit.yml               -- Security audit
.github/workflows/lighthouse.yml                   -- Web performance CI
```

### Existing Files to Modify

```
graphyne-core/src/memory/store.rs                  -- Add store_integration_tests module
graphyne-core/src/memory/context.rs                -- Add edge case tests
graphyne-core/src/memory/retention.rs              -- Add is_expired + boundary tests
graphyne-core/src/memory/scoring.rs                -- Add boundary + recency tests
graphyne-core/src/graph/mod.rs                     -- Add error path + complex graph tests
graphyne-core/src/graph/rag.rs                     -- Add seeded data tests
graphyne-core/src/lexical/mod.rs                   -- Add error path + accuracy tests
graphyne-core/src/vector/mod.rs                    -- Add accuracy + rebuild tests
graphyne-core/src/embeddings/mod.rs                -- Add edge case tests
graphyne-core/src/health/mod.rs                    -- Add error path tests
graphyne-core/src/admin/mod.rs                     -- Add operation tests
graphyne-core/src/scoring/mod.rs                   -- Add threshold + partial combine tests
graphyne-core/src/plugins/mod.rs                   -- Add hook chain + error tests
graphyne-core/Cargo.toml                           -- Add criterion dev-dependency
graphyne-web/package.json                          -- Add test dependencies + scripts
graphyne-web/vite.config.ts                        -- Already exists, keep as-is
```

---

## 6. Success Metrics

| Metric | Current | Phase 1-2 Target | Phase 3-5 Target |
|--------|---------|-------------------|-------------------|
| Total test count | 32 | ~130 | ~200+ |
| Rust code coverage | ~15% | ~60% | ~80% |
| Web code coverage | 0% | ~50% | ~70% |
| Integration tests | 0 | ~50 | ~75 |
| E2E tests | 0 | 0 | ~15 |
| CI pipeline | None | Rust + Web | Full (CI + E2E + Coverage) |
| Benchmark baselines | None | N/A | Established |
