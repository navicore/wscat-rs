use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;

#[test]
fn prints_help() {
    Command::cargo_bin("wscat-rs")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Usage"));
}

#[test]
fn works_without_tty_prefix() {
    Command::cargo_bin("wscat-rs")
        .unwrap()
        .arg("--connect")
        .arg("ws://localhost:...")
        .write_stdin("/ping test")
        .assert()
        .stdout(predicates::str::contains("[binary").not());
}
