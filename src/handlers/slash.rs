//! Slash-command definitions + dispatch.
//!
//! Add new commands in three steps:
//!   1. Push a CreateCommand in `register()`
//!   2. Add a match arm in `dispatch()` pointing at your handler function
//!   3. Implement the handler function

use std::time::Instant;

use serenity::all::Colour;
use serenity::builder::{
    CreateCommand, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse,
};
use serenity::client::Context;
use serenity::model::application::CommandInteraction;
use sqlx::PgPool;
use tracing::warn;

use crate::ping::{self, fmt_ms, fmt_ms_opt};

pub fn register() -> Vec<CreateCommand> {
    vec![
        CreateCommand::new("ping")
            .description("Check bot latency: WebSocket, API round-trip, and database"),
    ]
}

pub async fn dispatch(db: &PgPool, ctx: &Context, cmd: &CommandInteraction) {
    let result = match cmd.data.name.as_str() {
        "ping" => ping_command(db, ctx, cmd).await,
        other => {
            warn!(command = other, "unknown slash command");
            respond(ctx, cmd, "Unknown command", false).await
        }
    };

    if let Err(why) = result {
        warn!(?why, command = %cmd.data.name, "slash handler error");
    }
}

/// 3-latency ping for slash commands.
///
/// Interaction responses are different from regular messages: you can't
/// just `send` then `edit` a normal message. Instead:
///   1. `defer` - tells Discord "I'm thinking, give me up to 15 min"
///   2. Measure API round-trip on the defer call itself
///   3. Do the rest of the work (DB query, etc.)
///   4. `edit_response` - replaces the deferred spinner with the final embed
async fn ping_command(
    db: &PgPool,
    ctx: &Context,
    cmd: &CommandInteraction,
) -> serenity::Result<()> {
    // 1. WebSocket latency (instant).
    let ws = ping::websocket_latency(ctx).await;

    // 2. API latency: measure how long the defer HTTP call takes.
    let defer = CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new());
    let start = Instant::now();
    cmd.create_response(&ctx.http, defer).await?;
    let api = start.elapsed();

    // 3. Database latency.
    let db_lat = ping::db_latency(db).await;

    // Final embed replaces the deferred state.
    let embed = CreateEmbed::new()
        .title("🏓 Pong!")
        .colour(Colour::BLURPLE)
        .field("WebSocket", format!("`{}`", fmt_ms_opt(ws)), true)
        .field("API", format!("`{}`", fmt_ms(api)), true)
        .field("Database", format!("`{}`", fmt_ms_opt(db_lat)), true);

    cmd.edit_response(&ctx.http, EditInteractionResponse::new().embed(embed))
        .await?;
    Ok(())
}

// --- response helper for simple one-shot replies ------------------------
// Kept for future commands. /ping now uses defer+edit so it doesn't use this.
#[allow(dead_code)]
async fn respond(
    ctx: &Context,
    cmd: &CommandInteraction,
    content: &str,
    ephemeral: bool,
) -> serenity::Result<()> {
    let data = CreateInteractionResponseMessage::new()
        .content(content)
        .ephemeral(ephemeral);
    let builder = CreateInteractionResponse::Message(data);
    cmd.create_response(&ctx.http, builder).await
}
