use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn help_lists_required_commands() {
    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("new"))
        .stdout(predicate::str::contains("mod"))
        .stdout(predicate::str::contains("toc"));
}

#[test]
fn init_creates_marker_directory_template_and_status() {
    let temp = assert_fs::TempDir::new().expect("temp dir");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .args(["init", "docs/adr"])
        .assert()
        .success();

    temp.child(".adr-directory").assert("docs/adr\n");
    temp.child("docs/adr/.adr-template")
        .assert(predicate::path::exists());
    temp.child("docs/adr/.adr-status")
        .assert("DRAFT\nPROPOSED\nACCEPTED\nADOPTED\nSUPERSEDED\nEXPIRED\n");

    temp.close().expect("close temp dir");
}

#[test]
fn init_overwrites_marker_file() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    temp.child(".adr-directory")
        .write_str("old/path\n")
        .expect("seed marker");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .args(["init", "docs/adr"])
        .assert()
        .success();

    temp.child(".adr-directory").assert("docs/adr\n");

    temp.close().expect("close temp dir");
}

#[test]
fn init_does_not_overwrite_existing_template_or_status() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    temp.child("docs/adr").create_dir_all().expect("adr dir");
    temp.child("docs/adr/.adr-template")
        .write_str("custom template\n")
        .expect("seed template");
    temp.child("docs/adr/.adr-status")
        .write_str("CUSTOM\n")
        .expect("seed status");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .args(["init", "docs/adr"])
        .assert()
        .success();

    temp.child("docs/adr/.adr-template")
        .assert("custom template\n");
    temp.child("docs/adr/.adr-status").assert("CUSTOM\n");

    temp.close().expect("close temp dir");
}

#[test]
fn init_failure_returns_non_zero_and_non_tty_error_is_uncolored() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    temp.child("docs")
        .write_str("not a directory")
        .expect("seed file");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .args(["init", "docs/adr"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"))
        .stderr(predicate::str::contains("\u{1b}[31m").not());

    temp.close().expect("close temp dir");
}
