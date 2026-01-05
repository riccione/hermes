use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;
use std::io::Write;

const CODE: &str = "BQZH47HMIUUQOQVAXO3MCRUP3OGR3OIL";
const ALIAS: &str = "test_simple";
const PASSWORD: &str = "password";

/// helper fn hermes pointing to a temp file
fn hermes(path: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("hermes").expect("binary exists");
    cmd.arg("--path").arg(path);
    cmd
}

#[test]
fn fail_run_with_no_args() -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin("hermes")
        .expect("binary exists")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Usage:"))
        .stderr(predicate::str::contains("<COMMAND>"));

    Ok(())
}

#[test]
fn fail_add_missing_args() -> Result<(), Box<dyn std::error::Error>> {
    let file = NamedTempFile::new()?;
    
    // 'add' fails without -a and -c
    Command::cargo_bin("hermes")?
        .arg("--path")
        .arg(file.path())
        .arg("add")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("the following required arguments were not provided"));

    Ok(())
}

#[test]
#[ignore]
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
#[ignore]
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
