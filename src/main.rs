mod commands;
mod config;
mod db;
mod forecast_render;
mod ping;
mod weather;

use std::sync::Arc;

use anyhow::{Context as _, Result};
use poise::serenity_prelude as serenity;
use tracing::{error, info};

use crate::config::Config;

/// Shared application state accessible via `ctx.data()`.
pub struct Data {
    pub db: sqlx::PgPool,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");

    init_tracing();

    let config = Config::from_env().context("loading configuration")?;
    info!(env = %config.environment, "starting broly");

    let pool = db::connect(&config.database_url).await?;
    info!("database pool ready");

    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_PRESENCES;

    // Build extra prefixes from config.
    let additional_prefixes: Vec<poise::Prefix> = config
        .extra_prefixes
        .iter()
        .map(|p| poise::Prefix::Literal(Box::leak(p.clone().into_boxed_str())))
        .collect();

    let options = poise::FrameworkOptions {
        commands: vec![
            commands::ping::ping(),
            commands::nowplaying::nowplaying(),
            commands::debug::debug(),
            commands::forecast::forecast(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some(config.command_prefix.clone().into()),
            additional_prefixes,
            ..Default::default()
        },
        on_error: |error| {
            Box::pin(async move {
                if let Err(e) = poise::builtins::on_error(error).await {
                    error!("error handling error: {e}");
                }
            })
        },
        ..Default::default()
    };

    let token =
        serenity::Token::from_env("DISCORD_TOKEN").context("DISCORD_TOKEN missing or invalid")?;

    let data = Arc::new(Data { db: pool });

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(Box::new(poise::Framework::new(options)))
        .data(data as _)
        .event_handler(Arc::new(ReadyHandler))
        .await
        .context("creating Discord client")?;

    info!("starting gateway connection");
    client.start().await.map_err(|e| {
        error!(?e, "client error");
        e
    })?;

    Ok(())
}

/// Registers slash commands on Ready. Runs alongside Poise's own event handling.
struct ReadyHandler;

#[serenity::async_trait]
impl serenity::EventHandler for ReadyHandler {
    async fn dispatch(&self, ctx: &serenity::prelude::Context, event: &serenity::FullEvent) {
        if let serenity::FullEvent::Ready { data_about_bot, .. } = event {
            info!(
                user = %data_about_bot.user.name,
                guilds = data_about_bot.guilds.len(),
                "gateway ready"
            );

            // Register slash commands — guild-specific if DEV_GUILD_ID is set, global otherwise.
            let commands = &ctx.data::<Data>(); // we don't actually need Data here
            let _ = commands; // suppress unused

            // Use poise's builtins to create the command builders from our framework.
            // Since we can't easily access the Framework from here, we register manually.
            if let Ok(guild_id_raw) = std::env::var("DEV_GUILD_ID") {
                if let Ok(id) = guild_id_raw.parse::<u64>() {
                    let guild = serenity::GuildId::new(id);
                    match guild
                        .set_commands(&ctx.http, &create_slash_commands())
                        .await
                    {
                        Ok(_) => info!(guild = id, "registered guild slash commands"),
                        Err(e) => error!(?e, "failed to register guild commands"),
                    }
                }
            } else {
                match serenity::Command::set_global_commands(&ctx.http, &create_slash_commands())
                    .await
                {
                    Ok(_) => info!("registered global slash commands"),
                    Err(e) => error!(?e, "failed to register global commands"),
                }
            }
        }
    }
}

/// Build slash command definitions manually (poise handles prefix commands,
/// but we register slash commands ourselves for instant availability on startup).
fn create_slash_commands() -> Vec<serenity::CreateCommand<'static>> {
    use serenity::builder::{CreateCommand, CreateCommandOption};
    use serenity::model::application::CommandOptionType;

    vec![
        CreateCommand::new("ping").description("Check bot latency: WebSocket, API round-trip, and database"),
        CreateCommand::new("nowplaying")
            .description("Show what you or someone else is listening to on Spotify")
            .add_option(
                CreateCommandOption::new(CommandOptionType::User, "user", "User to check")
                    .required(false),
            ),
        CreateCommand::new("forecast")
            .description("Show a 7-day weather forecast for a location")
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "place", "Place name")
                    .required(true),
            ),
    ]
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
