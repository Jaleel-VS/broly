use anyhow::{Context, Result};
use std::env;

/// Application configuration loaded from environment variables.
///
/// Required vars:
///   DISCORD_TOKEN     - Bot token (read directly via Token::from_env, not stored here)
///   DATABASE_URL      - Postgres connection string
///
/// Optional vars:
///   ENVIRONMENT       - "local" | "staging" | "prod" (default: "local")
///   COMMAND_PREFIX    - Primary text-command prefix (default: "rs.")
///   EXTRA_PREFIXES    - Comma-separated additional prefixes (e.g. "$" to also respond to $np)
#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub environment: String,
    pub command_prefix: String,
    pub extra_prefixes: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let extra_prefixes = env::var("EXTRA_PREFIXES")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        Ok(Self {
            database_url: require("DATABASE_URL")?,
            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string()),
            command_prefix: env::var("COMMAND_PREFIX").unwrap_or_else(|_| "rs.".to_string()),
            extra_prefixes,
        })
    }
}

fn require(key: &str) -> Result<String> {
    env::var(key).with_context(|| format!("missing required env var: {key}"))
}
