#![allow(clippy::unwrap_used)] // helper outside a #[test] fn: a spawn failure must abort the test

use std::process::{Command, Output};

fn actit(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_actit"))
        .args(args)
        .env_remove("ACTIT_INDEXER")
        .output()
        .unwrap()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn bad_setting_value_is_a_usage_error() {
    let out = actit(&["--min-coverage", "3", "pdf"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("--min-coverage"), "{}", stderr(&out));
}

#[test]
fn missing_stopwords_file_is_reported_by_name() {
    let out = actit(&["--stopwords", "/no/such/words.txt", "pdf"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr(&out).contains("/no/such/words.txt"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn indexer_url_comes_from_the_flag() {
    let out = actit(&["--indexer", "http://127.0.0.1:1", "pdf"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("127.0.0.1:1"), "{}", stderr(&out));
}

#[test]
fn gh_program_comes_from_the_flag() {
    let out = actit(&["--gh", "/no/such/gh", "--name", "x", "--owner", "me"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("/no/such/gh"), "{}", stderr(&out));
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn license_of_a_permissive_expression_is_reusable_exit_0() {
    let out = actit(&["--license", "MIT OR GPL-3.0-only"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(
        stdout(&out).contains("permissive (reusable)"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn license_of_a_copyleft_or_unknown_expression_is_not_reusable_exit_1() {
    for expr in ["GPL-3.0-only", "Frobnicate-1.0", "(MIT"] {
        let out = actit(&["--license", expr]);
        assert_eq!(out.status.code(), Some(1), "{expr}: {}", stdout(&out));
        assert!(
            stdout(&out).contains("not reusable"),
            "{expr}: {}",
            stdout(&out)
        );
    }
}

#[test]
fn the_allow_policy_changes_the_verdict_and_a_bad_policy_is_exit_2() {
    let out = actit(&[
        "--allow-license",
        "permissive,strong-copyleft",
        "--license",
        "GPL-3.0-only",
    ]);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    let out = actit(&["--allow-license", "mit", "--license", "MIT"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("mit"), "{}", stderr(&out));
}
