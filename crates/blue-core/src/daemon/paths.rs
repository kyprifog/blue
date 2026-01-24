//! Daemon filesystem paths
//!
//! Handles platform-specific paths for daemon state.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PathError {
    #[error("Could not determine home directory")]
    NoHomeDir,

    #[error("Could not determine runtime directory")]
    NoRuntimeDir,

    #[error("Failed to create directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// Paths used by the Blue daemon
#[derive(Debug, Clone)]
pub struct DaemonPaths {
    /// Base directory: ~/.blue/
    pub base: PathBuf,

    /// Database file: ~/.blue/daemon.db
    pub database: PathBuf,

    /// Realm clones directory: ~/.blue/realms/
    pub realms: PathBuf,

    /// Runtime directory for PID file
    /// macOS: /var/run/blue/ or ~/Library/Caches/blue/
    /// Linux: $XDG_RUNTIME_DIR/blue/ or /tmp/blue-{uid}/
    pub runtime: PathBuf,

    /// PID file path
    pub pid_file: PathBuf,
}

impl DaemonPaths {
    /// Create paths, ensuring directories exist
    pub fn new() -> Result<Self, PathError> {
        let home = dirs::home_dir().ok_or(PathError::NoHomeDir)?;
        let base = home.join(".blue");
        let database = base.join("daemon.db");
        let realms = base.join("realms");

        // Runtime directory varies by platform
        let runtime = Self::runtime_dir()?;
        let pid_file = runtime.join("blue.pid");

        let paths = Self {
            base,
            database,
            realms,
            runtime,
            pid_file,
        };

        Ok(paths)
    }

    /// Ensure all directories exist
    pub fn ensure_dirs(&self) -> Result<(), PathError> {
        for dir in [&self.base, &self.realms, &self.runtime] {
            if !dir.exists() {
                std::fs::create_dir_all(dir).map_err(|e| PathError::CreateDir {
                    path: dir.clone(),
                    source: e,
                })?;
            }
        }
        Ok(())
    }

    /// Get the path for a specific realm's clone
    pub fn realm_path(&self, realm_name: &str) -> PathBuf {
        self.realms.join(realm_name)
    }

    /// Determine the runtime directory based on platform
    fn runtime_dir() -> Result<PathBuf, PathError> {
        // Try XDG_RUNTIME_DIR first (Linux)
        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            return Ok(PathBuf::from(xdg).join("blue"));
        }

        // macOS: Use ~/Library/Caches/blue for runtime
        #[cfg(target_os = "macos")]
        {
            if let Some(home) = dirs::home_dir() {
                return Ok(home.join("Library/Caches/blue"));
            }
        }

        // Fallback: Use cache directory
        if let Some(cache) = dirs::cache_dir() {
            return Ok(cache.join("blue"));
        }

        Err(PathError::NoRuntimeDir)
    }
}

impl Default for DaemonPaths {
    fn default() -> Self {
        Self::new().expect("Failed to determine daemon paths")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths_creation() {
        let paths = DaemonPaths::new().unwrap();
        assert!(paths.base.ends_with(".blue"));
        assert!(paths.database.ends_with("daemon.db"));
        assert!(paths.realms.ends_with("realms"));
    }
}
