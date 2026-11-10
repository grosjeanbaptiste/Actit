use super::*;
use std::collections::HashMap;

fn cand(id: &str) -> Candidate {
    Candidate {
        id: id.into(),
        registry: "crates_io".into(),
        url: format!("https://crates.io/crates/{id}"),
        docs_url: Some(format!("https://docs.rs/{id}")),
        description: String::new(),
    }
}

fn evidence(confidence: f32) -> Evidence {
    Evidence { location: "https://docs.rs/x".into(), snippet: "convert pdf text with serialize".into(), confidence }
}

#[derive(Default)]
struct FakeCatalog(HashMap<&'static str, Vec<&'static str>>);

impl Catalog for FakeCatalog {
    fn packages(&self, term: &str) -> Result<Vec<Candidate>, Error> {
        if term == "boom" {
            return Err(Error::Catalog("down".into()));
        }
        Ok(self.0.get(term).into_iter().flatten().map(|id| cand(id)).collect())
    }
}

struct FakeVerifier(HashMap<&'static str, Result<Option<f32>, &'static str>>);

impl Verifier for FakeVerifier {
    fn verify(&self, _: &Need, c: &Candidate) -> Result<Option<Evidence>, Error> {
        match self.0.get(c.id.as_str()) {
            Some(Ok(Some(conf))) => Ok(Some(evidence(*conf))),
            Some(Err(why)) => Err(Error::Kag((*why).into())),
            _ => Ok(None),
        }
    }
}

fn catalog(rows: &[(&'static str, &[&'static str])]) -> FakeCatalog {
    FakeCatalog(rows.iter().map(|(t, ids)| (*t, ids.to_vec())).collect())
}

fn verifier(rows: &[(&'static str, Result<Option<f32>, &'static str>)]) -> FakeVerifier {
    FakeVerifier(rows.iter().cloned().collect())
}

fn need(text: &str) -> Need {
    Need::new(text).unwrap()
}

#[test]
fn no_candidate_when_indexer_knows_nothing() {
    let report = find(&need("pdf text"), &catalog(&[]), &verifier(&[]), 5).unwrap();
    assert_eq!(report.verdict, Verdict::NoCandidate);
    assert!(report.findings.is_empty());
}

#[test]
fn candidates_are_ranked_by_matching_terms_and_deduplicated() {
    let cat = catalog(&[("pdf", &["a", "b"]), ("text", &["b"])]);
    let ver = verifier(&[("a", Ok(Some(0.9))), ("b", Ok(Some(0.9)))]);
    let report = find(&need("pdf text"), &cat, &ver, 5).unwrap();
    let ids: Vec<_> = report.findings.iter().map(|f| f.candidate.id.as_str()).collect();
    assert_eq!(ids, ["b", "a"]);
    assert_eq!(report.findings[0].term_hits, 2);
}

#[test]
fn only_the_best_candidates_are_verified() {
    let cat = catalog(&[("pdf", &["a", "b", "c"])]);
    let ver = verifier(&[("a", Ok(Some(0.9))), ("b", Ok(Some(0.9))), ("c", Ok(Some(0.9)))]);
    let report = find(&need("pdf"), &cat, &ver, 2).unwrap();
    assert_eq!(report.findings.len(), 2);
}

#[test]
fn confident_evidence_means_exists() {
    let cat = catalog(&[("pdf", &["a"])]);
    let report = find(&need("pdf"), &cat, &verifier(&[("a", Ok(Some(0.8)))]), 5).unwrap();
    assert_eq!(report.verdict, Verdict::Exists);
}

#[test]
fn weak_or_missing_evidence_stays_unconfirmed() {
    let cat = catalog(&[("pdf", &["a", "b"])]);
    let ver = verifier(&[("a", Ok(Some(0.1))), ("b", Ok(None))]);
    let report = find(&need("pdf"), &cat, &ver, 5).unwrap();
    assert_eq!(report.verdict, Verdict::Unconfirmed);
}

#[test]
fn catalog_failure_propagates() {
    let err = find(&need("boom"), &catalog(&[]), &verifier(&[]), 5).unwrap_err();
    assert_eq!(err, Error::Catalog("down".into()));
}

#[test]
fn verifier_failure_is_reported_and_does_not_hide_other_candidates() {
    let cat = catalog(&[("pdf", &["a", "b"])]);
    let ver = verifier(&[("a", Err("login wall")), ("b", Ok(Some(0.9)))]);
    let report = find(&need("pdf"), &cat, &ver, 5).unwrap();
    assert_eq!(report.verdict, Verdict::Exists);
    assert_eq!(report.failures.len(), 1);
    assert_eq!(report.failures[0].1, Error::Kag("login wall".into()));
}

#[test]
fn same_id_in_two_registries_stays_two_candidates() {
    struct TwoRegistries;
    impl Catalog for TwoRegistries {
        fn packages(&self, _: &str) -> Result<Vec<Candidate>, Error> {
            let mut other = cand("a");
            other.registry = "pypi".into();
            Ok(vec![cand("a"), other])
        }
    }
    let ver = verifier(&[("a", Ok(Some(0.9)))]);
    let report = find(&need("pdf"), &TwoRegistries, &ver, 5).unwrap();
    assert_eq!(report.findings.len(), 2);
}

#[test]
fn confident_but_off_topic_snippet_does_not_confirm() {
    let cat = catalog(&[("kubernetes", &["a"])]);
    let ver = verifier(&[("a", Ok(Some(1.0)))]);
    let report = find(&need("kubernetes operator"), &cat, &ver, 5).unwrap();
    assert_eq!(report.verdict, Verdict::Unconfirmed);
}

#[test]
fn policy_thresholds_decide_the_verdict() {
    let cat = catalog(&[("pdf", &["a"])]);
    let ver = verifier(&[("a", Ok(Some(0.6)))]);
    let n = need("pdf");
    let strict = Policy { min_confidence: 0.9, ..Policy::default() };
    assert_eq!(find_with(&n, &cat, &ver, &strict).unwrap().verdict, Verdict::Unconfirmed);
    let lax = Policy { min_confidence: 0.1, ..Policy::default() };
    assert_eq!(find_with(&n, &cat, &ver, &lax).unwrap().verdict, Verdict::Exists);
}

#[test]
fn policy_coverage_decides_the_verdict() {
    let cat = catalog(&[("pdf", &["a"])]);
    let ver = verifier(&[("a", Ok(Some(1.0)))]);
    let n = need("pdf nonsenseword");
    let strict = Policy { min_coverage: 1.0, ..Policy::default() };
    assert_eq!(find_with(&n, &cat, &ver, &strict).unwrap().verdict, Verdict::Unconfirmed);
    let lax = Policy { min_coverage: 0.5, ..Policy::default() };
    assert_eq!(find_with(&n, &cat, &ver, &lax).unwrap().verdict, Verdict::Exists);
}
