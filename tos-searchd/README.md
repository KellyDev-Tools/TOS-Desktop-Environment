# TOS Search Service (`tos-searchd`)

A low-latency, hybrid discovery engine for the **Terminal On Steroids**. `tos-searchd` provide a unified interface for exact, regex, and semantic (vector-based) search over system metadata, file contents, and logs.

## 1. Core Architecture

`tos-searchd` implements a **Dual-Engine Strategy** to balance speed and intelligence:

- **Tantivy (Exact Engine):** An inverted index for lightning-fast keyword and regex matching on file paths and log streams.
- **hnsw_rs (Semantic Engine):** A pure-Rust HNSW (Hierarchical Navigable Small World) index for nearest-neighbor retrieval on context embeddings.

### Data Flow Pipeline
1. **Watch (Notify):** Real-time monitoring of the filesystem via `inotify`/`fsevents`.
2. **Pre-Process:** Text extraction and normalization from supported file types (.rs, .md, .toml, .txt).
3. **Embed (ONNX):** Lightweight embedding generation using a local model (AllMiniLML6V2).
4. **Index (Dual-Tier):** Incremental index updates for both Tantivy and HNSW stores.
5. **Serve (UDS):** High-speed JSON-RPC responses to the Brain over `/tmp/tos-search.sock`.

## 2. IPC Protocol (§211)

The service listens on a Unix Domain Socket. It supports the following messages:

### `search:<pattern>`
Filters the global index using exact or regex patterns via Tantivy.
- **Example:** `search:main.rs` -> Returns `[{"path": "src/main.rs", "score": 1.0, ...}]`

### `semantic:<prompt>`
Performs vector-based nearest-neighbor search.
- **Example:** `semantic:how to register a daemon?`

### `rebuild`
Forces a manual re-scan and synchronous embedding update.

## 3. Performance Targets

- **Retrieval Latency:** <10ms for exact match; <50ms for semantic retrieval.
- **Memory Pressure:** <256MB baseline footprint.
- **Indexing Overhead:** <5% CPU usage for background watching.

## 4. Build & Compatibility

### Ubuntu 22.04 (WSL/Linux)
This service includes a manually vendored **ONNX Runtime (v1.16.3)** in `third_party/onnxruntime` to ensure compatibility with GLIBC 2.35.

### GPU Support
Enable CUDA acceleration via features:
```bash
cargo build --features cuda
```
(Requires `ORT_DYLIB_PATH` pointing to a CUDA-enabled ONNX Runtime build).
