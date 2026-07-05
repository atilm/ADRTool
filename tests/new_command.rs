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

#[test]
fn new_creates_first_adr_with_expected_filename_and_metadata() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    init_repo(&temp);

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .args(["new", "Decision X"])
        .assert()
        .success();

    let file = temp.child("docs/adr/001-Decision-X.md");
    file.assert(predicate::path::exists());

    let content = fs::read_to_string(file.path()).expect("read adr file");
    assert!(content.contains("# ADR001 - Decision X"));
    assert!(content.contains("* Status: DRAFT"));
    assert!(content.contains("* Date: "));

    temp.close().expect("close temp dir");
}

#[test]
fn new_uses_max_plus_one_policy_for_id_generation() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    init_repo(&temp);

    temp.child("docs/adr/001-One.md")
        .write_str("# ADR`001` - `One`\n")
        .expect("seed adr");
    temp.child("docs/adr/002-Two.md")
        .write_str("# ADR`002` - `Two`\n")
        .expect("seed adr");
    temp.child("docs/adr/004-Four.md")
        .write_str("# ADR`004` - `Four`\n")
        .expect("seed adr");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .args(["new", "Decision X"])
        .assert()
        .success();

    temp.child("docs/adr/005-Decision-X.md")
        .assert(predicate::path::exists());

    temp.close().expect("close temp dir");
}

#[test]
fn new_silently_ignores_files_that_do_not_match_adr_filename_format() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    init_repo(&temp);

    temp.child("docs/adr/001-One.md")
        .write_str("# ADR001 - One\n")
        .expect("seed adr");
    temp.child("docs/adr/002-Two.md")
        .write_str("# ADR002 - Two\n")
        .expect("seed adr");

    temp.child("docs/adr/notes.md")
        .write_str("not an adr file")
        .expect("seed non-adr file");
    temp.child("docs/adr/123_no_dash.md")
        .write_str("not an adr file")
        .expect("seed non-adr file");
    temp.child("docs/adr/-missing-id.md")
        .write_str("not an adr file")
        .expect("seed non-adr file");
    temp.child("docs/adr/010-no-markdown.txt")
        .write_str("not an adr file")
        .expect("seed non-adr file");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .args(["new", "Decision X"])
        .assert()
        .success();

    temp.child("docs/adr/003-Decision-X.md")
        .assert(predicate::path::exists());

    temp.close().expect("close temp dir");
}

#[test]
fn new_resolves_marker_from_nested_directory() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    init_repo(&temp);
    temp.child("sub/dir").create_dir_all().expect("nested dir");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.child("sub/dir").path())
        .args(["new", "Decision X"])
        .assert()
        .success();

    temp.child("docs/adr/001-Decision-X.md")
        .assert(predicate::path::exists());

    temp.close().expect("close temp dir");
}

#[test]
fn new_fails_when_marker_is_missing() {
    let temp = assert_fs::TempDir::new().expect("temp dir");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .args(["new", "Decision X"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));

    temp.close().expect("close temp dir");
}

#[test]
fn new_fails_when_marker_points_to_missing_adr_directory() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    temp.child(".adr-directory")
        .write_str("docs/adr\n")
        .expect("write marker");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .args(["new", "Decision X"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));

    temp.close().expect("close temp dir");
}

#[test]
fn new_fails_for_invalid_title_characters() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    init_repo(&temp);

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .args(["new", "Decision: X"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid ADR title"));

    temp.close().expect("close temp dir");
}

#[test]
fn new_keeps_succeeding_with_toc_trigger_hook_enabled() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    init_repo(&temp);

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .args(["new", "Hook Smoke"])
        .assert()
        .success();

    temp.child("docs/adr/001-Hook-Smoke.md")
        .assert(predicate::path::exists());
    temp.child("docs/adr/adr-overview.md")
        .assert(predicate::path::exists());

    temp.close().expect("close temp dir");
}
