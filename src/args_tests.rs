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

fn words(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn license_takes_the_rest_of_the_line_as_one_expression() {
    assert_eq!(parse_license(&words(&["--license", "MIT", "OR", "Apache-2.0"])).unwrap(), Some("MIT OR Apache-2.0".to_string()));
    assert_eq!(parse_license(&words(&["pdf", "text"])).unwrap(), None);
    assert!(parse_license(&words(&["--license"])).is_err());
}
