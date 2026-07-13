use crate::id::format_id;
use crate::resolver;
use crate::toc;
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const DATE_PREFIX: &str = "* Date:";
const STATUS_PREFIX: &str = "* Status:";

#[derive(Debug, Error)]
pub enum ModError {
    #[error("at least one modifier must be provided: use '-a' or '-s <id>'")]
    MissingAction,
    #[error("cannot supersede ADR '{id}' with itself")]
    SupersedeSelf { id: String },
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
    #[error("ADR '{id}' not found in directory '{directory}'")]
    AdrNotFound { id: String, directory: String },
    #[error("multiple ADR files found for id '{id}': '{first}' and '{second}'")]
    DuplicateAdrId {
        id: String,
        first: String,
        second: String,
    },
    #[error("failed to read ADR file '{path}': {source}")]
    ReadAdrFile {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to write ADR file '{path}': {source}")]
    WriteAdrFile {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to trigger TOC regeneration: {0}")]
    TocTrigger(#[source] TocTriggerError),
}

#[derive(Debug, Error)]
#[error("TOC regeneration hook failed")]
pub struct TocTriggerError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModResult {
    pub warnings: Vec<String>,
}

struct UpdateRequest<'a> {
    new_status: &'a str,
    update_date: bool,
    date: &'a str,
}

struct UpdateOutcome {
    content: String,
    warnings: Vec<String>,
}

pub fn run(id: u32, accept: bool, supersede: Option<u32>) -> Result<ModResult, ModError> {
    if !accept && supersede.is_none() {
        return Err(ModError::MissingAction);
    }
    if let Some(superseded_id) = supersede
        && superseded_id == id
    {
        return Err(ModError::SupersedeSelf { id: format_id(id) });
    }

    let resolved = resolver::resolve_current_dir().map_err(ModError::ResolveAdrDirectory)?;
    let date = current_local_date();

    let target_path = find_adr_file_by_id(resolved.adr_directory.as_path(), id)?;
    let target_original = read_file(target_path.as_path())?;
    let target_update = apply_update(
        target_original.as_str(),
        &UpdateRequest {
            new_status: "ACCEPTED",
            update_date: true,
            date: date.as_str(),
        },
    );

    let mut warnings = prefix_warnings(
        file_name_for_warning(target_path.as_path()).as_str(),
        target_update.warnings,
    );

    if let Some(superseded_id) = supersede {
        let superseded_path = find_adr_file_by_id(resolved.adr_directory.as_path(), superseded_id)?;
        let superseded_original = read_file(superseded_path.as_path())?;
        let superseded_update = apply_update(
            superseded_original.as_str(),
            &UpdateRequest {
                new_status: format!("SUPERSEDED by {}", format_id(id)).as_str(),
                update_date: false,
                date: date.as_str(),
            },
        );

        warnings.extend(prefix_warnings(
            file_name_for_warning(superseded_path.as_path()).as_str(),
            superseded_update.warnings,
        ));

        write_file(target_path.as_path(), target_update.content.as_str())?;
        write_file(
            superseded_path.as_path(),
            superseded_update.content.as_str(),
        )?;
    } else {
        write_file(target_path.as_path(), target_update.content.as_str())?;
    }

    let toc_result = toc::run_for_directory(resolved.adr_directory.as_path())
        .map_err(|_| ModError::TocTrigger(TocTriggerError))?;
    warnings.extend(
        toc_result
            .warnings
            .into_iter()
            .map(|warning| warning.message),
    );

    Ok(ModResult { warnings })
}

fn read_file(path: &Path) -> Result<String, ModError> {
    fs::read_to_string(path).map_err(|source| ModError::ReadAdrFile {
        path: path_string(path),
        source,
    })
}

fn write_file(path: &Path, content: &str) -> Result<(), ModError> {
    fs::write(path, content).map_err(|source| ModError::WriteAdrFile {
        path: path_string(path),
        source,
    })
}

fn find_adr_file_by_id(adr_directory: &Path, target_id: u32) -> Result<PathBuf, ModError> {
    let entries = fs::read_dir(adr_directory).map_err(|source| ModError::ReadAdrDirectory {
        path: path_string(adr_directory),
        source,
    })?;

    let mut match_path: Option<PathBuf> = None;
    for entry_result in entries {
        let entry = entry_result.map_err(|source| ModError::ReadAdrDirectoryEntry {
            path: path_string(adr_directory),
            source,
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        let Some(id) = extract_id_from_filename(file_name.as_ref()) else {
            continue;
        };
        if id != target_id {
            continue;
        }

        if let Some(existing) = &match_path {
            return Err(ModError::DuplicateAdrId {
                id: format_id(target_id),
                first: file_name_for_warning(existing.as_path()),
                second: file_name.to_string(),
            });
        }
        match_path = Some(path);
    }

    match_path.ok_or_else(|| ModError::AdrNotFound {
        id: format_id(target_id),
        directory: path_string(adr_directory),
    })
}

fn extract_id_from_filename(file_name: &str) -> Option<u32> {
    let stem = file_name.strip_suffix(".md")?;
    let (id_part, _) = stem.split_once('-')?;
    if id_part.is_empty() || !id_part.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    id_part.parse::<u32>().ok()
}

fn apply_update(content: &str, request: &UpdateRequest<'_>) -> UpdateOutcome {
    let mut lines: Vec<String> = content.lines().map(ToString::to_string).collect();
    let mut warnings = Vec::new();

    if request.update_date {
        upsert_metadata_line(
            &mut lines,
            DATE_PREFIX,
            format!("{DATE_PREFIX} {}", request.date).as_str(),
            "Date",
            &mut warnings,
        );
    }

    upsert_metadata_line(
        &mut lines,
        STATUS_PREFIX,
        format!("{STATUS_PREFIX} {}", request.new_status).as_str(),
        "Status",
        &mut warnings,
    );

    let mut updated = lines.join("\n");
    if content.ends_with('\n') || !updated.is_empty() {
        updated.push('\n');
    }

    UpdateOutcome {
        content: updated,
        warnings,
    }
}

fn upsert_metadata_line(
    lines: &mut Vec<String>,
    prefix: &str,
    replacement: &str,
    key: &str,
    warnings: &mut Vec<String>,
) {
    if let Some(index) = lines.iter().position(|line| line.starts_with(prefix)) {
        lines[index] = replacement.to_string();
        return;
    }

    let insert_index = metadata_insert_index(lines.as_slice());
    lines.insert(insert_index, replacement.to_string());
    warnings.push(format!("missing metadata line '* {key}:' inserted"));
}

fn metadata_insert_index(lines: &[String]) -> usize {
    if lines.is_empty() {
        return 0;
    }

    let mut index = if lines.len() > 1 && lines[1].trim().is_empty() {
        2
    } else {
        1
    };

    while index < lines.len() {
        let line = lines[index].trim_start();
        if !line.starts_with("* ") {
            break;
        }
        index += 1;
    }

    index
}

fn prefix_warnings(file_name: &str, warnings: Vec<String>) -> Vec<String> {
    warnings
        .into_iter()
        .map(|warning| format!("{file_name}: {warning}"))
        .collect()
}

fn file_name_for_warning(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path_string(path))
}

fn current_local_date() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_id_from_filename_handles_valid_and_invalid_cases() {
        assert_eq!(extract_id_from_filename("0001-First.md"), Some(1));
        assert_eq!(extract_id_from_filename("1-First.md"), Some(1));
        assert_eq!(extract_id_from_filename("adr-overview.md"), None);
        assert_eq!(extract_id_from_filename("0001-First.txt"), None);
        assert_eq!(extract_id_from_filename("not-an-adr.md"), None);
    }

    #[test]
    fn apply_update_replaces_existing_status_and_date() {
        let original = "# ADR001 - Test\n\n* Date: 2020-01-01\n* Status: DRAFT\n* Author: Jane\n";
        let updated = apply_update(
            original,
            &UpdateRequest {
                new_status: "ACCEPTED",
                update_date: true,
                date: "2026-07-05",
            },
        );

        assert!(updated.content.contains("* Date: 2026-07-05"));
        assert!(updated.content.contains("* Status: ACCEPTED"));
        assert!(updated.warnings.is_empty());
    }

    #[test]
    fn apply_update_inserts_missing_status_and_date_with_warnings() {
        let original = "# ADR001 - Test\n\n* Author: Jane\n* Labels: core\n";
        let updated = apply_update(
            original,
            &UpdateRequest {
                new_status: "ACCEPTED",
                update_date: true,
                date: "2026-07-05",
            },
        );

        assert!(updated.content.contains("* Date: 2026-07-05"));
        assert!(updated.content.contains("* Status: ACCEPTED"));
        assert_eq!(updated.warnings.len(), 2);
        assert!(
            updated
                .warnings
                .iter()
                .any(|warning| warning.contains("* Date:"))
        );
        assert!(
            updated
                .warnings
                .iter()
                .any(|warning| warning.contains("* Status:"))
        );
    }

    #[test]
    fn apply_update_without_date_change_keeps_existing_date() {
        let original = "# ADR001 - Test\n\n* Date: 2020-01-01\n* Status: ACCEPTED\n";
        let updated = apply_update(
            original,
            &UpdateRequest {
                new_status: "SUPERSEDED by 002",
                update_date: false,
                date: "2026-07-05",
            },
        );

        assert!(updated.content.contains("* Date: 2020-01-01"));
        assert!(updated.content.contains("* Status: SUPERSEDED by 002"));
        assert!(updated.warnings.is_empty());
    }

    #[test]
    fn metadata_insert_index_places_new_lines_after_metadata_block() {
        let lines = vec![
            "# ADR001 - Test".to_string(),
            "".to_string(),
            "* Author: Jane".to_string(),
            "* Labels: core".to_string(),
            "".to_string(),
            "## Decision".to_string(),
        ];

        assert_eq!(metadata_insert_index(lines.as_slice()), 4);
    }
}
