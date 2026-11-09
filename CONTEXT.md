# Context actit — 2026-11-09 (GitHub name check added)

## State
- v0 CLI works end to end: `cargo run -- "<need>"`. 32 tests, clippy `-D warnings` and fmt clean.
- Flow: need → terms → indexer `/v0/packages?q=<term>` per term → rank by terms matched →
  top 5 verified by `kag "<need>" --docs <eco>:<id> | --crawl <docs_url|url> --format json`.
- Verdict EXISTS needs kag confidence ≥ 0.5 **and** snippet covering ≥ 50 % of the need's terms
  (kag confidence is relative, 1.0 on off-topic pages: seen with "translate text offline").

- `actit --name <repo> [--owner <o>]`: GitHub name check through `gh` (port `RepoNames`, adapter `github.rs`,
  use case `name.rs`). Exit 0 taken, 1 free, 2 error. Moved here from Actname, whose README says it does not check GitHub.

## Known gaps
- indexer `q` is a single-term filter (a phrase returns `[]`, checked 2026-11-09), so actit queries per term.
- indexer holds 149 packages (pypi 67, homebrew 52, npm 11, github 8, crates_io 5, huggingface_hub 4,
  pub_dev 1, maven_central 1): coverage of "all registries" is only as wide as indexer.
- kag `--docs` resolves crates|pypi|npm|gem|go|hex|cpan|hackage; other registries are crawled by URL.
- Hooks in `.githooks/` (fmt, clippy, tests via heavy queue); enable with `git config core.hooksPath .githooks`.

Sources: `src/*.rs`, indexer API, `kag --help`.
