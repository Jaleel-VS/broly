mod config;
mod db;
mod handlers;
mod ping;

use anyhow::{Context as _, Result};
use serenity::model::gateway::GatewayIntents;
use serenity::prelude::*;
use tokio::signal;
use tracing::{error, info};

use crate::config::Config;
use crate::handlers::Handler;
use crate::ping::ShardManagerContainer;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env if present (ignored in production when env is set directly)
    let _ = dotenvy::dotenv();

    init_tracing();

    let config = Config::from_env().context("loading configuration")?;
    info!(env = %config.environment, "starting broly");

    let pool = db::connect(&config.database_url).await?;
    // NOTE: auto-migrations are intentionally disabled because Broly reads
    // from a database that is also written to by another application.
    // Re-enable once Broly owns its own schema.
    info!("database pool ready");

    // Privileged intent: MESSAGE_CONTENT must also be enabled in the
    // Discord developer portal for this bot.
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS;

    let handler = Handler::new(pool.clone(), config.command_prefix.clone());

    let mut client = Client::builder(&config.discord_token, intents)
        .event_handler(handler)
        .await
        .context("creating Discord client")?;

    // Stash the shard manager in client.data so commands can read shard
    // latency (the canonical Serenity 0.12 pattern). Must happen BEFORE
    // client.start().
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    // Capture shard manager so we can shut it down gracefully.
    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        if let Err(why) = signal::ctrl_c().await {
            error!(?why, "failed to listen for ctrl_c");
            return;
        }
        info!("ctrl_c received, shutting down shards");
        shard_manager.shutdown_all().await;
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

    // RUST_LOG=broly=debug,serenity=info etc.
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,broly=debug,serenity=warn"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true).compact())
        .init();
}
