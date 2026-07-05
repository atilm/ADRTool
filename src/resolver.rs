use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const MARKER_FILE: &str = ".adr-directory";

#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("failed to determine current working directory: {0}")]
    CurrentDirectory(std::io::Error),
    #[error("unable to locate '{marker}' starting from '{start}'")]
    MarkerNotFound { start: String, marker: String },
    #[error("failed to read marker file '{path}': {source}")]
    ReadMarker {
        path: String,
        source: std::io::Error,
    },
    #[error("marker file '{path}' points to missing ADR directory '{resolved}'")]
    AdrDirectoryMissing { path: String, resolved: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAdrDirectory {
    pub marker_path: PathBuf,
    pub marker_directory: PathBuf,
    pub adr_directory: PathBuf,
}

pub(crate) fn resolve_current_dir() -> Result<ResolvedAdrDirectory, ResolveError> {
    let current_dir = std::env::current_dir().map_err(ResolveError::CurrentDirectory)?;
    resolve_from(current_dir.as_path())
}

pub(crate) fn resolve_from(start: &Path) -> Result<ResolvedAdrDirectory, ResolveError> {
    let marker_path = find_marker(start).ok_or_else(|| ResolveError::MarkerNotFound {
        start: path_string(start),
        marker: MARKER_FILE.to_string(),
    })?;
    resolve_marker_path(marker_path.as_path())
}

fn resolve_marker_path(marker_path: &Path) -> Result<ResolvedAdrDirectory, ResolveError> {
    let marker_directory = marker_path
        .parent()
        .expect("marker file path should always have a parent")
        .to_path_buf();
    let marker_contents = fs::read_to_string(marker_path).map_err(|source| {
        ResolveError::ReadMarker {
            path: path_string(marker_path),
            source,
        }
    })?;
    let relative_path = marker_contents.trim();
    let adr_directory = marker_directory.join(relative_path);

    if !adr_directory.exists() {
        return Err(ResolveError::AdrDirectoryMissing {
            path: path_string(marker_path),
            resolved: path_string(&adr_directory),
        });
    }

    Ok(ResolvedAdrDirectory {
        marker_path: marker_path.to_path_buf(),
        marker_directory,
        adr_directory,
    })
}

fn find_marker(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .map(|directory| directory.join(MARKER_FILE))
        .find(|marker_path| marker_path.exists())
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;

    #[test]
    fn resolves_marker_in_current_directory() {
        let temp = TempDir::new().expect("temp dir");
        let marker = temp.child(".adr-directory");
        marker.write_str("docs/adr\n").expect("write marker");
        temp.child("docs/adr").create_dir_all().expect("adr dir");

        let resolved = resolve_from(temp.path()).expect("resolved adr directory");

        assert_eq!(resolved.marker_path, marker.path());
        assert_eq!(resolved.marker_directory, temp.path());
        assert_eq!(resolved.adr_directory, temp.child("docs/adr").path());

        temp.close().expect("close temp dir");
    }

    #[test]
    fn resolves_marker_in_parent_directory() {
        let temp = TempDir::new().expect("temp dir");
        temp.child(".adr-directory")
            .write_str("docs/adr\n")
            .expect("write marker");
        temp.child("docs/adr").create_dir_all().expect("adr dir");
        let nested = temp.child("nested");
        nested.create_dir_all().expect("nested dir");

        let resolved = resolve_from(nested.path()).expect("resolved adr directory");

        assert_eq!(resolved.marker_path, temp.child(".adr-directory").path());
        assert_eq!(resolved.marker_directory, temp.path());
        assert_eq!(resolved.adr_directory, temp.child("docs/adr").path());

        temp.close().expect("close temp dir");
    }

    #[test]
    fn reports_missing_marker_when_none_found() {
        let temp = TempDir::new().expect("temp dir");
        let error = resolve_from(temp.path()).expect_err("missing marker should fail");

        match error {
            ResolveError::MarkerNotFound { start, marker } => {
                assert_eq!(start, path_string(temp.path()));
                assert_eq!(marker, MARKER_FILE);
            }
            other => panic!("unexpected error: {other:?}"),
        }

        temp.close().expect("close temp dir");
    }

    #[test]
    fn reports_missing_adr_directory_when_marker_points_to_absent_path() {
        let temp = TempDir::new().expect("temp dir");
        temp.child(".adr-directory")
            .write_str("docs/adr\n")
            .expect("write marker");

        let error = resolve_from(temp.path()).expect_err("missing adr dir should fail");

        match error {
            ResolveError::AdrDirectoryMissing { path, resolved } => {
                assert_eq!(path, path_string(temp.child(".adr-directory").path()));
                assert_eq!(resolved, path_string(temp.child("docs/adr").path()));
            }
            other => panic!("unexpected error: {other:?}"),
        }

        temp.close().expect("close temp dir");
    }

    #[test]
    fn trims_marker_contents_before_resolution() {
        let temp = TempDir::new().expect("temp dir");
        temp.child(".adr-directory")
            .write_str("docs/adr\n")
            .expect("write marker");
        temp.child("docs/adr").create_dir_all().expect("adr dir");

        let resolved = resolve_from(temp.path()).expect("resolved adr directory");

        assert_eq!(resolved.adr_directory, temp.child("docs/adr").path());

        temp.close().expect("close temp dir");
    }
}
