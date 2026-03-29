//! Integration tests for tos-searchd

use tos_searchd::{SearchState};
use tempfile::tempdir;

#[tokio::test]
async fn test_hybrid_search_logic() -> anyhow::Result<()> {
    // 1. Initialize with In-Memory indices and Candle-Embedder
    let state = SearchState::new()?;
    let dir = tempdir()?;
    
    // 2. Create mock filesystem content
    let rust_file = dir.path().join("rust_tutorial.md");
    std::fs::write(&rust_file, "Learn how to build daemons in Rust for the TOS ecosystem.")?;
    
    let market_file = dir.path().join("market_spec.txt");
    std::fs::write(&market_file, "Marketplace service architecture and fee structure documentation.")?;

    // 3. Index the files
    state.index_file(&rust_file).await?;
    state.index_file(&market_file).await?;

    // 4. Verify Exact Search (Tantivy)
    let exact_rust = state.search("rust");
    assert!(!exact_rust.is_empty(), "Tantivy should find 'rust'");
    assert!(exact_rust[0].path.contains("rust_tutorial.md"));

    let exact_market = state.search("marketplace");
    assert!(!exact_market.is_empty(), "Tantivy should find 'marketplace'");
    assert!(exact_market[0].path.contains("market_spec.txt"));

    // 5. Verify Semantic Search (Candle + HNSW)
    // Querying "building daemons in rust" should semantically match "rust_tutorial.md"
    let semantic_services = state.semantic_search("building daemons in rust");
    assert!(!semantic_services.is_empty(), "Semantic search should return results");
    
    // MiniLM-L6v2 should correctly identify the Rust tutorial as relevant
    let top_hit = &semantic_services[0];
    assert!(top_hit.path.contains("rust_tutorial.md"), 
        "Semantic match failed. Top hit: {}", top_hit.path);

    Ok(())
}
