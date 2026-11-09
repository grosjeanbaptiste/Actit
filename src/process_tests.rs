use super::*;
use std::os::unix::fs::PermissionsExt;

fn stub(dir: &tempfile::TempDir, body: &str) -> String {
    let path = dir.path().join("kag-stub");
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path.to_str().unwrap().to_string()
}

fn args() -> Vec<String> {
    vec!["q".into()]
}

#[test]
fn stdout_is_returned_and_args_are_forwarded() {
    let dir = tempfile::tempdir().unwrap();
    let out = KagProcess::new(&stub(&dir, "echo \"got $1\"")).run(&args()).unwrap();
    assert_eq!(out.trim(), "got q");
}

#[test]
fn non_zero_exit_returns_stderr() {
    let dir = tempfile::tempdir().unwrap();
    let err = KagProcess::new(&stub(&dir, "echo boom >&2; exit 3")).run(&args()).unwrap_err();
    assert!(err.contains("boom") && err.contains("3"), "{err}");
}

#[test]
fn missing_program_names_it() {
    let err = KagProcess::new("/no/such/kag").run(&args()).unwrap_err();
    assert!(err.contains("/no/such/kag"), "{err}");
}
