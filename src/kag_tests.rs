use super::*;
use std::cell::RefCell;

struct FakeRunner {
    out: Result<String, String>,
    args: RefCell<Vec<String>>,
}

impl Runner for FakeRunner {
    fn run(&self, args: &[String]) -> Result<String, String> {
        *self.args.borrow_mut() = args.to_vec();
        self.out.clone()
    }
}

fn runner(out: Result<&str, &str>) -> FakeRunner {
    FakeRunner { out: out.map(String::from).map_err(String::from), args: RefCell::default() }
}

fn cand(registry: &str, docs: Option<&str>) -> Candidate {
    Candidate {
        id: "serde".into(),
        registry: registry.into(),
        url: "https://home.example/serde".into(),
        docs_url: docs.map(String::from),
        description: String::new(),
    }
}

fn need() -> Need {
    Need::new("serialize structs").unwrap()
}

#[test]
fn registry_known_to_kag_uses_docs_resolution() {
    let args = kag_args(&need(), &cand("crates_io", Some("https://docs.rs/serde")), &Ecosystems::default());
    assert_eq!(args, ["serialize structs", "--docs", "crates:serde", "--format", "json"]);
}

#[test]
fn other_registry_crawls_docs_url_then_url() {
    let with = kag_args(&need(), &cand("homebrew", Some("https://docs.example")), &Ecosystems::default());
    assert_eq!(with[1..3], ["--crawl", "https://docs.example"]);
    let without = kag_args(&need(), &cand("github", None), &Ecosystems::default());
    assert_eq!(without[1..3], ["--crawl", "https://home.example/serde"]);
}

#[test]
fn best_source_becomes_the_evidence() {
    let out = r#"{"question":"q","answer":"a","sources":[
      {"location":"l1","snippet":"weak","confidence":0.2},
      {"location":"l2","snippet":"strong","confidence":0.9}]}"#;
    let verifier = KagVerifier::new(runner(Ok(out)));
    let evidence = verifier.verify(&need(), &cand("pypi", None)).unwrap().unwrap();
    assert_eq!((evidence.location.as_str(), evidence.confidence), ("l2", 0.9));
}

#[test]
fn no_source_means_no_evidence() {
    let out = r#"{"question":"q","answer":"","sources":[]}"#;
    let got = KagVerifier::new(runner(Ok(out))).verify(&need(), &cand("pypi", None)).unwrap();
    assert_eq!(got, None);
}

#[test]
fn kag_failure_and_garbage_are_errors() {
    let failing = KagVerifier::new(runner(Err("login wall")));
    assert_eq!(
        failing.verify(&need(), &cand("pypi", None)).unwrap_err(),
        Error::Kag("login wall".into())
    );
    let garbage = KagVerifier::new(runner(Ok("Resolved crates:serde")));
    let err = garbage.verify(&need(), &cand("pypi", None)).unwrap_err();
    assert!(matches!(err, Error::Kag(m) if m.contains("invalid output")));
}

#[test]
fn extra_ecosystems_extend_and_override_the_built_in_ones() {
    let eco = Ecosystems::default().with([("homebrew".to_string(), "brew".to_string())]);
    let args = kag_args(&need(), &cand("homebrew", None), &eco);
    assert_eq!(args[1..3], ["--docs", "brew:serde"]);
    let eco = Ecosystems::default().with([("pypi".to_string(), "pip".to_string())]);
    assert_eq!(kag_args(&need(), &cand("pypi", None), &eco)[2], "pip:serde");
}

#[test]
fn built_in_ecosystems_come_from_the_data_file() {
    let eco = Ecosystems::default();
    assert_eq!(eco.get("crates_io"), Some("crates"));
    assert_eq!(eco.get("unknown_registry"), None);
}

#[test]
fn verifier_uses_the_ecosystems_it_was_given() {
    let out = r#"{"sources":[]}"#;
    let run = runner(Ok(out));
    let eco = Ecosystems::default().with([("homebrew".to_string(), "brew".to_string())]);
    let verifier = KagVerifier::new(run).with_ecosystems(eco);
    verifier.verify(&need(), &cand("homebrew", None)).unwrap();
    assert_eq!(verifier.runner.args.borrow()[2], "brew:serde");
}
