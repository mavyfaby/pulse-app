// Pulse — Bayanihan Emergency Network
// Copyright (C) 2026 Maverick Fabroa (@mavyfaby)
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{collections::HashMap, env, path::Path, str::FromStr};

use thiserror::Error;
use tracing::info;

#[derive(Debug)]
pub struct AppConfig {
    pub tcp: TcpConfig,
}

#[derive(Debug)]
pub struct TcpConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub read_timeout_seconds: u64,
}

#[derive(Error, Debug, PartialEq)]
pub enum ConfigError {
    #[error("Config error: {0}")]
    GeneralError(String),

    #[error("Environment variable {0} is not set")]
    EnvVarNotSet(String),

    #[error("Failed to parse environment variable {0}")]
    EnvVarParseError(String),
}

/// Loads `AppConfig` from the env file referenced by `PULSE_ENV_FILE`.
pub fn load() -> Result<AppConfig, ConfigError> {
    // Get the path to the env file
    let env_file = env::var("PULSE_ENV_FILE")
        .map_err(|_| ConfigError::EnvVarNotSet("PULSE_ENV_FILE".to_string()))?;

    // Construct the path
    let env_file_path = env::current_dir()
        .map_err(|err| ConfigError::GeneralError(err.to_string()))?
        .join(env_file);

    // Load config from the path
    load_from(&env_file_path)
}

/// Loads `AppConfig` from a given env file path without touching process environment.
pub fn load_from(path: &Path) -> Result<AppConfig, ConfigError> {
    // Load the env file from the provided path
    let vars: HashMap<String, String> = dotenvy::from_path_iter(path)
        .map_err(|e| ConfigError::GeneralError(e.to_string()))?
        .collect::<Result<_, _>>()
        .map_err(|e| ConfigError::GeneralError(e.to_string()))?;

    // Log the .env file path
    info!("Loaded env file from {}", path.display());

    // Construct the config and return it
    Ok(AppConfig {
        tcp: TcpConfig {
            host: required(&vars, "PULSE_TCP_HOST")?,
            port: required::<u16>(&vars, "PULSE_TCP_PORT")?,
            max_connections: required::<usize>(&vars, "PULSE_TCP_MAX_CONNECTIONS")?,
            read_timeout_seconds: required::<u64>(&vars, "PULSE_TCP_READ_TIMEOUT_SECONDS")?,
        },
    })
}

/// Get a required env var, returning an error if not present or parseable.
fn required<T: FromStr>(vars: &HashMap<String, String>, name: &str) -> Result<T, ConfigError> {
    vars.get(name)
        .ok_or_else(|| ConfigError::EnvVarNotSet(name.to_string()))?
        .parse::<T>()
        .map_err(|_| ConfigError::EnvVarParseError(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn loads_config_from_env_file() {
        let dir = tempdir().unwrap();
        let env_path = dir.path().join("test.env");
        fs::write(
            &env_path,
            "PULSE_TCP_HOST=127.0.0.1\nPULSE_TCP_PORT=8080\nPULSE_TCP_MAX_CONNECTIONS=1024\nPULSE_TCP_READ_TIMEOUT_SECONDS=30\n",
        )
        .unwrap();

        let cfg = load_from(&env_path).expect("should load");
        assert_eq!(cfg.tcp.host, "127.0.0.1");
        assert_eq!(cfg.tcp.port, 8080);
        assert_eq!(cfg.tcp.max_connections, 1024);
        assert_eq!(cfg.tcp.read_timeout_seconds, 30);
    }

    #[test]
    fn errors_when_env_file_path_missing() {
        let dir = tempdir().unwrap();
        let env_path = dir.path().join("does_not_exist.env");

        match load_from(&env_path) {
            Err(ConfigError::GeneralError(_)) => {}
            other => panic!("expected GeneralError, got {:?}", other),
        }
    }

    #[test]
    fn errors_when_required_var_missing() {
        let dir = tempdir().unwrap();
        let env_path = dir.path().join("missing_host.env");

        // PULSE_TCP_HOST intentionally omitted
        fs::write(&env_path, "PULSE_TCP_PORT=8080\n").unwrap();

        match load_from(&env_path) {
            Err(ConfigError::EnvVarNotSet(name)) => assert_eq!(name, "PULSE_TCP_HOST"),
            other => panic!("expected EnvVarNotSet for PULSE_TCP_HOST, got {:?}", other),
        }
    }

    #[test]
    fn errors_when_port_unparseable() {
        let dir = tempdir().unwrap();
        let env_path = dir.path().join("bad_port.env");
        fs::write(
            &env_path,
            "PULSE_TCP_HOST=127.0.0.1\nPULSE_TCP_PORT=not_a_number\nPULSE_TCP_MAX_CONNECTIONS=1024\nPULSE_TCP_READ_TIMEOUT_SECONDS=30\n",
        )
        .unwrap();

        match load_from(&env_path) {
            Err(ConfigError::EnvVarParseError(name)) => assert_eq!(name, "PULSE_TCP_PORT"),
            other => panic!(
                "expected EnvVarParseError for PULSE_TCP_PORT, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn errors_when_port_out_of_range() {
        let dir = tempdir().unwrap();
        let env_path = dir.path().join("big_port.env");

        // 70000 overflows u16
        fs::write(
            &env_path,
            "PULSE_TCP_HOST=127.0.0.1\nPULSE_TCP_PORT=70000\nPULSE_TCP_MAX_CONNECTIONS=1024\nPULSE_TCP_READ_TIMEOUT_SECONDS=30\n",
        )
        .unwrap();

        match load_from(&env_path) {
            Err(ConfigError::EnvVarParseError(name)) => assert_eq!(name, "PULSE_TCP_PORT"),
            other => panic!(
                "expected EnvVarParseError for PULSE_TCP_PORT, got {:?}",
                other
            ),
        }
    }
}
