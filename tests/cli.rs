use assert_cmd::Command;
use predicates::prelude::*;

const CODE: &str = "BQZH47HMIUUQOQVAXO3MCRUP3OGR3OIL";
const ALIAS: &str = "test_simple";
const PASSWORD: &str = "password";

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
        .stderr(predicate::str::contains(
            "error: the following required arguments were not provided",
        ));

    Ok(())
}

#[test]
fn add_remove_code_simple() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hermes").expect("binary exists");

    cmd.arg("add")
        .args(&["-a", ALIAS])
        .args(&["-c", CODE])
        .args(&["-p", PASSWORD])
        .assert()
        .success()
        .stdout(predicate::str::is_match("[0-9]{6}").expect("Regex error!"));

    let mut cmd = Command::cargo_bin("hermes").expect("binary exists");

    cmd.arg("add")
        .args(&["-a", ALIAS])
        .args(&["-c", CODE])
        .args(&["-p", PASSWORD])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Alias already exists, please select another one",
        ));

    let mut cmd = Command::cargo_bin("hermes").expect("binary exists");

    let stdout_removed = format!("Record for {ALIAS} has been removed from codex");

    cmd.arg("remove")
        .args(&["-a", ALIAS])
        .assert()
        .success()
        .stdout(predicate::str::contains(stdout_removed));

    Ok(())
}

#[test]
fn add_update_remove_code_simple() -> Result<(), Box<dyn std::error::Error>> {
    let alias = "test_update";
    let stdout_removed = format!("Record for {alias} has been removed from codex");

    let mut cmd = Command::cargo_bin("hermes").expect("binary exists");

    cmd.arg("add")
        .args(&["-a", alias])
        .args(&["-c", CODE])
        .args(&["-p", PASSWORD])
        .assert()
        .success()
        .stdout(predicate::str::is_match("[0-9]{6}").expect("Regex error!"));

    let mut cmd = Command::cargo_bin("hermes").expect("binary exists");

    cmd.arg("update")
        .args(&["-a", alias])
        .args(&["-c", CODE])
        .args(&["-p", PASSWORD])
        .assert()
        .success()
        .stdout(predicate::str::contains(stdout_removed.clone()));

    let mut cmd = Command::cargo_bin("hermes").expect("binary exists");

    cmd.arg("remove")
        .args(&["-a", alias])
        .assert()
        .success()
        .stdout(predicate::str::contains(stdout_removed));

    Ok(())
}

#[test]
#[ignore]
fn fail_add_new_code_simple() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
