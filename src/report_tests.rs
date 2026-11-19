use super::*;
use crate::error::Error;
use crate::model::{Candidate, Evidence, Finding};

fn cand() -> Candidate {
    Candidate {
        id: "serde".into(),
        registry: "crates_io".into(),
        url: "https://crates.io/crates/serde".into(),
        docs_url: None,
        description: String::new(),
        license: None,
    }
}

fn report(verdict: Verdict, failures: Vec<(Candidate, Error)>) -> Report {
    let evidence = Evidence { location: "https://docs.rs/serde".into(), snippet: "s".into(), confidence: 0.9 };
    let findings = vec![Finding { candidate: cand(), term_hits: 2, evidence }];
    Report { verdict, findings, failures }
}

#[test]
fn exists_lists_each_tool_with_its_citation() {
    let text = render(&report(Verdict::Exists, vec![]));
    assert!(text.starts_with("EXISTS"), "{text}");
    assert!(text.contains("serde (crates_io) https://crates.io/crates/serde"));
    assert!(text.contains("https://docs.rs/serde"));
}

#[test]
fn no_candidate_says_so() {
    let empty = Report { verdict: Verdict::NoCandidate, findings: vec![], failures: vec![] };
    assert!(render(&empty).starts_with("NO CANDIDATE"));
}

#[test]
fn failures_are_rendered_with_their_reason() {
    let text = render(&report(Verdict::Unconfirmed, vec![(cand(), Error::Kag("login wall".into()))]));
    assert!(text.contains("serde") && text.contains("login wall"), "{text}");
}

#[test]
fn exit_codes_tell_exists_from_not_and_failures() {
    assert_eq!(exit_code(&report(Verdict::Exists, vec![])), 0);
    assert_eq!(exit_code(&report(Verdict::Unconfirmed, vec![])), 1);
    assert_eq!(exit_code(&report(Verdict::NoCandidate, vec![])), 1);
    assert_eq!(exit_code(&report(Verdict::Exists, vec![(cand(), Error::Kag("x".into()))])), 2);
}

#[test]
fn licensed_rendering_puts_the_verdict_under_each_finding() {
    use crate::license::{check, default_allow, Table};
    let r = report(Verdict::Exists, vec![]);
    let mut licensed = r.clone();
    licensed.findings[0].candidate.license = Some("MIT".into());
    let checks: Vec<_> = licensed
        .findings
        .iter()
        .map(|f| check(f.candidate.license.as_deref(), &Table::builtin(), &default_allow()))
        .collect();
    let text = render_licensed(&licensed, &checks);
    assert!(text.starts_with("EXISTS"), "{text}");
    let lines: Vec<&str> = text.lines().collect();
    let at = lines.iter().position(|l| l.contains("serde (crates_io)")).unwrap();
    assert_eq!(lines[at + 2], "  licence MIT: permissive (reusable)", "{text}");
    assert_eq!(render(&licensed), render_licensed(&licensed, &[]), "no checks, same text");
}
