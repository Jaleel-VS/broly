//! Plain text commands over the gateway (the configured-prefix world).
//!
//! Slash commands are preferred for anything new. Text commands are handy
//! for owner-only debug tools and for muscle-memory invocations.

use std::time::Instant;

use serenity::all::Colour;
use serenity::builder::{CreateEmbed, CreateMessage, EditMessage};
use serenity::client::Context;
use serenity::model::channel::Message;
use sqlx::PgPool;
use tracing::warn;

use crate::ping::{self, fmt_ms, fmt_ms_opt};

pub async fn on_message(db: &PgPool, prefix: &str, ctx: &Context, msg: &Message) {
    let Some(rest) = msg.content.strip_prefix(prefix) else {
        return;
    };

    let (cmd, _args) = match rest.split_once(' ') {
        Some((c, a)) => (c, a),
        None => (rest, ""),
    };

    match cmd {
        "ping" => {
            if let Err(why) = ping_command(db, ctx, msg).await {
                warn!(?why, "ping command failed");
            }
        }
        _ => {
            // Unknown command - intentionally silent to avoid spam.
        }
    }
}

/// Full 3-latency ping:
///   WebSocket  - shard heartbeat RTT (from ShardManager)
///   API        - time to POST placeholder message and get 200 back
///   Database   - SELECT 1 round-trip
///
/// Posts a placeholder first, then edits the message in place with the
/// final embed.
async fn ping_command(db: &PgPool, ctx: &Context, msg: &Message) -> serenity::Result<()> {
    // 1. WebSocket latency (instant - just reads from the shard runner).
    let ws = ping::websocket_latency(ctx).await;

    // 2. API latency - send placeholder and measure the round-trip.
    let placeholder = CreateEmbed::new()
        .title("🏓 Pinging...")
        .colour(Colour::from_rgb(87, 242, 135));

    let start = Instant::now();
    let mut sent = msg
        .channel_id
        .send_message(&ctx.http, CreateMessage::new().embed(placeholder))
        .await?;
    let api = start.elapsed();

    // 3. Database latency - SELECT 1.
    let db_lat = ping::db_latency(db).await;

    // Edit the placeholder in place with the final three-field embed.
    let final_embed = CreateEmbed::new()
        .title("🏓 Pong!")
        .colour(Colour::BLURPLE)
        .field("WebSocket", format!("`{}`", fmt_ms_opt(ws)), true)
        .field("API", format!("`{}`", fmt_ms(api)), true)
        .field("Database", format!("`{}`", fmt_ms_opt(db_lat)), true);

    sent.edit(&ctx.http, EditMessage::new().embed(final_embed)).await?;
    Ok(())
}
