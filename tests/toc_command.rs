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
fn toc_regenerates_overview_sorted_and_listed() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    init_repo(&temp);

    temp.child("docs/adr/0010-Tenth.md")
        .write_str(
            "# ADR010 - Tenth\n\n* Date: 2026-07-01\n* Status: ACCEPTED\n* Author: Jane\n* Labels: ten\n",
        )
        .expect("write adr");
    temp.child("docs/adr/0002-Second.md")
        .write_str(
            "# ADR002 - Second\n\n* Date: 2026-07-01\n* Status: ACCEPTED\n* Author: Jane\n* Labels: two\n",
        )
        .expect("write adr");
    temp.child("docs/adr/0001-First.md")
        .write_str(
            "# ADR001 - First\n\n* Date: 2026-07-01\n* Status: ACCEPTED\n* Author: Jane\n* Labels: one\n",
        )
        .expect("write adr");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path()).arg("toc").assert().success();

    let overview_path = temp.child("docs/adr/adr-overview.md");
    overview_path.assert(predicate::path::exists());
    let overview = fs::read_to_string(overview_path.path()).expect("read overview");

    let first_idx = overview
        .find("[ADR0001 - First](0001-First.md)")
        .expect("first entry");
    let second_idx = overview
        .find("[ADR0002 - Second](0002-Second.md)")
        .expect("second entry");
    let tenth_idx = overview
        .find("[ADR0010 - Tenth](0010-Tenth.md)")
        .expect("tenth entry");
    assert!(first_idx < second_idx && second_idx < tenth_idx);

    temp.close().expect("close temp dir");
}

#[test]
fn toc_applies_required_status_styles_and_labels() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    init_repo(&temp);

    temp.child("docs/adr/0001-Draft.md")
        .write_str(
            "# ADR001 - Draft\n\n* Date: 2026-07-01\n* Status: DRAFT\n* Author: Jane\n* Labels: alpha,beta\n",
        )
        .expect("write adr");
    temp.child("docs/adr/0002-Proposed.md")
        .write_str(
            "# ADR002 - Proposed\n\n* Date: 2026-07-01\n* Status: PROPOSED\n* Author: Jane\n* Labels: proposal\n",
        )
        .expect("write adr");
    temp.child("docs/adr/0003-Superseded.md")
        .write_str(
            "# ADR003 - Superseded\n\n* Date: 2026-07-01\n* Status: SUPERSEDED by 004\n* Author: Jane\n* Labels: history\n",
        )
        .expect("write adr");
    temp.child("docs/adr/0004-Expired.md")
        .write_str(
            "# ADR004 - Expired\n\n* Date: 2026-07-01\n* Status: EXPIRED\n* Author: Jane\n* Labels: \n",
        )
        .expect("write adr");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path()).arg("toc").assert().success();

    let overview =
        fs::read_to_string(temp.child("docs/adr/adr-overview.md").path()).expect("read overview");

    assert!(overview.contains("**[ADR0001 - Draft](0001-Draft.md)** (alpha, beta)"));
    assert!(overview.contains("**[ADR0002 - Proposed](0002-Proposed.md)** (proposal)"));
    assert!(
        overview
            .contains("~~[ADR0003 - Superseded](0003-Superseded.md)~~ (history) superseded by 004")
    );
    assert!(overview.contains("~~[ADR0004 - Expired](0004-Expired.md)~~ expired"));

    temp.close().expect("close temp dir");
}

#[test]
fn toc_reports_warnings_but_exits_success() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    init_repo(&temp);

    temp.child("docs/adr/0001-One.md")
        .write_str(
            "# ADR001 - One\n\n* Date: 2026-07-01\n* Status: ACCEPTED\n* Author: Jane\n* Labels: one\n",
        )
        .expect("write adr");
    temp.child("docs/adr/0003-Three.md")
        .write_str(
            "# ADR004 - Three\n\n* Date: invalid\n* Status: ACCEPTED\n* Author: Jane\n* Labels: three\n",
        )
        .expect("write adr");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .arg("toc")
        .assert()
        .success()
        .stderr(predicate::str::contains("warning:"))
        .stderr(predicate::str::contains("missing ADR ID in sequence: 0002"))
        .stderr(predicate::str::contains("ID mismatch"))
        .stderr(predicate::str::contains("invalid date"));

    temp.child("docs/adr/adr-overview.md")
        .assert(predicate::path::exists());

    temp.close().expect("close temp dir");
}

#[test]
fn toc_warns_for_empty_author_value() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    init_repo(&temp);

    temp.child("docs/adr/0001-Empty-Author.md")
        .write_str(
            "# ADR001 - Empty Author\n\n* Date: 2026-07-01\n* Status: ACCEPTED\n* Author: \n* Labels: core\n",
        )
        .expect("write adr");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .arg("toc")
        .assert()
        .success()
        .stderr(predicate::str::contains("warning:"))
        .stderr(predicate::str::contains("empty author value"));

    temp.child("docs/adr/adr-overview.md")
        .assert(predicate::path::exists());

    temp.close().expect("close temp dir");
}

#[test]
fn toc_fails_when_marker_is_missing() {
    let temp = assert_fs::TempDir::new().expect("temp dir");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .arg("toc")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));

    temp.close().expect("close temp dir");
}

#[test]
fn toc_fails_when_marker_points_to_missing_directory() {
    let temp = assert_fs::TempDir::new().expect("temp dir");
    temp.child(".adr-directory")
        .write_str("docs/adr\n")
        .expect("write marker");

    let mut cmd = Command::cargo_bin("adr").expect("binary exists");
    cmd.current_dir(temp.path())
        .arg("toc")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));

    temp.close().expect("close temp dir");
}
