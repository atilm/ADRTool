use crate::id::format_id;
use crate::resolver;
use crate::toc;
use chrono::Local;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use thiserror::Error;

const TEMPLATE_FILE: &str = ".adr-template";

#[derive(Debug, Error)]
pub enum NewError {
    #[error("failed to resolve ADR directory: {0}")]
    ResolveAdrDirectory(#[source] resolver::ResolveError),
    #[error("failed to read template file '{path}': {source}")]
    ReadTemplate {
        path: String,
        source: std::io::Error,
    },
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
    #[error("ADR id overflow while calculating next id after '{max}'")]
    IdOverflow { max: String },
    #[error("invalid ADR title '{title}': {reason}")]
    InvalidTitle { title: String, reason: String },
    #[error("failed to create ADR file '{path}': {source}")]
    CreateAdrFile {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to trigger TOC regeneration: {0}")]
    TocTrigger(#[source] TocTriggerError),
}

#[derive(Debug, Error)]
#[error("TOC regeneration hook failed")]
pub struct TocTriggerError;

pub fn run(title: &str) -> Result<(), NewError> {
    let resolved = resolver::resolve_current_dir().map_err(NewError::ResolveAdrDirectory)?;
    let template_path = resolved.adr_directory.join(TEMPLATE_FILE);
    let template =
        fs::read_to_string(template_path.as_path()).map_err(|source| NewError::ReadTemplate {
            path: path_string(template_path.as_path()),
            source,
        })?;

    let next_id = next_adr_id(resolved.adr_directory.as_path())?;
    let id_string = format_id(next_id);
    let normalized_title = normalize_title_for_filename(title)?;
    let date = current_local_date();

    let file_name = format!("{id_string}-{normalized_title}.md");
    let file_path = resolved.adr_directory.join(file_name);
    let content = render_template(template.as_str(), id_string.as_str(), title, date.as_str());

    create_file(file_path.as_path(), content.as_str())?;
    trigger_toc_regeneration(resolved.adr_directory.as_path()).map_err(NewError::TocTrigger)?;

    Ok(())
}

fn create_file(path: &Path, content: &str) -> Result<(), NewError> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|source| NewError::CreateAdrFile {
            path: path_string(path),
            source,
        })?;

    file.write_all(content.as_bytes())
        .map_err(|source| NewError::CreateAdrFile {
            path: path_string(path),
            source,
        })
}

fn trigger_toc_regeneration(_adr_directory: &Path) -> Result<(), TocTriggerError> {
    toc::run_for_directory(_adr_directory).map_err(|_| TocTriggerError)?;
    Ok(())
}

fn next_adr_id(adr_directory: &Path) -> Result<u32, NewError> {
    let entries = fs::read_dir(adr_directory).map_err(|source| NewError::ReadAdrDirectory {
        path: path_string(adr_directory),
        source,
    })?;

    let mut max_id = 0u32;
    for entry_result in entries {
        let entry = entry_result.map_err(|source| NewError::ReadAdrDirectoryEntry {
            path: path_string(adr_directory),
            source,
        })?;

        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if let Some(id) = extract_id_from_filename(file_name.as_ref()) {
            max_id = max_id.max(id);
        }
    }

    max_id.checked_add(1).ok_or(NewError::IdOverflow {
        max: format_id(max_id),
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

fn normalize_title_for_filename(title: &str) -> Result<String, NewError> {
    if title.trim().is_empty() {
        return Err(NewError::InvalidTitle {
            title: title.to_string(),
            reason: "title must not be empty".to_string(),
        });
    }

    if title.ends_with(' ') || title.ends_with('.') {
        return Err(NewError::InvalidTitle {
            title: title.to_string(),
            reason: "title must not end with a space or dot".to_string(),
        });
    }

    if let Some(invalid_char) = first_invalid_title_char(title) {
        return Err(NewError::InvalidTitle {
            title: title.to_string(),
            reason: format!("character '{invalid_char}' is not allowed in file names"),
        });
    }

    Ok(title.replace(' ', "-"))
}

fn first_invalid_title_char(title: &str) -> Option<char> {
    title.chars().find(|c| {
        c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
    })
}

fn render_template(template: &str, id: &str, title: &str, date: &str) -> String {
    template
        .replace("`ID`", id)
        .replace("`TITLE`", title)
        .replace("`YYYY-MM-DD`", date)
        .replace("`STATUS`", "DRAFT")
        .replace("`AUTHOR`", "")
        .replace("`LABELS`", "")
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
    use assert_fs::TempDir;
    use assert_fs::prelude::*;

    #[test]
    fn extracts_id_from_valid_filename() {
        assert_eq!(extract_id_from_filename("0005-Decision-X.md"), Some(5));
    }

    #[test]
    fn ignores_non_adr_filenames_when_extracting_id() {
        assert_eq!(extract_id_from_filename("adr-overview.md"), None);
        assert_eq!(extract_id_from_filename("foo.txt"), None);
        assert_eq!(extract_id_from_filename("-missing-id.md"), None);
    }

    #[test]
    fn next_id_uses_max_plus_one_policy() {
        let temp = TempDir::new().expect("temp dir");
        temp.child("0001-first.md").write_str("a").expect("write");
        temp.child("0002-second.md").write_str("a").expect("write");
        temp.child("0004-fourth.md").write_str("a").expect("write");

        let next = next_adr_id(temp.path()).expect("next id");
        assert_eq!(next, 5);

        temp.close().expect("close temp dir");
    }

    #[test]
    fn normalize_title_replaces_spaces_with_dashes() {
        let normalized = normalize_title_for_filename("Decision X").expect("normalized");
        assert_eq!(normalized, "Decision-X");
    }

    #[test]
    fn normalize_title_rejects_invalid_characters() {
        let err = normalize_title_for_filename("Decision: X").expect_err("invalid title");
        match err {
            NewError::InvalidTitle { reason, .. } => {
                assert!(reason.contains(":"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn render_template_replaces_expected_placeholders() {
        let template = "# ADR`ID` - `TITLE`\n* Date: `YYYY-MM-DD`\n* Status: `STATUS`\n";
        let rendered = render_template(template, "005", "Decision X", "2026-07-05");

        assert!(rendered.contains("# ADR005 - Decision X"));
        assert!(rendered.contains("* Date: 2026-07-05"));
        assert!(rendered.contains("* Status: DRAFT"));
    }

    #[test]
    fn local_date_has_expected_shape() {
        let date = current_local_date();
        assert_eq!(date.len(), 10);
        assert_eq!(date.chars().nth(4), Some('-'));
        assert_eq!(date.chars().nth(7), Some('-'));
    }
}
