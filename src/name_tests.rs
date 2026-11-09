use super::*;

struct Fixed(Result<bool, Error>);

impl RepoNames for Fixed {
    fn owner(&self) -> Result<String, Error> {
        Ok("me".into())
    }
    fn exists(&self, _: &str, _: &str) -> Result<bool, Error> {
        self.0.clone()
    }
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
