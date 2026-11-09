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
}
