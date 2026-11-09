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
        Self { runner }
    }
}

impl<R: Runner> Verifier for KagVerifier<R> {
    fn verify(&self, need: &Need, candidate: &Candidate) -> Result<Option<Evidence>, Error> {
        let out = self
            .runner
            .run(&kag_args(need, candidate))
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

/// Ecosystems `kag --docs` resolves by itself, keyed by indexer registry id.
fn kag_ecosystem(registry: &str) -> Option<&'static str> {
    match registry {
        "crates_io" => Some("crates"),
        "pypi" => Some("pypi"),
        "npm" => Some("npm"),
        "rubygems" => Some("gem"),
        "go" => Some("go"),
        "hex" => Some("hex"),
        "cpan" => Some("cpan"),
        "hackage" => Some("hackage"),
        _ => None,
    }
}

pub fn kag_args(need: &Need, candidate: &Candidate) -> Vec<String> {
    let source = match kag_ecosystem(&candidate.registry) {
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
