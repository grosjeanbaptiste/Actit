use super::*;
use std::cell::RefCell;

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
    fn run(&self, args: &[String]) -> Result<String, String> {
        self.calls.borrow_mut().push(args.to_vec());
        self.reply.clone()
    }
}

#[test]
fn existing_repository_is_taken() {
    let f = fake(Ok("{}"));
    assert_eq!(GhNames::new(&f).exists("me", "x"), Ok(true));
    assert_eq!(f.calls.borrow()[0], ["api", "repos/me/x"]);
}

#[test]
fn not_found_means_free() {
    let f = fake(Err("gh exited with 1: gh: Not Found (HTTP 404)"));
    assert_eq!(GhNames::new(&f).exists("me", "x"), Ok(false));
}

#[test]
fn any_other_failure_is_an_error() {
    let f = fake(Err("gh exited with 1: HTTP 401: Bad credentials"));
    let err = GhNames::new(&f).exists("me", "x");
    assert!(matches!(err, Err(Error::Github(m)) if m.contains("Bad credentials")));
}

#[test]
fn owner_is_the_authenticated_login() {
    let f = fake(Ok("me\n"));
    assert_eq!(GhNames::new(&f).owner(), Ok("me".to_string()));
    assert_eq!(f.calls.borrow()[0], ["api", "user", "-q", ".login"]);
}

#[test]
fn empty_or_failing_owner_is_an_error() {
    assert!(matches!(GhNames::new(&fake(Ok("\n"))).owner(), Err(Error::Github(_))));
    assert!(matches!(GhNames::new(&fake(Err("no auth"))).owner(), Err(Error::Github(_))));
}

#[test]
fn search_keeps_only_exact_names_case_insensitively() {
    let f = fake(Ok("bob/pr\nacme/pr-agent\neve/PR\nzed/xpr\n"));
    assert_eq!(GhNames::new(&f).search("pr"), Ok(vec!["bob/pr".into(), "eve/PR".into()]));
    let call = f.calls.borrow()[0].clone();
    assert_eq!(call[1], "search/repositories?q=pr+in:name&per_page=100");
}

#[test]
fn search_failure_is_an_error() {
    let f = fake(Err("HTTP 403: rate limit"));
    assert!(matches!(GhNames::new(&f).search("pr"), Err(Error::Github(m)) if m.contains("rate")));
}
