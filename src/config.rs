use anyhow::{Context, Result};
use std::env;

/// Application configuration loaded from environment variables.
///
/// Required vars:
///   DISCORD_TOKEN     - Bot token from the Discord developer portal
///   DATABASE_URL      - Postgres connection string
///
/// Optional vars:
///   ENVIRONMENT       - "local" | "staging" | "prod" (default: "local")
///   COMMAND_PREFIX    - Text-command prefix (default: "rs.")
#[derive(Clone, Debug)]
pub struct Config {
    pub discord_token: String,
    pub database_url: String,
    pub environment: String,
    pub command_prefix: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            discord_token: require("DISCORD_TOKEN")?,
            database_url: require("DATABASE_URL")?,
            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string()),
            command_prefix: env::var("COMMAND_PREFIX").unwrap_or_else(|_| "rs.".to_string()),
        })
    }
}

fn require(key: &str) -> Result<String> {
    env::var(key).with_context(|| format!("missing required env var: {key}"))
}
