use crate::error::Error;
use crate::model::{Candidate, Evidence};
use crate::need::Need;
use crate::ports::Verifier;
use serde::Deserialize;

/// Process seam: runs `kag` with these arguments, returns stdout.
pub trait Runner {
    fn run(&self, args: &[String]) -> Result<String, String>;
}

pub struct KagVerifier<R: Runner> {
    runner: R,
    ecosystems: Ecosystems,
}

#[derive(Deserialize)]
struct Answer {
    sources: Vec<Source>,
}

#[derive(Deserialize)]
struct Source {
    location: String,
    snippet: String,
    confidence: f32,
}

impl<R: Runner> KagVerifier<R> {
    pub fn new(runner: R) -> Self {
        Self {
            runner,
            ecosystems: Ecosystems::default(),
        }
    }

    pub fn with_ecosystems(mut self, ecosystems: Ecosystems) -> Self {
        self.ecosystems = ecosystems;
        self
    }
}

impl<R: Runner> Verifier for KagVerifier<R> {
    fn verify(&self, need: &Need, candidate: &Candidate) -> Result<Option<Evidence>, Error> {
        let out = self
            .runner
            .run(&kag_args(need, candidate, &self.ecosystems))
            .map_err(Error::Kag)?;
        let answer: Answer = serde_json::from_str(&out)
            .map_err(|e| Error::Kag(format!("invalid output for {}: {e}", candidate.id)))?;
        let best = answer
            .sources
            .into_iter()
            .max_by(|a, b| a.confidence.total_cmp(&b.confidence));
        Ok(best.map(|s| Evidence {
            location: s.location,
            snippet: s.snippet,
            confidence: s.confidence,
        }))
    }
}

/// Which `kag --docs` ecosystem resolves each indexer registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ecosystems(Vec<(String, String)>);

impl Default for Ecosystems {
    fn default() -> Self {
        let pairs = include_str!("../resources/ecosystems.txt")
            .lines()
            .filter_map(|l| l.split_once('='))
            .map(|(r, e)| (r.trim().to_string(), e.trim().to_string()))
            .collect();
        Self(pairs)
    }
}

impl Ecosystems {
    /// `extra` pairs win over the ones already known.
    pub fn with(mut self, extra: impl IntoIterator<Item = (String, String)>) -> Self {
        for (registry, eco) in extra {
            self.0.retain(|(known, _)| *known != registry);
            self.0.push((registry, eco));
        }
        self
    }

    pub fn get(&self, registry: &str) -> Option<&str> {
        let found = self.0.iter().find(|(known, _)| known == registry);
        found.map(|(_, eco)| eco.as_str())
    }
}

pub fn kag_args(need: &Need, candidate: &Candidate, ecosystems: &Ecosystems) -> Vec<String> {
    let source = match ecosystems.get(&candidate.registry) {
        Some(eco) => ["--docs".to_string(), format!("{eco}:{}", candidate.id)],
        None => {
            let url = candidate.docs_url.as_ref().unwrap_or(&candidate.url);
            ["--crawl".to_string(), url.clone()]
        }
    };
    let mut args = vec![need.text().to_string()];
    args.extend(source);
    args.extend(["--format".to_string(), "json".to_string()]);
    args
}

#[cfg(test)]
mod tests {
    include!("kag_tests.rs");
}
