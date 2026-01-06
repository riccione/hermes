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
fn add_remove_isolated_flow() -> Result<(), Box<dyn std::error::Error>> {
    let file = NamedTempFile::new()?;
    let path = file.path();

    hermes(path)
        .arg("add")
        .args(&["-a", ALIAS, "-c", CODE, "--password", PASSWORD])
        .assert()
        .success();

    hermes(path)
        .arg("remove")
        .args(&["-a", ALIAS])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("Record for {} removed.", ALIAS)));

    Ok(())
}

#[test]
fn add_update_remove_isolated_flow() -> Result<(), Box<dyn std::error::Error>> {
    let file = NamedTempFile::new()?;
    let path = file.path();
    let alias = "test_update";
    let stdout_removed = format!("Record for {alias} removed.");

    hermes(path)
        .arg("add")
        .args(&["-a", alias, "-c", CODE, "--password", PASSWORD])
        .assert()
        .success()
        .stdout(predicate::str::is_match("[0-9]{6}")?);

    hermes(path)
        .arg("update")
        .args(&["-a", alias, "-c", CODE, "--password", PASSWORD])
        .assert()
        .success()
        .stdout(predicate::str::contains(&stdout_removed));

    hermes(path)
        .arg("remove")
        .args(&["-a", alias])
        .assert()
        .success()
        .stdout(predicate::str::contains(stdout_removed));

    Ok(())
}

#[test]
fn rename_alias_isolated_flow() -> Result<(), Box<dyn std::error::Error>> {
    let file = NamedTempFile::new()?;
    let path = file.path();

    // add two initial records
    hermes(path)
        .arg("add")
        .args(&["-a", "github", "-c", CODE, "--password", PASSWORD])
        .assert()
        .success();

    hermes(path)
        .arg("add")
        .args(&["-a", "google", "-c", CODE, "--password", PASSWORD])
        .assert()
        .success();

    // rename: github -> gh
    hermes(path)
        .arg("rename")
        .args(&["github", "gh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully renamed"));

    // verify: new alias exists, old alias is gone
    hermes(path)
        .arg("ls")
        .args(&["-a", "gh"])
        .args(&["--password", PASSWORD])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^\d{6}")?);

    hermes(path)
        .arg("ls")
        .args(&["-a", "github"])
        .args(&["--password", PASSWORD])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Alias not found"));

    // collision Check: try to rename 'gh' to 'google' (exists)
    hermes(path)
        .arg("rename")
        .args(&["gh", "google"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    Ok(())
}
