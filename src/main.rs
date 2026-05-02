mod config;
mod db;
mod handlers;
mod ping;

use std::sync::Arc;

use anyhow::{Context as _, Result};
use serenity::model::gateway::GatewayIntents;
use serenity::prelude::*;
use tokio::signal;
use tracing::{error, info};

use crate::config::Config;
use crate::handlers::Handler;

/// Shared application state accessible via `ctx.data::<AppState>()`.
pub struct AppState {
    pub db: sqlx::PgPool,
    pub prefix: String,
    pub extra_prefixes: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();

    // Must be called before any rustls usage (serenity gateway, sqlx, reqwest).
    // ring is already in our dep tree via sqlx's tls-rustls-ring-native-roots.
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");

    init_tracing();

    let config = Config::from_env().context("loading configuration")?;
    info!(env = %config.environment, "starting broly");

    let pool = db::connect(&config.database_url).await?;
    info!("database pool ready");

    let state = Arc::new(AppState {
        db: pool,
        prefix: config.command_prefix.clone(),
        extra_prefixes: config.extra_prefixes.clone(),
    });

    // Privileged intents: MESSAGE_CONTENT AND GUILD_PRESENCES must also
    // be enabled in the Discord developer portal for this bot.
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_PRESENCES;

    let token = Token::from_env("DISCORD_TOKEN").context("DISCORD_TOKEN missing or invalid")?;

    let mut client = Client::builder(token, intents)
        .data(state)
        .event_handler(Arc::new(Handler))
        .await
        .context("creating Discord client")?;

    let _shard_manager = client.shard_manager.runners.clone();

    tokio::spawn(async move {
        if let Err(why) = signal::ctrl_c().await {
            error!(?why, "failed to listen for ctrl_c");
        }
    });

    if let Err(why) = client.start().await {
        error!(?why, "client error");
        return Err(why.into());
    }

    info!("shutdown complete");
    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,broly=debug,serenity=warn"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true).compact())
        .init();
}
