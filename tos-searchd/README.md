# TOS Search Service (`tos-searchd`)

A unified search index for file contents, logs, and system metadata.

## Design Drivers
- **Architecture Spec §4**: Identifies the search service as a core auxiliary component.
- **Architecture Spec §211**: Defines the `search_query` API and requirement for regex/semantic filters.

## Responsibilities
- Indexing filesystem and TOS log events.
- Providing high-speed search results for the Hub [SEARCH] mode.
- (Planned) Vector-based semantic search integration via Faiss.
