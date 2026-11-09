# actit

Does a tool already exist for this need? Answers only from **indexer** (catalogue) and **kag** (reader).
No other search engine, no LLM. Pure TDD in Rust: red test first, then code.

- Read `CONTEXT.md` first.
- Run cargo through `dotfiles heavy run --medium -- cargo test|clippy --all-targets -- -D warnings`; `cargo fmt --check`.
- Hexagonal: `search.rs` is the use case; `ports.rs` holds `Catalog` (indexer) and `Verifier` (kag);
  `RepoNames` (GitHub, via `gh`) is a third port: `name.rs` use case, `github.rs` adapter.
  `indexer.rs`, `kag.rs`, `process.rs` are adapters behind the `Http`/`Runner` seams.
- No silent failure: adapter errors become `Error`; unreadable candidates land in `Report.failures`.
