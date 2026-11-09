use crate::error::Error;

/// A tool recorded by indexer: a package, library, program or repository.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub id: String,
    pub registry: String,
    pub url: String,
    pub docs_url: Option<String>,
    pub description: String,
}

/// What kag cited from the candidate's documentation for the need.
#[derive(Debug, Clone, PartialEq)]
pub struct Evidence {
    pub location: String,
    pub snippet: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub candidate: Candidate,
    pub term_hits: usize,
    pub evidence: Evidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Documentation of a candidate answers the need with enough confidence.
    Exists,
    /// Candidates were found but no documentation confirms them.
    Unconfirmed,
    /// indexer holds no candidate for any term of the need.
    NoCandidate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub verdict: Verdict,
    pub findings: Vec<Finding>,
    /// Candidates kag could not read, with the reason: never dropped silently.
    pub failures: Vec<(Candidate, Error)>,
}
