//use std::process::Command;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use assert_cmd::Command;

#[test]
fn fail_run_with_defaults() -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin("hermes")
        .expect("binary exists")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Usage: hermes <COMMAND>"));

    Ok(())
}

#[test]
fn fail_add_new_code() -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin("hermes")
        .expect("binary exists")
        .arg("add")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Please provide valid alias and code"));

    Ok(())
}

#[test]
fn add_new_code() -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin("hermes")
        .expect("binary exists")
        .arg("add")
        .args(&["-a", "test"])
        .args(&["-c", "BQZH47HMIUUQOQVAXO3MCRUP3OGR3OIL"]);

    Ok(())
}
