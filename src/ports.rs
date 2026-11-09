use crate::error::Error;
use crate::model::{Candidate, Evidence};
use crate::need::Need;

/// Where candidates come from: indexer, the only catalogue.
pub trait Catalog {
    fn packages(&self, term: &str) -> Result<Vec<Candidate>, Error>;
}

/// How a candidate is checked against the need: kag, the only reader.
pub trait Verifier {
    /// `Ok(None)`: kag read the documentation and found nothing for the need.
    fn verify(&self, need: &Need, candidate: &Candidate) -> Result<Option<Evidence>, Error>;
}

/// Where repository names live: GitHub, asked through `gh`.
pub trait RepoNames {
    /// The login whose repositories are checked when no owner is given.
    fn owner(&self) -> Result<String, Error>;
    fn exists(&self, owner: &str, name: &str) -> Result<bool, Error>;
    /// `owner/name` of the repositories, under any owner, whose name is exactly `name`.
    /// Only the first 100 best matches are read (exact names rank first).
    fn search(&self, name: &str) -> Result<Vec<String>, Error>;
}
