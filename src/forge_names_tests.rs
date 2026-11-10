use super::*;
use actautoma::forge::{Error as ForgeError, Runner};
use actautoma::gh::Gh;
use std::cell::RefCell;
use std::path::Path;

/// A scripted `gh`: the names adapter is exercised through the real `Gh` adapter.
struct Fake {
    reply: Result<String, String>,
    calls: RefCell<Vec<Vec<String>>>,
}

fn fake(reply: Result<&str, &str>) -> Fake {
    Fake {
        reply: reply.map(String::from).map_err(String::from),
        calls: RefCell::new(vec![]),
    }
}

impl Runner for &Fake {
    fn run(&self, _: &str, args: &[&str], _: Option<&Path>) -> Result<String, ForgeError> {
        self.calls.borrow_mut().push(args.iter().map(|a| a.to_string()).collect());
        self.reply.clone().map_err(ForgeError::Run)
    }
}

fn names(f: &Fake) -> ForgeNames<Gh<&Fake>> {
    ForgeNames::new(Gh::new(f))
}

#[test]
fn existing_repository_is_taken() {
    let f = fake(Ok("{}"));
    assert_eq!(names(&f).exists("me", "x"), Ok(true));
    assert_eq!(f.calls.borrow()[0], ["api", "repos/me/x"]);
}

#[test]
fn not_found_means_free() {
    let f = fake(Err("gh exited with 1: gh: Not Found (HTTP 404)"));
    assert_eq!(names(&f).exists("me", "x"), Ok(false));
}

#[test]
fn any_other_failure_is_an_error_keeping_the_reason() {
    let f = fake(Err("gh exited with 1: HTTP 401: Bad credentials"));
    let err = names(&f).exists("me", "x");
    assert!(matches!(err, Err(Error::Github(m)) if m.contains("Bad credentials")));
}

#[test]
fn owner_is_the_authenticated_login() {
    let f = fake(Ok("me\n"));
    assert_eq!(names(&f).owner(), Ok("me".to_string()));
}

#[test]
fn empty_or_failing_owner_is_an_error() {
    assert!(matches!(names(&fake(Ok("\n"))).owner(), Err(Error::Github(_))));
    assert!(matches!(names(&fake(Err("no auth"))).owner(), Err(Error::Github(m)) if m.contains("no auth")));
}

#[test]
fn search_keeps_only_exact_names_case_insensitively() {
    let f = fake(Ok("4\nbob/pr\nacme/pr-agent\neve/PR\nzed/xpr\n"));
    assert_eq!(names(&f).search("pr"), Ok(vec!["bob/pr".into(), "eve/PR".into()]));
}

#[test]
fn search_failure_is_an_error() {
    let f = fake(Err("HTTP 403: rate limit"));
    assert!(matches!(names(&f).search("pr"), Err(Error::Github(m)) if m.contains("rate")));
}

#[test]
fn contains_reports_the_platform_total_and_keeps_names_containing_the_text() {
    let f = fake(Ok("42\nbob/pr\nacme/my-PR-tool\nzed/zzz\n"));
    let l = names(&f).contains("pr").unwrap();
    assert_eq!(l, Listing { total: 42, names: vec!["bob/pr".into(), "acme/my-PR-tool".into()] });
}

#[test]
fn contains_with_unreadable_total_or_failure_is_an_error() {
    assert!(matches!(names(&fake(Ok("oops\n"))).contains("pr"), Err(Error::Github(_))));
    assert!(matches!(names(&fake(Err("HTTP 403"))).contains("pr"), Err(Error::Github(_))));
}

#[test]
fn an_adapter_without_name_lookups_says_so() {
    struct NoNames;
    impl actautoma::forge::Forge for NoNames {
        fn view_pr(&self, _: Option<u64>) -> Result<actautoma::forge::PullRequest, ForgeError> { Err(ForgeError::Unsupported("x".into())) }
        fn create_pr(&self, _: &actautoma::forge::NewPullRequest) -> Result<String, ForgeError> { Err(ForgeError::Unsupported("x".into())) }
        fn merge_pr(&self, _: u64) -> Result<(), ForgeError> { Err(ForgeError::Unsupported("x".into())) }
        fn view_repo(&self) -> Result<actautoma::forge::Repo, ForgeError> { Err(ForgeError::Unsupported("x".into())) }
        fn set_default_branch(&self, _: &str, _: &str) -> Result<(), ForgeError> { Err(ForgeError::Unsupported("x".into())) }
        fn create_repo(&self, _: &actautoma::forge::NewRepo) -> Result<(), ForgeError> { Err(ForgeError::Unsupported("x".into())) }
        fn available(&self) -> Result<(), ForgeError> { Ok(()) }
    }
    let err = ForgeNames::new(NoNames).owner();
    assert!(matches!(&err, Err(Error::Github(m)) if m.contains("not supported")), "{err:?}");
}
