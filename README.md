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
actit --name <repo>                  # under any owner
actit --name <repo> --owner <owner>  # one owner, exact
actit --name <repo> --contains       # every repository whose name contains <repo>
```

Asks GitHub through [actautoma](https://github.com/grosjeanbaptiste/actautoma) (its `gh` adapter; `--gh <path>` picks the binary). Without `--owner`, every owner counts: your own
repositories (private ones included) are checked exactly, the others come from
GitHub search (the first 100 best matches, where exact names rank first, so
`FREE` means "none among them"). Prints `TAKEN <repo>` and up to 10
`- owner/repo` lines, or `FREE <repo>`. With `--owner`, prints `TAKEN|FREE
<owner>/<repo>`. Exit `0` taken, `1` free, `2` error (invalid name, `gh`
missing or unauthenticated, rate limit, any `gh` failure other than 404).
Requires `gh` on `PATH`. A valid name follows GitHub: 1 to 100 characters of `A-Z a-z 0-9 . _ -`,
not `.` or `..`, not ending in `.git`.

`--contains` (not combinable with `--owner`) lists repositories whose name
contains the text: `CONTAINS <repo>: GitHub reports <N> matches, <k> listed`
then one `- owner/repo` line per repository (the first 100 best matches; `N` is
GitHub's own count and may exceed `k`), or `NONE <repo>`. Exit `0` something
matches, `1` none, `2` error.

### Settings

Nothing is fixed in the code: every value below is a flag, and each one except
`--ecosystem` can also be set by the environment variable `ACTIT_<FLAG>` (upper case,
`-` becomes `_`; `ACTIT_ECOSYSTEMS=a=b,c=d` for ecosystems). A flag beats the
environment, which beats the default. Flags may appear anywhere on the command line.

| Flag | Default | Meaning |
|---|---|---|
| `--indexer <url>` | `https://indexer.acteble.workers.dev` | catalogue asked for candidates |
| `--kag <program>` | `kag` | reader run on each candidate |
| `--gh <program>` | `gh` | GitHub client used by `--name` |
| `--limit <n>` | `5` | candidates read by kag |
| `--min-confidence <0..1>` | `0.5` | kag confidence needed to confirm |
| `--min-coverage <0..1>` | `0.5` | share of the need's terms the snippet must repeat |
| `--min-term-length <n>` | `3` | shortest word kept as a search term |
| `--stopwords <file>` | `resources/stopwords.txt` | one word per line, `#` comments; replaces the built-in list |
| `--ecosystem <registry=eco>` | `resources/ecosystems.txt` | repeatable; adds to or overrides the registry→`kag --docs` map |

The defaults are written once, in `Settings::default()`; the word and ecosystem
lists are data files under `resources/`. Library callers pass the same values as
`TermRules` (`Need::with_rules`), `Policy` (`search::find_with`) and `Ecosystems`
(`KagVerifier::with_ecosystems`).

### How a verdict is decided

1. The need is split into terms (lowercase, stopwords removed, 3 letters or more).
2. indexer `/v0/packages?q=<term>` is asked once per term; candidates are ranked by
   how many terms matched.
3. The best `--limit` are read by `kag "<need>" --docs <ecosystem>:<id> --format json`
   (crates, pypi, npm, gem, go, hex, cpan, hackage) or
   `kag "<need>" --crawl <docs_url> --format json` for other registries.
4. A candidate confirms the need when kag's confidence is at least `--min-confidence` **and** the
   cited snippet repeats at least `--min-coverage` of the need's terms.

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
not "does not exist". `--name` only covers GitHub: a name free there may be taken
on another registry, domain or trademark. GitHub search is capped at 30 requests a minute.

## Develop

```text
dotfiles heavy run --medium -- cargo test
dotfiles heavy run --medium -- cargo clippy --all-targets -- -D warnings
```

Hooks in `.githooks/` run the same gate: `git config core.hooksPath .githooks`.
