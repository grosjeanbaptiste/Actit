use crate::error::Error;
use crate::model::{Candidate, Evidence, Finding, Report, Verdict};
use crate::need::Need;
use crate::ports::{Catalog, Verifier};

/// A snippet must repeat at least this share of the need's terms to confirm a tool:
/// kag scores relative to the pages it read, so a high score alone proves nothing.
pub const MIN_COVERAGE: f32 = 0.5;

/// Below this kag confidence a documentation hit does not confirm the tool.
pub const MIN_CONFIDENCE: f32 = 0.5;

/// Ask indexer for every term, rank by terms matched, let kag verify the best `limit`.
pub fn find(
    need: &Need,
    catalog: &dyn Catalog,
    verifier: &dyn Verifier,
    limit: usize,
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
    for (candidate, term_hits) in ranked.into_iter().take(limit) {
        match verifier.verify(need, &candidate) {
            Ok(Some(evidence)) => {
                confirmed |= confirms(need, &evidence);
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

fn confirms(need: &Need, evidence: &Evidence) -> bool {
    evidence.confidence >= MIN_CONFIDENCE && need.coverage(&evidence.snippet) >= MIN_COVERAGE
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
