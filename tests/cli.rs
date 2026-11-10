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
