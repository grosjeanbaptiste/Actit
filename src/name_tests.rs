use super::*;

struct Fixed(Result<bool, Error>);

impl RepoNames for Fixed {
    fn owner(&self) -> Result<String, Error> {
        Ok("me".into())
    }
    fn exists(&self, _: &str, _: &str) -> Result<bool, Error> {
        self.0.clone()
    }
    fn search(&self, _: &str) -> Result<Vec<String>, Error> {
        Ok(vec![])
    }
}

struct Anywhere {
    own: Result<bool, Error>,
    found: Result<Vec<String>, Error>,
}

impl RepoNames for Anywhere {
    fn owner(&self) -> Result<String, Error> {
        Ok("me".into())
    }
    fn exists(&self, _: &str, _: &str) -> Result<bool, Error> {
        self.own.clone()
    }
    fn search(&self, _: &str) -> Result<Vec<String>, Error> {
        self.found.clone()
    }
}

fn anywhere(own: bool, found: &[&str]) -> Anywhere {
    Anywhere { own: Ok(own), found: Ok(found.iter().map(|s| s.to_string()).collect()) }
}

#[test]
fn taken_and_free_follow_the_registry() {
    assert_eq!(check("me", "x", &Fixed(Ok(true))), Ok(NameStatus::Taken));
    assert_eq!(check("me", "x", &Fixed(Ok(false))), Ok(NameStatus::Free));
}

#[test]
fn registry_failure_propagates() {
    let boom = Error::Github("boom".into());
    assert_eq!(check("me", "x", &Fixed(Err(boom.clone()))), Err(boom));
}

#[test]
fn invalid_names_are_rejected_before_any_call() {
    for bad in ["", "a/b", "a b", ".", ".."] {
        let r = check("me", bad, &Fixed(Err(Error::Github("must not be called".into()))));
        assert!(matches!(r, Err(Error::InvalidName(_))), "{bad:?}");
    }
}

#[test]
fn exit_codes_follow_the_need_convention() {
    assert_eq!(exit_code(NameStatus::Taken), 0);
    assert_eq!(exit_code(NameStatus::Free), 1);
}

#[test]
fn render_names_owner_and_repo() {
    assert_eq!(render("me", "x", NameStatus::Taken), "TAKEN me/x\n");
    assert_eq!(render("me", "x", NameStatus::Free), "FREE me/x\n");
}

#[test]
fn args_name_with_optional_owner() {
    let a = |v: &[&str]| parse_args(&v.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    assert_eq!(a(&["--name", "x"]), Ok(Some(("x".into(), None))));
    assert_eq!(a(&["--name", "x", "--owner", "o"]), Ok(Some(("x".into(), Some("o".into())))));
    assert_eq!(a(&["serialize", "json"]), Ok(None));
    assert!(matches!(a(&["--name"]), Err(Error::Usage(_))));
    assert!(matches!(a(&["--name", "x", "--bogus"]), Err(Error::Usage(_))));
}

#[test]
fn anywhere_is_taken_when_another_owner_has_the_name() {
    let out = check_anywhere("pr", &anywhere(false, &["bob/pr", "eve/PR"])).unwrap();
    assert_eq!(out.status, NameStatus::Taken);
    assert_eq!(out.matches, ["bob/pr", "eve/PR"]);
}

#[test]
fn anywhere_is_free_when_nobody_has_it() {
    let out = check_anywhere("pr", &anywhere(false, &[])).unwrap();
    assert_eq!((out.status, out.matches.len()), (NameStatus::Free, 0));
}

#[test]
fn own_private_repository_counts_even_if_search_misses_it() {
    let out = check_anywhere("pr", &anywhere(true, &["bob/pr"])).unwrap();
    assert_eq!(out.matches, ["me/pr", "bob/pr"]);
}

#[test]
fn own_repository_is_not_listed_twice() {
    let out = check_anywhere("pr", &anywhere(true, &["ME/pr"])).unwrap();
    assert_eq!(out.matches, ["ME/pr"]);
}

#[test]
fn anywhere_failures_propagate_and_names_are_validated() {
    let boom = Error::Github("rate limit".into());
    let s = Anywhere { own: Ok(false), found: Err(boom.clone()) };
    assert_eq!(check_anywhere("pr", &s).err(), Some(boom));
    assert!(matches!(check_anywhere("a b", &anywhere(false, &[])), Err(Error::InvalidName(_))));
}

#[test]
fn render_anywhere_lists_matches_and_caps_them() {
    let many: Vec<String> = (0..12).map(|i| format!("o{i}/pr")).collect();
    let out = Outcome { status: NameStatus::Taken, matches: many };
    let text = render_anywhere("pr", &out);
    assert!(text.starts_with("TAKEN pr\n- o0/pr\n"), "{text}");
    assert!(text.contains("- o9/pr\n") && !text.contains("o10/pr"), "{text}");
    assert!(text.ends_with("(+2 more)\n"), "{text}");
    let free = Outcome { status: NameStatus::Free, matches: vec![] };
    assert!(render_anywhere("pr", &free).starts_with("FREE pr"));
}
