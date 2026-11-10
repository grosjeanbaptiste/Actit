# Context actit — 2026-11-09 (GitHub name check added)

## State
- v0 CLI works end to end: `cargo run -- "<need>"`. 32 tests, clippy `-D warnings` and fmt clean.
- Flow: need → terms → indexer `/v0/packages?q=<term>` per term → rank by terms matched →
  top 5 verified by `kag "<need>" --docs <eco>:<id> | --crawl <docs_url|url> --format json`.
- Verdict EXISTS needs kag confidence ≥ 0.5 **and** snippet covering ≥ 50 % of the need's terms
  (kag confidence is relative, 1.0 on off-topic pages: seen with "translate text offline").

- `actit --name <repo> [--owner <o>]`: GitHub name check through actautoma's `Forge` (`Gh` adapter; port `RepoNames`, adapter `forge_names.rs`,
  use case `name.rs`). Without `--owner`: any owner (own repos exact, others via search API, first 100 best matches).
  `--contains` lists repos whose name contains the text (total from GitHub + first 100; `contains.rs`, args in `args.rs`).
  Exit 0 taken, 1 free, 2 error. Moved here from Actname, whose README says it does not check GitHub.

## Known gaps
- indexer `q` is a single-term filter (a phrase returns `[]`, checked 2026-11-09), so actit queries per term.
- indexer holds 149 packages (pypi 67, homebrew 52, npm 11, github 8, crates_io 5, huggingface_hub 4,
  pub_dev 1, maven_central 1): coverage of "all registries" is only as wide as indexer.
- kag `--docs` resolves crates|pypi|npm|gem|go|hex|cpan|hackage; other registries are crawled by URL.
- Hooks in `.githooks/` (fmt, clippy, tests via heavy queue); enable with `git config core.hooksPath .githooks`.

Sources: `src/*.rs`, indexer API, `kag --help`.

- 2026-11-10: hardcoded values removed from the code (branch `feat/configurable`): `Settings` (flags + `ACTIT_*` env,
  defaults in one place), `TermRules`, `Policy`, `Ecosystems`; word and ecosystem lists live in `resources/`.
  Still written in code: GitHub's own rules and search page size (`name.rs`, `forge_names.rs`), kag's `--format json`/`--docs`/`--crawl` protocol.
- 2026-11-10 (branch `feat/forge-names`): Actit now depends on the `actautoma` crate (git dep, pinned `d0abc17`). `github.rs`/`GhNames` is replaced by `forge_names.rs`/`ForgeNames<F: Forge>`: Actit no longer spawns `gh` itself; actautoma's `Gh` adapter does (`user`, `repo_exists`, `search_repos`), the exact/contains filtering stays here. Same output and exit codes. Verified live with the real `gh`: `--name actautoma --owner grosjeanbaptiste` TAKEN (exit 0), a random name FREE (1), no owner and `--contains` list `grosjeanbaptiste/actautoma`, `--gh /no/such/gh` exit 2 with `not found`. 83 unit + 4 CLI tests pass, clippy `-D warnings` clean. Limit: GitLab (`Glab`) answers `Unsupported` for these lookups.
