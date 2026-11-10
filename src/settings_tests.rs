use super::*;

fn s(args: &[&str]) -> Vec<String> {
    args.iter().map(|a| a.to_string()).collect()
}

fn no_env(_: &str) -> Option<String> {
    None
}

#[test]
fn defaults_apply_when_nothing_is_given() {
    let (settings, rest) = Settings::parse(&s(&["pdf", "text"]), &no_env).unwrap();
    assert_eq!(settings, Settings::default());
    assert_eq!(rest, s(&["pdf", "text"]));
}

#[test]
fn flags_are_consumed_and_the_rest_is_returned_in_order() {
    let args = s(&["--indexer", "https://i", "pdf", "--limit", "3", "text"]);
    let (settings, rest) = Settings::parse(&args, &no_env).unwrap();
    assert_eq!((settings.indexer.as_str(), settings.limit), ("https://i", 3));
    assert_eq!(rest, s(&["pdf", "text"]));
}

#[test]
fn flag_beats_environment_beats_default() {
    let env = |k: &str| (k == "ACTIT_KAG").then(|| "/env/kag".to_string());
    let (from_env, _) = Settings::parse(&[], &env).unwrap();
    assert_eq!(from_env.kag, "/env/kag");
    let (from_flag, _) = Settings::parse(&s(&["--kag", "/flag/kag"]), &env).unwrap();
    assert_eq!(from_flag.kag, "/flag/kag");
}

#[test]
fn policy_numbers_are_parsed_and_checked() {
    let args = s(&["--min-confidence", "0.7", "--min-coverage", "0.25", "--min-term-length", "4"]);
    let (settings, _) = Settings::parse(&args, &no_env).unwrap();
    assert_eq!(settings.min_confidence, 0.7);
    assert_eq!(settings.min_coverage, 0.25);
    assert_eq!(settings.min_term_length, 4);
    for bad in [["--limit", "x"], ["--min-coverage", "1.5"], ["--min-term-length", "0"]] {
        let err = Settings::parse(&s(&bad), &no_env).unwrap_err();
        assert!(matches!(err, Error::Usage(m) if m.contains(bad[0])), "{bad:?}");
    }
}

#[test]
fn missing_value_is_a_usage_error_naming_the_flag() {
    let err = Settings::parse(&s(&["--indexer"]), &no_env).unwrap_err();
    assert!(matches!(err, Error::Usage(m) if m.contains("--indexer")));
}

#[test]
fn ecosystems_are_repeatable_and_come_from_env_too() {
    let args = s(&["--ecosystem", "rubygems=gem", "--ecosystem", "go_proxy=go"]);
    let (settings, _) = Settings::parse(&args, &no_env).unwrap();
    assert_eq!(settings.ecosystems, [("rubygems".into(), "gem".into()), ("go_proxy".into(), "go".into())]);
    let env = |k: &str| (k == "ACTIT_ECOSYSTEMS").then(|| "a=b,c=d".to_string());
    let (from_env, _) = Settings::parse(&[], &env).unwrap();
    assert_eq!(from_env.ecosystems, [("a".into(), "b".into()), ("c".into(), "d".into())]);
}

#[test]
fn malformed_ecosystem_is_refused() {
    let err = Settings::parse(&s(&["--ecosystem", "nogood"]), &no_env).unwrap_err();
    assert!(matches!(err, Error::Usage(m) if m.contains("registry=ecosystem")));
}

#[test]
fn unrelated_flags_pass_through_for_the_name_command() {
    let args = s(&["--name", "x", "--owner", "me"]);
    let (_, rest) = Settings::parse(&args, &no_env).unwrap();
    assert_eq!(rest, args);
}
