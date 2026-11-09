use super::*;

struct Fake(Result<Listing, Error>);

impl RepoNames for Fake {
    fn owner(&self) -> Result<String, Error> {
        Ok("me".into())
    }
    fn exists(&self, _: &str, _: &str) -> Result<bool, Error> {
        Ok(false)
    }
    fn search(&self, _: &str) -> Result<Vec<String>, Error> {
        Ok(vec![])
    }
    fn contains(&self, _: &str) -> Result<Listing, Error> {
        self.0.clone()
    }
}

fn listing(total: u64, names: &[&str]) -> Listing {
    Listing { total, names: names.iter().map(|s| s.to_string()).collect() }
}

#[test]
fn list_returns_what_the_registry_found() {
    let l = list("pr", &Fake(Ok(listing(3, &["a/pr", "b/pr-x"])))).unwrap();
    assert_eq!((l.total, l.names.len()), (3, 2));
}

#[test]
fn registry_failure_and_invalid_name_are_errors() {
    let boom = Error::Github("rate limit".into());
    assert_eq!(list("pr", &Fake(Err(boom.clone()))).err(), Some(boom));
    assert!(matches!(list("a b", &Fake(Ok(listing(0, &[])))), Err(Error::InvalidName(_))));
}

#[test]
fn exit_code_is_0_when_something_matches() {
    assert_eq!(exit_code(&listing(2, &["a/pr"])), 0);
    assert_eq!(exit_code(&listing(0, &[])), 1);
}

#[test]
fn render_gives_total_and_every_listed_name() {
    let text = render("pr", &listing(30, &["a/pr", "b/pr-x"]));
    assert_eq!(text, "CONTAINS pr: GitHub reports 30 matches, 2 listed\n- a/pr\n- b/pr-x\n");
}

#[test]
fn render_says_so_when_nothing_matches() {
    assert!(render("zzqq", &listing(0, &[])).starts_with("NONE zzqq"));
}
