use crate::resolver;
use chrono::NaiveDate;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use thiserror::Error;

const STATUS_FILE: &str = ".adr-status";
const OVERVIEW_FILE: &str = "adr-overview.md";

#[derive(Debug, Error)]
pub enum TocError {
    #[error("failed to resolve ADR directory: {0}")]
    ResolveAdrDirectory(#[source] resolver::ResolveError),
    #[error("failed to read ADR directory '{path}': {source}")]
    ReadAdrDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to read ADR directory entry in '{path}': {source}")]
    ReadAdrDirectoryEntry {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to read ADR file '{path}': {source}")]
    ReadAdrFile {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to read status file '{path}': {source}")]
    ReadStatusFile {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to write overview file '{path}': {source}")]
    WriteOverviewFile {
        path: String,
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocWarning {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocResult {
    pub warnings: Vec<TocWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AdrRecord {
    id: u32,
    file_name: String,
    title: Option<String>,
    date: Option<String>,
    status: Option<String>,
    author: Option<String>,
    labels: Option<String>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RenderStyle {
    Bold,
    Strike,
    Plain,
}

pub fn run() -> Result<TocResult, TocError> {
    let resolved = resolver::resolve_current_dir().map_err(TocError::ResolveAdrDirectory)?;
    run_for_directory(resolved.adr_directory.as_path())
}

pub fn run_for_directory(adr_directory: &Path) -> Result<TocResult, TocError> {
    let valid_statuses = read_statuses(adr_directory)?;
    let mut records = read_adr_records(adr_directory, valid_statuses.as_slice())?;
    records.sort_by_key(|record| record.id);

    let mut warnings = collect_sequence_warnings(records.as_slice());
    for record in &records {
        warnings.extend(record.warnings.iter().cloned());
    }

    let overview = render_overview(records.as_slice());
    let overview_path = adr_directory.join(OVERVIEW_FILE);
    fs::write(overview_path.as_path(), overview).map_err(|source| TocError::WriteOverviewFile {
        path: path_string(overview_path.as_path()),
        source,
    })?;

    Ok(TocResult {
        warnings: warnings
            .into_iter()
            .map(|message| TocWarning { message })
            .collect(),
    })
}

fn read_statuses(adr_directory: &Path) -> Result<Vec<String>, TocError> {
    let status_path = adr_directory.join(STATUS_FILE);
    let content =
        fs::read_to_string(status_path.as_path()).map_err(|source| TocError::ReadStatusFile {
            path: path_string(status_path.as_path()),
            source,
        })?;

    Ok(content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect())
}

fn read_adr_records(
    adr_directory: &Path,
    valid_statuses: &[String],
) -> Result<Vec<AdrRecord>, TocError> {
    let entries = fs::read_dir(adr_directory).map_err(|source| TocError::ReadAdrDirectory {
        path: path_string(adr_directory),
        source,
    })?;

    let mut records = Vec::new();
    for entry_result in entries {
        let entry = entry_result.map_err(|source| TocError::ReadAdrDirectoryEntry {
            path: path_string(adr_directory),
            source,
        })?;

        let file_name_os = entry.file_name();
        let file_name = file_name_os.to_string_lossy().to_string();
        let Some(file_id) = extract_id_from_filename(file_name.as_str()) else {
            continue;
        };

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let content =
            fs::read_to_string(path.as_path()).map_err(|source| TocError::ReadAdrFile {
                path: path_string(path.as_path()),
                source,
            })?;
        let record = parse_adr(
            file_id,
            file_name.as_str(),
            content.as_str(),
            valid_statuses,
        );
        records.push(record);
    }

    Ok(records)
}

fn parse_adr(file_id: u32, file_name: &str, content: &str, valid_statuses: &[String]) -> AdrRecord {
    let mut warnings = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let header_line = lines.first().copied();
    let (doc_id, title) = parse_header(header_line, file_name, &mut warnings);

    if let Some(parsed_doc_id) = doc_id
        && parsed_doc_id != file_id
    {
        warnings.push(format!(
            "ID mismatch in '{file_name}': file ID {file_id:03} but header ID {parsed_doc_id:03}"
        ));
    }

    let date = extract_metadata_value(lines.as_slice(), "Date", file_name, &mut warnings);
    let status = extract_metadata_value(lines.as_slice(), "Status", file_name, &mut warnings);
    let author = extract_metadata_value(lines.as_slice(), "Author", file_name, &mut warnings);
    let labels = extract_metadata_value(lines.as_slice(), "Labels", file_name, &mut warnings);

    validate_date(date.as_deref(), file_name, &mut warnings);
    validate_status(status.as_deref(), valid_statuses, file_name, &mut warnings);
    validate_author(author.as_deref(), file_name, &mut warnings);

    AdrRecord {
        id: file_id,
        file_name: file_name.to_string(),
        title,
        date,
        status,
        author,
        labels,
        warnings,
    }
}

fn parse_header(
    header: Option<&str>,
    file_name: &str,
    warnings: &mut Vec<String>,
) -> (Option<u32>, Option<String>) {
    let Some(header) = header else {
        warnings.push(format!("missing header line in '{file_name}'"));
        return (None, None);
    };

    let Some(rest) = header.strip_prefix("# ADR") else {
        warnings.push(format!(
            "invalid header format in '{file_name}': expected '# ADR<ID> - <TITLE>'"
        ));
        return (None, None);
    };

    let Some((id_part, title_part)) = rest.split_once(" - ") else {
        warnings.push(format!(
            "invalid header format in '{file_name}': expected '# ADR<ID> - <TITLE>'"
        ));
        return (None, None);
    };

    if id_part.is_empty() || !id_part.chars().all(|c| c.is_ascii_digit()) {
        warnings.push(format!("invalid ADR ID in header of '{file_name}'"));
        return (None, Some(title_part.trim().to_string()));
    }

    let doc_id = id_part.parse::<u32>().ok();
    if doc_id.is_none() {
        warnings.push(format!("invalid ADR ID in header of '{file_name}'"));
    }

    let title = title_part.trim();
    if title.is_empty() {
        warnings.push(format!("missing title value in header of '{file_name}'"));
        (doc_id, None)
    } else {
        (doc_id, Some(title.to_string()))
    }
}

fn extract_metadata_value(
    lines: &[&str],
    key: &str,
    file_name: &str,
    warnings: &mut Vec<String>,
) -> Option<String> {
    let prefix = format!("* {key}:");
    let value = lines.iter().find_map(|line| {
        line.strip_prefix(prefix.as_str())
            .map(str::trim)
            .map(ToString::to_string)
    });

    if value.is_none() {
        warnings.push(format!("missing metadata line '* {key}:' in '{file_name}'"));
    }

    value
}

fn validate_date(date: Option<&str>, file_name: &str, warnings: &mut Vec<String>) {
    let Some(date) = date else {
        return;
    };

    if NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err() {
        warnings.push(format!(
            "invalid date '{date}' in '{file_name}': expected YYYY-MM-DD"
        ));
    }
}

fn validate_status(
    status: Option<&str>,
    valid_statuses: &[String],
    file_name: &str,
    warnings: &mut Vec<String>,
) {
    let Some(status) = status else {
        return;
    };

    let base_status = status
        .split_once(" by ")
        .map_or(status, |(left, _)| left)
        .trim();

    let status_set: HashSet<&str> = valid_statuses.iter().map(String::as_str).collect();
    if !status_set.contains(base_status) {
        warnings.push(format!(
            "invalid status '{status}' in '{file_name}': not listed in .adr-status"
        ));
    }

    if base_status == "SUPERSEDED" && !is_valid_superseded_status(status) {
        warnings.push(format!(
            "invalid superseded status '{status}' in '{file_name}': expected 'SUPERSEDED by <id>'"
        ));
    }
}

fn validate_author(author: Option<&str>, file_name: &str, warnings: &mut Vec<String>) {
    let Some(author) = author else {
        return;
    };
    if author.trim().is_empty() {
        warnings.push(format!("empty author value in '{file_name}'"));
    }
}

fn is_valid_superseded_status(status: &str) -> bool {
    if status.trim() == "SUPERSEDED" {
        return true;
    }
    let Some((left, right)) = status.split_once(" by ") else {
        return false;
    };
    left.trim() == "SUPERSEDED"
        && !right.trim().is_empty()
        && right.trim().chars().all(|c| c.is_ascii_digit())
}

fn collect_sequence_warnings(records: &[AdrRecord]) -> Vec<String> {
    if records.is_empty() {
        return Vec::new();
    }

    let mut warnings = Vec::new();
    let mut ids: Vec<u32> = records.iter().map(|record| record.id).collect();
    ids.sort_unstable();
    ids.dedup();

    let first = ids[0];
    let last = *ids.last().expect("non-empty ids has a last");
    let id_set: HashSet<u32> = ids.iter().copied().collect();

    for expected in first..=last {
        if !id_set.contains(&expected) {
            warnings.push(format!("missing ADR ID in sequence: {expected:03}"));
        }
    }

    warnings
}

fn render_overview(records: &[AdrRecord]) -> String {
    let mut out = String::from("# ADR Overview\n\n");

    for record in records {
        let title = record.title.as_deref().unwrap_or("(missing title)");
        let title_text = format!("ADR{:03} - {}", record.id, title);
        let linked_title = format!("[{title_text}]({})", record.file_name);

        let status = record.status.as_deref().unwrap_or("UNKNOWN");
        let labels_text = normalize_labels(record.labels.as_deref().unwrap_or(""));
        let style = style_for_status(status);
        let decorated = match style {
            RenderStyle::Bold => format!("**{linked_title}**"),
            RenderStyle::Strike => format!("~~{linked_title}~~"),
            RenderStyle::Plain => linked_title,
        };

        let suffix = if status.trim() == "EXPIRED" {
            " expired".to_string()
        } else if let Some(superseding_id) = superseded_by_id(status) {
            format!(" superseded by {superseding_id}")
        } else {
            String::new()
        };

        out.push_str("- ");
        out.push_str(decorated.as_str());
        if !labels_text.is_empty() {
            out.push(' ');
            out.push_str(labels_text.as_str());
        }
        out.push_str(suffix.as_str());
        out.push('\n');
    }

    out
}

fn normalize_labels(raw_labels: &str) -> String {
    let labels: Vec<&str> = raw_labels
        .split(',')
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .collect();

    if labels.is_empty() {
        "".to_string()
    } else {
        format!("({})", labels.join(", "))
    }
}

fn superseded_by_id(status: &str) -> Option<&str> {
    let (left, right) = status.split_once(" by ")?;
    if left.trim() != "SUPERSEDED" {
        return None;
    }

    let superseding_id = right.trim();
    if superseding_id.is_empty() {
        None
    } else {
        Some(superseding_id)
    }
}

fn style_for_status(status: &str) -> RenderStyle {
    let normalized = status.trim();
    if normalized == "DRAFT" || normalized == "PROPOSED" {
        return RenderStyle::Bold;
    }
    if normalized == "EXPIRED" || normalized.starts_with("SUPERSEDED") {
        return RenderStyle::Strike;
    }
    RenderStyle::Plain
}

fn extract_id_from_filename(file_name: &str) -> Option<u32> {
    let stem = file_name.strip_suffix(".md")?;
    let (id_part, _) = stem.split_once('-')?;
    if id_part.is_empty() || !id_part.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    id_part.parse::<u32>().ok()
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;
    use assert_fs::prelude::*;

    fn default_statuses() -> Vec<String> {
        vec![
            "DRAFT".to_string(),
            "PROPOSED".to_string(),
            "ACCEPTED".to_string(),
            "ADOPTED".to_string(),
            "SUPERSEDED".to_string(),
            "EXPIRED".to_string(),
        ]
    }

    #[test]
    fn parse_adr_collects_header_and_metadata() {
        let content = "# ADR005 - Decision X\n\n* Date: 2026-07-05\n* Status: DRAFT\n* Author: Jane\n* Labels: api,auth\n";
        let record = parse_adr(
            5,
            "005-Decision-X.md",
            content,
            default_statuses().as_slice(),
        );

        assert_eq!(record.title.as_deref(), Some("Decision X"));
        assert_eq!(record.date.as_deref(), Some("2026-07-05"));
        assert_eq!(record.status.as_deref(), Some("DRAFT"));
        assert_eq!(record.author.as_deref(), Some("Jane"));
        assert_eq!(record.labels.as_deref(), Some("api,auth"));
        assert!(record.warnings.is_empty());
    }

    #[test]
    fn parse_adr_warns_for_missing_metadata_lines() {
        let content = "# ADR005 - Decision X\n";
        let record = parse_adr(
            5,
            "005-Decision-X.md",
            content,
            default_statuses().as_slice(),
        );

        assert!(
            record
                .warnings
                .iter()
                .any(|warning| warning.contains("Date"))
        );
        assert!(
            record
                .warnings
                .iter()
                .any(|warning| warning.contains("Status"))
        );
        assert!(
            record
                .warnings
                .iter()
                .any(|warning| warning.contains("Author"))
        );
        assert!(
            record
                .warnings
                .iter()
                .any(|warning| warning.contains("Labels"))
        );
    }

    #[test]
    fn parse_adr_warns_for_header_id_mismatch() {
        let content = "# ADR006 - Decision X\n\n* Date: 2026-07-05\n* Status: DRAFT\n* Author: Jane\n* Labels: a\n";
        let record = parse_adr(
            5,
            "005-Decision-X.md",
            content,
            default_statuses().as_slice(),
        );

        assert!(
            record
                .warnings
                .iter()
                .any(|warning| warning.contains("ID mismatch"))
        );
    }

    #[test]
    fn parse_adr_warns_for_invalid_date_status_and_author() {
        let content = "# ADR005 - Decision X\n\n* Date: 2026/07/05\n* Status: UNKNOWN\n* Author: \n* Labels: \n";
        let record = parse_adr(
            5,
            "005-Decision-X.md",
            content,
            default_statuses().as_slice(),
        );

        assert!(
            record
                .warnings
                .iter()
                .any(|warning| warning.contains("invalid date"))
        );
        assert!(
            record
                .warnings
                .iter()
                .any(|warning| warning.contains("invalid status"))
        );
        assert!(
            record
                .warnings
                .iter()
                .any(|warning| warning.contains("empty author"))
        );
    }

    #[test]
    fn parse_adr_warns_for_empty_author_value() {
        let content = "# ADR005 - Decision X\n\n* Date: 2026-07-05\n* Status: ACCEPTED\n* Author: \n* Labels: core\n";
        let record = parse_adr(
            5,
            "005-Decision-X.md",
            content,
            default_statuses().as_slice(),
        );

        assert!(
            record
                .warnings
                .iter()
                .any(|warning| warning.contains("empty author value"))
        );
    }

    #[test]
    fn parse_adr_accepts_superseded_by_id_status() {
        let content = "# ADR005 - Decision X\n\n* Date: 2026-07-05\n* Status: SUPERSEDED by 006\n* Author: Jane\n* Labels: \n";
        let record = parse_adr(
            5,
            "005-Decision-X.md",
            content,
            default_statuses().as_slice(),
        );

        assert!(
            !record
                .warnings
                .iter()
                .any(|warning| warning.contains("superseded status"))
        );
    }

    #[test]
    fn collect_sequence_warnings_detects_missing_ids() {
        let records = vec![
            AdrRecord {
                id: 1,
                file_name: "001-a.md".to_string(),
                title: Some("A".to_string()),
                date: None,
                status: None,
                author: None,
                labels: None,
                warnings: Vec::new(),
            },
            AdrRecord {
                id: 2,
                file_name: "002-b.md".to_string(),
                title: Some("B".to_string()),
                date: None,
                status: None,
                author: None,
                labels: None,
                warnings: Vec::new(),
            },
            AdrRecord {
                id: 4,
                file_name: "004-c.md".to_string(),
                title: Some("C".to_string()),
                date: None,
                status: None,
                author: None,
                labels: None,
                warnings: Vec::new(),
            },
        ];

        let warnings = collect_sequence_warnings(records.as_slice());
        assert_eq!(warnings, vec!["missing ADR ID in sequence: 003"]);
    }

    #[test]
    fn render_overview_applies_required_styles_and_labels() {
        let records = vec![
            AdrRecord {
                id: 1,
                file_name: "001-a.md".to_string(),
                title: Some("A".to_string()),
                date: Some("2026-07-01".to_string()),
                status: Some("DRAFT".to_string()),
                author: Some("J".to_string()),
                labels: Some("api".to_string()),
                warnings: Vec::new(),
            },
            AdrRecord {
                id: 2,
                file_name: "002-b.md".to_string(),
                title: Some("B".to_string()),
                date: Some("2026-07-01".to_string()),
                status: Some("SUPERSEDED by 003".to_string()),
                author: Some("J".to_string()),
                labels: Some("security".to_string()),
                warnings: Vec::new(),
            },
            AdrRecord {
                id: 3,
                file_name: "003-c.md".to_string(),
                title: Some("C".to_string()),
                date: Some("2026-07-01".to_string()),
                status: Some("EXPIRED".to_string()),
                author: Some("J".to_string()),
                labels: Some("".to_string()),
                warnings: Vec::new(),
            },
        ];

        let out = render_overview(records.as_slice());

        assert!(out.contains("- **[ADR001 - A](001-a.md)** (api)"));
        assert!(out.contains("- ~~[ADR002 - B](002-b.md)~~ (security) superseded by 003"));
        assert!(out.contains("- ~~[ADR003 - C](003-c.md)~~ expired"));
    }

    #[test]
    fn run_for_directory_generates_overview_and_warnings() {
        let temp = TempDir::new().expect("temp dir");
        temp.child(".adr-status")
            .write_str("DRAFT\nPROPOSED\nACCEPTED\nADOPTED\nSUPERSEDED\nEXPIRED\n")
            .expect("write status");

        temp.child("001-A.md")
            .write_str(
                "# ADR001 - A\n\n* Date: 2026-07-01\n* Status: DRAFT\n* Author: Jane\n* Labels: api\n",
            )
            .expect("write adr");
        temp.child("003-C.md")
            .write_str(
                "# ADR004 - C\n\n* Date: 2026-07-01\n* Status: ACCEPTED\n* Author: Jane\n* Labels: core\n",
            )
            .expect("write adr");

        let result = run_for_directory(temp.path()).expect("run toc");

        temp.child("adr-overview.md")
            .assert(predicates::path::exists());
        assert!(
            result
                .warnings
                .iter()
                .any(|warning| warning.message.contains("missing ADR ID in sequence: 002"))
        );
        assert!(
            result
                .warnings
                .iter()
                .any(|warning| warning.message.contains("ID mismatch"))
        );

        temp.close().expect("close temp");
    }
}
