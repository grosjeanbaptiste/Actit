use crate::error::Error;
use crate::model::{Candidate, Evidence, Finding, Report, Verdict};
use crate::need::Need;
use crate::ports::{Catalog, Verifier};
use crate::settings::Settings;

/// What it takes to confirm a tool, and how many candidates are read.
/// A snippet must repeat `min_coverage` of the need's terms: kag scores relative to the
/// pages it read, so a high `min_confidence` score alone proves nothing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Policy {
    pub limit: usize,
    pub min_confidence: f32,
    pub min_coverage: f32,
}

impl Default for Policy {
    fn default() -> Self {
        let defaults = Settings::default();
        Self {
            limit: defaults.limit,
            min_confidence: defaults.min_confidence,
            min_coverage: defaults.min_coverage,
        }
    }
}

/// Ask indexer for every term, rank by terms matched, let kag verify the best `limit`.
pub fn find(
    need: &Need,
    catalog: &dyn Catalog,
    verifier: &dyn Verifier,
    limit: usize,
) -> Result<Report, Error> {
    let policy = Policy {
        limit,
        ..Policy::default()
    };
    find_with(need, catalog, verifier, &policy)
}

pub fn find_with(
    need: &Need,
    catalog: &dyn Catalog,
    verifier: &dyn Verifier,
    policy: &Policy,
) -> Result<Report, Error> {
    let ranked = rank(need, catalog)?;
    if ranked.is_empty() {
        return Ok(Report {
            verdict: Verdict::NoCandidate,
            findings: vec![],
            failures: vec![],
        });
    }
    let (mut findings, mut failures) = (Vec::new(), Vec::new());
    let mut confirmed = false;
    for (candidate, term_hits) in ranked.into_iter().take(policy.limit) {
        match verifier.verify(need, &candidate) {
            Ok(Some(evidence)) => {
                confirmed |= confirms(need, &evidence, policy);
                findings.push(finding(candidate, term_hits, evidence));
            }
            Ok(None) => {}
            Err(error) => failures.push((candidate, error)),
        }
    }
    let verdict = if confirmed {
        Verdict::Exists
    } else {
        Verdict::Unconfirmed
    };
    Ok(Report {
        verdict,
        findings,
        failures,
    })
}

fn confirms(need: &Need, evidence: &Evidence, policy: &Policy) -> bool {
    evidence.confidence >= policy.min_confidence
        && need.coverage(&evidence.snippet) >= policy.min_coverage
}

fn finding(candidate: Candidate, term_hits: usize, evidence: Evidence) -> Finding {
    Finding {
        candidate,
        term_hits,
        evidence,
    }
}

fn rank(need: &Need, catalog: &dyn Catalog) -> Result<Vec<(Candidate, usize)>, Error> {
    let mut hits: Vec<(Candidate, usize)> = Vec::new();
    for term in need.terms() {
        for candidate in catalog.packages(term)? {
            match hits
                .iter_mut()
                .find(|(known, _)| known.id == candidate.id && known.registry == candidate.registry)
            {
                Some((_, count)) => *count += 1,
                None => hits.push((candidate, 1)),
            }
        }
    }
    hits.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.id.cmp(&b.0.id)));
    Ok(hits)
}

#[cfg(test)]
mod tests {
    include!("search_tests.rs");
}
