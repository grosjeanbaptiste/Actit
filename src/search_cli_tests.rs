use super::*;

fn words(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn search_needs_a_need_and_reports_a_catalogue_failure_by_name() {
    let none = |_: &str| None;
    assert_eq!(search(&words(&[]), &none).err(), Some(Error::EmptyNeed));
    let got = search(&words(&["--indexer", "http://127.0.0.1:1", "pdf", "text"]), &none);
    assert!(matches!(got, Err(Error::Catalog(ref m)) if m.contains("127.0.0.1:1")), "{:?}", got.err());
}

#[test]
fn search_refuses_a_bad_policy_before_any_network_call() {
    let none = |_: &str| None;
    let got = search(&words(&["--allow-license", "mit", "pdf"]), &none);
    assert!(matches!(got, Err(Error::License(_))), "{:?}", got.err());
}

#[test]
fn reusable_keeps_only_the_findings_with_an_allowed_licence() {
    use crate::license::{check, default_allow, Table};
    use crate::model::{Candidate, Evidence, Finding, Report, Verdict};
    let finding = |id: &str, lic: &str| Finding {
        candidate: Candidate {
            id: id.into(),
            registry: "npm".into(),
            url: "u".into(),
            docs_url: None,
            description: String::new(),
            license: Some(lic.into()),
        },
        term_hits: 1,
        evidence: Evidence { location: "l".into(), snippet: "s".into(), confidence: 0.9 },
    };
    let report = Report {
        verdict: Verdict::Exists,
        findings: vec![finding("free", "MIT"), finding("viral", "AGPL-3.0-only")],
        failures: vec![],
    };
    let licenses = report
        .findings
        .iter()
        .map(|f| check(f.candidate.license.as_deref(), &Table::builtin(), &default_allow()))
        .collect();
    let outcome = Outcome { report, licenses };
    let ids: Vec<_> = outcome.reusable().iter().map(|f| f.candidate.id.clone()).collect();
    assert_eq!(ids, ["free"]);
}
