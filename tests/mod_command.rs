use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

fn init_repo(temp: &assert_fs::TempDir) {
    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .args(["init", "docs/adr"])
        .assert()
        .success();
}

fn current_local_date() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

#[test]
fn mod_accept_updates_status_and_date() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    init_repo(&temp);

    temp.child("docs/adr/001-Test.md")
        .write_str(
            "# ADR001 - Test\n\n* Date: 2020-01-01\n* Status: DRAFT\n* Author: Jane\n* Labels: core\n",
        )
        .expect("seed adr");

    let today = current_local_date();

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .args(["mod", "1", "-a"])
        .assert()
        .success();

    let content = fs::read_to_string(temp.child("docs/adr/001-Test.md").path()).expect("read adr");
    assert!(content.contains("* Status: ACCEPTED"));
    assert!(content.contains(format!("* Date: {today}").as_str()));

    temp.child("docs/adr/adr-overview.md")
        .assert(predicate::path::exists());

    temp.close().expect("close temp dir");
}

#[test]
fn mod_supersede_updates_both_adrs_and_only_new_date_changes() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    init_repo(&temp);

    temp.child("docs/adr/001-Old.md")
        .write_str(
            "# ADR001 - Old\n\n* Date: 2020-01-01\n* Status: ACCEPTED\n* Author: Jane\n* Labels: old\n",
        )
        .expect("seed old adr");
    temp.child("docs/adr/002-New.md")
        .write_str(
            "# ADR002 - New\n\n* Date: 2021-01-01\n* Status: DRAFT\n* Author: Jane\n* Labels: new\n",
        )
        .expect("seed new adr");

    let today = current_local_date();

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .args(["mod", "2", "-s", "1"])
        .assert()
        .success();

    let new_content =
        fs::read_to_string(temp.child("docs/adr/002-New.md").path()).expect("read new adr");
    assert!(new_content.contains("* Status: ACCEPTED"));
    assert!(new_content.contains(format!("* Date: {today}").as_str()));

    let old_content =
        fs::read_to_string(temp.child("docs/adr/001-Old.md").path()).expect("read old adr");
    assert!(old_content.contains("* Status: SUPERSEDED by 002"));
    assert!(old_content.contains("* Date: 2020-01-01"));

    temp.close().expect("close temp dir");
}

#[test]
fn mod_fails_when_target_id_is_missing() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    init_repo(&temp);

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .args(["mod", "42", "-a"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"))
        .stderr(predicate::str::contains("ADR '042' not found"));

    temp.close().expect("close temp dir");
}

#[test]
fn mod_fails_when_superseded_id_is_missing() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    init_repo(&temp);

    temp.child("docs/adr/002-New.md")
        .write_str(
            "# ADR002 - New\n\n* Date: 2021-01-01\n* Status: DRAFT\n* Author: Jane\n* Labels: new\n",
        )
        .expect("seed new adr");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .args(["mod", "2", "-s", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"))
        .stderr(predicate::str::contains("ADR '001' not found"));

    temp.close().expect("close temp dir");
}

#[test]
fn mod_resolves_marker_from_nested_directory() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    init_repo(&temp);

    temp.child("docs/adr/001-Test.md")
        .write_str(
            "# ADR001 - Test\n\n* Date: 2020-01-01\n* Status: DRAFT\n* Author: Jane\n* Labels: core\n",
        )
        .expect("seed adr");
    temp.child("sub/dir").create_dir_all().expect("nested dir");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.child("sub/dir").path())
        .args(["mod", "1", "-a"])
        .assert()
        .success();

    let content = fs::read_to_string(temp.child("docs/adr/001-Test.md").path()).expect("read adr");
    assert!(content.contains("* Status: ACCEPTED"));

    temp.close().expect("close temp dir");
}

#[test]
fn mod_best_effort_inserts_missing_metadata_and_warns() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    init_repo(&temp);

    temp.child("docs/adr/001-Test.md")
        .write_str("# ADR001 - Test\n\n* Author: Jane\n* Labels: core\n")
        .expect("seed adr");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .args(["mod", "1", "-a"])
        .assert()
        .success()
        .stderr(predicate::str::contains("warning:"))
        .stderr(predicate::str::contains(
            "missing metadata line '* Date:' inserted",
        ))
        .stderr(predicate::str::contains(
            "missing metadata line '* Status:' inserted",
        ));

    let content = fs::read_to_string(temp.child("docs/adr/001-Test.md").path()).expect("read adr");
    assert!(content.contains("* Date: "));
    assert!(content.contains("* Status: ACCEPTED"));

    temp.close().expect("close temp dir");
}
