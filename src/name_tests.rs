use super::*;
use crate::contains::Listing;

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
    fn contains(&self, _: &str) -> Result<Listing, Error> {
        Ok(Listing { total: 0, names: vec![] })
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
    fn contains(&self, _: &str) -> Result<Listing, Error> {
        Ok(Listing { total: 0, names: vec![] })
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

#[test]
fn github_length_limit_is_100_characters() {
    let ok = "a".repeat(100);
    let long = "a".repeat(101);
    assert_eq!(check("me", &ok, &Fixed(Ok(false))), Ok(NameStatus::Free));
    assert!(matches!(check("me", &long, &Fixed(Ok(false))), Err(Error::InvalidName(_))));
}

#[test]
fn a_dot_git_suffix_is_refused_in_any_case() {
    for bad in ["x.git", "x.GIT", "x.Git", ".git"] {
        let r = check("me", bad, &Fixed(Err(Error::Github("must not be called".into()))));
        assert!(matches!(r, Err(Error::InvalidName(_))), "{bad:?}");
    }
}

#[test]
fn names_that_only_look_like_the_git_suffix_stay_valid() {
    for ok in [".github", "git", "x.gitx", "my.git.tool", "a-.git-b"] {
        assert_eq!(check("me", ok, &Fixed(Ok(false))), Ok(NameStatus::Free), "{ok:?}");
    }
}

#[test]
fn unicode_and_symbols_are_refused() {
    for bad in ["ifdçk", "日本", "a+b", "a@b", "a:b", "a*b", "a\\b", "a~b"] {
        let r = check("me", bad, &Fixed(Err(Error::Github("must not be called".into()))));
        assert!(matches!(r, Err(Error::InvalidName(_))), "{bad:?}");
    }
}

#[test]
fn contains_and_anywhere_apply_the_same_rules() {
    let long = "a".repeat(101);
    let any = anywhere(false, &[]);
    assert!(matches!(check_anywhere(&long, &any), Err(Error::InvalidName(_))));
    assert!(matches!(check_anywhere("x.git", &any), Err(Error::InvalidName(_))));
}
