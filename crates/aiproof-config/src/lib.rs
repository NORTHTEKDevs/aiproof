//! aiproof-config: Configuration loader with cascade support.

pub mod config;
pub mod error;

pub use config::Config;
pub use error::ConfigError;

use std::path::Path;

/// Cascade loader: checks for config files in order:
/// 1. `.aiproofrc` in `start` or any ancestor — standalone TOML, deserialized as `Config`.
/// 2. `pyproject.toml` in `start` or any ancestor — look for `[tool.aiproof]` table.
/// 3. `Config::default()`.
///
/// First file that matches (by walking upward from `start`) wins. We don't merge.
pub fn load(start: &Path) -> Result<Config, ConfigError> {
    for ancestor in start.ancestors() {
        let aiproofrc = ancestor.join(".aiproofrc");
        if aiproofrc.is_file() {
            return load_aiproofrc(&aiproofrc);
        }
        let pyproject = ancestor.join("pyproject.toml");
        if pyproject.is_file()
            && let Some(cfg) = try_pyproject(&pyproject)?
        {
            return Ok(cfg);
        }
    }
    Ok(Config::default())
}

fn load_aiproofrc(path: &Path) -> Result<Config, ConfigError> {
    let text = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.display().to_string(),
        source,
    })?;
    toml::from_str::<Config>(&text).map_err(|e| ConfigError::Toml {
        path: path.display().to_string(),
        message: e.to_string(),
    })
}

fn try_pyproject(path: &Path) -> Result<Option<Config>, ConfigError> {
    let text = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.display().to_string(),
        source,
    })?;
    let value: toml::Value = text
        .parse()
        .map_err(|e: toml::de::Error| ConfigError::Toml {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
    let Some(table) = value.get("tool").and_then(|t| t.get("aiproof")) else {
        return Ok(None);
    };
    let serialized = toml::to_string_pretty(table).map_err(|e| ConfigError::Toml {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;
    toml::from_str::<Config>(&serialized)
        .map(Some)
        .map_err(|e| ConfigError::Toml {
            path: path.display().to_string(),
            message: format!("in [tool.aiproof]: {e}"),
        })
}
