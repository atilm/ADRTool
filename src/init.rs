use crate::template::ADR_TEMPLATE;
use pathdiff::diff_paths;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const MARKER_FILE: &str = ".adr-directory";
const TEMPLATE_FILE: &str = ".adr-template";
const STATUS_FILE: &str = ".adr-status";

const STATUS_CONTENT: &str = "DRAFT\nPROPOSED\nACCEPTED\nADOPTED\nSUPERSEDED\nEXPIRED\n";

#[derive(Debug, Error)]
pub enum InitError {
    #[error("failed to determine current working directory: {0}")]
    CurrentDirectory(std::io::Error),
    #[error("failed to create ADR directory '{path}': {source}")]
    CreateAdrDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to write marker file '{path}': {source}")]
    WriteMarker {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to create template file '{path}': {source}")]
    CreateTemplate {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to create status file '{path}': {source}")]
    CreateStatus {
        path: String,
        source: std::io::Error,
    },
    #[error("unable to compute relative path for marker file")]
    RelativePath,
}

pub fn run(adr_directory: &Path) -> Result<(), InitError> {
    let cwd = std::env::current_dir().map_err(InitError::CurrentDirectory)?;
    let abs_adr_dir = absolute_from(cwd.as_path(), adr_directory);
    let marker_value = marker_value_for(cwd.as_path(), abs_adr_dir.as_path())?;

    fs::create_dir_all(abs_adr_dir.as_path()).map_err(|source| InitError::CreateAdrDirectory {
        path: abs_adr_dir.display().to_string(),
        source,
    })?;

    let marker_path = cwd.join(MARKER_FILE);
    fs::write(marker_path.as_path(), format!("{marker_value}\n")).map_err(|source| {
        InitError::WriteMarker {
            path: marker_path.display().to_string(),
            source,
        }
    })?;

    let template_path = abs_adr_dir.join(TEMPLATE_FILE);
    if !template_path.exists() {
        fs::write(template_path.as_path(), ADR_TEMPLATE).map_err(|source| {
            InitError::CreateTemplate {
                path: template_path.display().to_string(),
                source,
            }
        })?;
    }

    let status_path = abs_adr_dir.join(STATUS_FILE);
    if !status_path.exists() {
        fs::write(status_path.as_path(), status_content()).map_err(|source| {
            InitError::CreateStatus {
                path: status_path.display().to_string(),
                source,
            }
        })?;
    }

    Ok(())
}

fn absolute_from(cwd: &Path, input: &Path) -> PathBuf {
    if input.is_absolute() {
        input.to_path_buf()
    } else {
        cwd.join(input)
    }
}

fn marker_value_for(cwd: &Path, abs_adr_dir: &Path) -> Result<String, InitError> {
    let relative = diff_paths(abs_adr_dir, cwd).ok_or(InitError::RelativePath)?;
    Ok(path_to_portable_string(relative.as_path()))
}

fn path_to_portable_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn status_content() -> &'static str {
    STATUS_CONTENT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_value_is_relative_for_nested_path() {
        let cwd = Path::new("/repo");
        let adr_path = Path::new("/repo/docs/adr");

        let value = marker_value_for(cwd, adr_path).expect("marker value");
        assert_eq!(value, "docs/adr");
    }

    #[test]
    fn marker_value_is_relative_for_parent_path() {
        let cwd = Path::new("/repo/subdir");
        let adr_path = Path::new("/repo/docs/adr");

        let value = marker_value_for(cwd, adr_path).expect("marker value");
        assert_eq!(value, "../docs/adr");
    }

    #[test]
    fn status_content_has_exact_values_and_order() {
        assert_eq!(
            status_content(),
            "DRAFT\nPROPOSED\nACCEPTED\nADOPTED\nSUPERSEDED\nEXPIRED\n"
        );
    }

    #[test]
    fn absolute_from_keeps_absolute_path() {
        let cwd = Path::new("/repo");
        let input = Path::new("/tmp/adrs");
        let absolute = absolute_from(cwd, input);
        assert_eq!(absolute, PathBuf::from("/tmp/adrs"));
    }
}
