use std::process::{Command, Output};

fn surety(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_surety"))
        .args(args)
        .output()
        .expect("failed to run surety")
}

#[test]
fn version_prints_version() {
    let out = surety(&["--version"]);
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "surety 0.0.1\n");
}

#[test]
fn help_prints_usage() {
    let out = surety(&["--help"]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).starts_with("Usage: surety"));
}

#[test]
fn anything_else_is_not_implemented() {
    for args in [&[][..], &["build"], &["--version", "--help"], &["-V"]] {
        let out = surety(args);
        assert_eq!(out.status.code(), Some(2), "args: {args:?}");
        assert!(out.stdout.is_empty(), "args: {args:?}");
        assert_eq!(
            String::from_utf8_lossy(&out.stderr),
            "surety 0.0.1: not yet implemented\n",
            "args: {args:?}"
        );
    }
}
