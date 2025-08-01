use std::path::PathBuf;

/// Configuration for Ghost application
#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    pub log_dir: PathBuf,
    pub db_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = get_data_dir();
        let log_dir = data_dir.join("logs");
        let db_path = data_dir.join("tasks.db");

        Config {
            data_dir,
            log_dir,
            db_path,
        }
    }
}

impl Config {
    /// Create a new config with custom data directory
    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        let log_dir = data_dir.join("logs");
        let db_path = data_dir.join("tasks.db");

        Config {
            data_dir,
            log_dir,
            db_path,
        }
    }

    /// Ensure all required directories exist
    pub fn ensure_directories(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(&self.log_dir)?;
        Ok(())
    }

    /// Get the database path for this config
    pub fn get_db_path(&self) -> PathBuf {
        self.db_path.clone()
    }
}

/// Get the default data directory for Ghost
pub fn get_data_dir() -> PathBuf {
    // Priority 1: GHOST_DATA_DIR environment variable
    if let Ok(data_dir) = std::env::var("GHOST_DATA_DIR") {
        return PathBuf::from(data_dir);
    }

    // Priority 2: GHOST_HOME environment variable
    if let Ok(ghost_home) = std::env::var("GHOST_HOME") {
        return PathBuf::from(ghost_home);
    }

    // Priority 3: XDG_DATA_HOME (Linux/Unix)
    #[cfg(target_os = "linux")]
    if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg_data).join("ghost");
    }

    // Priority 4: OS-specific default locations
    #[cfg(target_os = "linux")]
    if let Some(home) = dirs::home_dir() {
        return home.join(".local").join("share").join("ghost");
    }

    // macOS and Windows: Use platform-specific dirs
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ghost")
}

/// Get the default log directory
pub fn get_log_dir() -> PathBuf {
    // Option to use cache directory for logs
    if let Ok(use_cache) = std::env::var("GHOST_LOGS_IN_CACHE") {
        if use_cache == "true" || use_cache == "1" {
            return get_cache_dir().join("logs");
        }
    }

    get_data_dir().join("logs")
}

/// Get the cache directory for temporary files
pub fn get_cache_dir() -> PathBuf {
    // Priority 1: GHOST_CACHE_DIR environment variable
    if let Ok(cache_dir) = std::env::var("GHOST_CACHE_DIR") {
        return PathBuf::from(cache_dir);
    }

    // Priority 2: XDG_CACHE_HOME (Linux/Unix)
    #[cfg(target_os = "linux")]
    if let Ok(xdg_cache) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(xdg_cache).join("ghost");
    }

    // Priority 3: OS-specific cache locations
    #[cfg(target_os = "linux")]
    if let Some(home) = dirs::home_dir() {
        return home.join(".cache").join("ghost");
    }

    // macOS and Windows: Use cache dir
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ghost")
}

/// Get the default database path
pub fn get_db_path() -> PathBuf {
    get_data_dir().join("tasks.db")
}

/// Environment variable parsing utilities
pub mod env {
    use crate::app::error::{GhostError, Result};

    /// Parse environment variables from KEY=VALUE format
    pub fn parse_env_vars(env_strings: &[String]) -> Result<Vec<(String, String)>> {
        let mut env_vars = Vec::new();
        for env_str in env_strings {
            if let Some((key, value)) = env_str.split_once('=') {
                env_vars.push((key.to_string(), value.to_string()));
            } else {
                return Err(GhostError::InvalidArgument {
                    message: format!(
                        "Invalid environment variable format: {env_str}. Use KEY=VALUE"
                    ),
                });
            }
        }
        Ok(env_vars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.data_dir.ends_with("ghost"));
        assert!(config.log_dir.ends_with("logs"));
        assert!(config.db_path.ends_with("tasks.db"));
    }

    #[test]
    fn test_config_with_custom_dir() {
        let temp_dir = tempdir().unwrap();
        let config = Config::with_data_dir(temp_dir.path().to_path_buf());

        assert_eq!(config.data_dir, temp_dir.path());
        assert_eq!(config.log_dir, temp_dir.path().join("logs"));
        assert_eq!(config.db_path, temp_dir.path().join("tasks.db"));
    }

    #[test]
    fn test_ensure_directories() {
        let temp_dir = tempdir().unwrap();
        let config = Config::with_data_dir(temp_dir.path().join("ghost"));

        config.ensure_directories().unwrap();

        assert!(config.data_dir.exists());
        assert!(config.log_dir.exists());
    }

    #[test]
    fn test_parse_env_vars_valid() {
        let env_strings = vec!["KEY1=value1".to_string(), "KEY2=value2".to_string()];

        let result = env::parse_env_vars(&env_strings).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("KEY1".to_string(), "value1".to_string()));
        assert_eq!(result[1], ("KEY2".to_string(), "value2".to_string()));
    }

    #[test]
    fn test_parse_env_vars_invalid() {
        let env_strings = vec!["INVALID_FORMAT".to_string()];

        let result = env::parse_env_vars(&env_strings);
        assert!(result.is_err());
    }
}
