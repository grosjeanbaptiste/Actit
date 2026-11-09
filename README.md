# actit

Does a tool already exist for this need? Actit answers from **indexer** (the
catalogue of packages) and **kag** (reads their documentation, cites sources).
It uses no other search engine and no LLM.

## Install

Needs Rust (`rustup`, toolchain pinned by `rust-toolchain.toml`) and `kag` on `PATH`.

```text
# 1. kag, the reader (private repository: needs SSH access to it)
cargo install --locked --git git@github.com:grosjeanbaptiste/kag.git kag-cli

# 2. actit
cargo install --locked --git https://github.com/grosjeanbaptiste/Actit

# 3. check
export PATH="$HOME/.cargo/bin:$PATH"
kag --help | head -1
actit "serialize rust structs to json"
```

From a checkout: `cargo install --locked --path .`. `actit` installs to
`~/.cargo/bin/actit`; update by re-running step 2 with `--force`.

## API

### Command line

```text
actit "<need, in plain words>"
```

One argument group: every word after `actit` is joined into the need.

Requires `kag` on `PATH` and network access to https://indexer.acteble.workers.dev.

### Is a repository name taken on GitHub?

```text
actit --name <repo> [--owner <owner>]
```

Asks GitHub through `gh` (so the owner's private repositories count). Without
`--owner`, the authenticated login is used. Prints `TAKEN <owner>/<repo>` or
`FREE <owner>/<repo>`; exit `0` taken, `1` free, `2` error (invalid name, `gh`
missing or unauthenticated, any `gh` failure other than 404). Requires `gh` on `PATH`.

Output, on stdout:

```text
EXISTS
- serde (crates_io) https://crates.io/crates/serde
  1.00 https://docs.rs/serde/... <snippet kag cited>
! some_tool (github): kag: <why this candidate could not be read>
```

- First line is the verdict: `EXISTS`, `UNCONFIRMED` (candidates found, no
  documentation confirms them) or `NO CANDIDATE` (indexer knows nothing for these terms).
- `- <id> (<registry>) <url>` is a candidate kag read; the next line is its
  confidence, the page cited and the snippet.
- `! <id> (<registry>): <reason>` is a candidate that could not be read. It is
  never dropped silently.

| Exit code | Meaning |
|---|---|
| `0` | `EXISTS`: a documented tool answers the need |
| `1` | `UNCONFIRMED` or `NO CANDIDATE` |
| `2` | error (empty need, indexer or kag failure) or at least one `!` line; message on stderr |

### How a verdict is decided

1. The need is split into terms (lowercase, stopwords removed, 3 letters or more).
2. indexer `/v0/packages?q=<term>` is asked once per term; candidates are ranked by
   how many terms matched.
3. The best 5 are read by `kag "<need>" --docs <ecosystem>:<id> --format json`
   (crates, pypi, npm, gem, go, hex, cpan, hackage) or
   `kag "<need>" --crawl <docs_url> --format json` for other registries.
4. A candidate confirms the need when kag's confidence is at least 0.5 **and** the
   cited snippet repeats at least half of the need's terms.

### Library

```rust,no_run
use actit::{indexer::IndexerCatalog, kag::KagVerifier, need::Need,
            process::{KagProcess, UreqHttp}, search::find};

let need = Need::new("serialize rust structs to json")?;
let catalog = IndexerCatalog::new("https://indexer.acteble.workers.dev", &UreqHttp);
let verifier = KagVerifier::new(KagProcess::new("kag"));
let report = find(&need, &catalog, &verifier, 5)?; // Report { verdict, findings, failures }
# Ok::<(), actit::error::Error>(())
```

`Catalog` and `Verifier` (in `ports`) are the two seams: another catalogue or
reader plugs in by implementing them.

### Limits

indexer holds a few hundred packages, so a `NO CANDIDATE` means "not in indexer",
not "does not exist". `--name` only covers GitHub, for one owner: a name free
there may be taken on another registry, domain or trademark.

## Develop

```text
dotfiles heavy run --medium -- cargo test
dotfiles heavy run --medium -- cargo clippy --all-targets -- -D warnings
```

Hooks in `.githooks/` run the same gate: `git config core.hooksPath .githooks`.
