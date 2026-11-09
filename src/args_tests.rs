use super::*;

fn parse(v: &[&str]) -> Result<Option<NameArgs>, Error> {
    parse_args(&v.iter().map(|s| s.to_string()).collect::<Vec<_>>())
}

fn args(name: &str, owner: Option<&str>, contains: bool) -> Option<NameArgs> {
    Some(NameArgs { name: name.into(), owner: owner.map(String::from), contains })
}

#[test]
fn a_need_is_not_a_name_check() {
    assert_eq!(parse(&["serialize", "json"]), Ok(None));
    assert_eq!(parse(&[]), Ok(None));
}

#[test]
fn name_with_owner_or_contains() {
    assert_eq!(parse(&["--name", "x"]), Ok(args("x", None, false)));
    assert_eq!(parse(&["--name", "x", "--owner", "o"]), Ok(args("x", Some("o"), false)));
    assert_eq!(parse(&["--name", "x", "--contains"]), Ok(args("x", None, true)));
}

#[test]
fn bad_usage_is_an_error() {
    for bad in [
        vec!["--name"],
        vec!["--name", "x", "--bogus"],
        vec!["--name", "x", "--owner"],
        vec!["--name", "x", "--owner", "o", "--contains"],
        vec!["--name", "--contains"],
    ] {
        assert!(matches!(parse(&bad), Err(Error::Usage(_))), "{bad:?}");
    }
}
