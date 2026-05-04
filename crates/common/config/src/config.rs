use std::fs;
use std::path::Path;

use serde_yaml as yaml;

use crate::types::AppConfig;

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse YAML: {0}")]
    Yaml(#[from] yaml::Error),
}

/// Reads and parses a YAML config file from disk.
pub fn read_from_path(path: impl AsRef<Path>) -> Result<AppConfig, LoadError> {
    let contents = fs::read_to_string(path.as_ref())?;
    read_from_str(&contents)
}

/// Parses YAML config from a string (for tests or embedded fixtures).
pub fn read_from_str(contents: &str) -> Result<AppConfig, LoadError> {
    Ok(yaml::from_str(contents)?)
}
