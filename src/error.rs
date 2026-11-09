#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("the need is empty")]
    EmptyNeed,
    #[error("the need has no searchable term: {0:?}")]
    NoTerms(String),
    #[error("indexer: {0}")]
    Catalog(String),
    #[error("kag: {0}")]
    Kag(String),
    #[error("github: {0}")]
    Github(String),
    #[error("not a valid GitHub repository name: {0:?}")]
    InvalidName(String),
    #[error("usage: {0}")]
    Usage(String),
}
