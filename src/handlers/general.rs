//! Plain text commands over the gateway (the configured-prefix world).
//!
//! Slash commands are preferred for anything new. Text commands are handy
//! for owner-only debug tools and for muscle-memory invocations.

use std::time::Instant;

use serenity::all::Colour;
use serenity::builder::{CreateEmbed, CreateMessage, EditMessage};
use serenity::prelude::Context;
use serenity::model::channel::Message;
use sqlx::PgPool;
use tracing::warn;

use crate::handlers::nowplaying;
use crate::ping::{self, fmt_ms, fmt_ms_opt};

pub async fn on_message(db: &PgPool, prefix: &str, ctx: &Context, msg: &Message) {
    let Some(rest) = msg.content.strip_prefix(prefix) else {
        return;
    };

    let (cmd, args) = match rest.split_once(' ') {
        Some((c, a)) => (c, a),
        None => (rest, ""),
    };

    match cmd {
        "ping" => {
            if let Err(why) = ping_command(db, ctx, msg).await {
                warn!(?why, "ping command failed");
            }
        }
        "nowplaying" | "np" => {
            if let Err(why) = nowplaying::run(ctx, msg, args).await {
                warn!(?why, "nowplaying command failed");
            }
        }
        "debug" => {
            if let Err(why) = debug_presence(ctx, msg).await {
                warn!(?why, "debug command failed");
            }
        }
        _ => {}
    }
}

async fn ping_command(db: &PgPool, ctx: &Context, msg: &Message) -> serenity::Result<()> {
    let ws = ping::websocket_latency(ctx);

    let placeholder = CreateEmbed::new()
        .title("🏓 Pinging...")
        .colour(Colour::from_rgb(87, 242, 135));

    let start = Instant::now();
    let mut sent = msg
        .channel_id
        .send_message(&ctx.http, CreateMessage::new().embed(placeholder))
        .await?;
    let api = start.elapsed();

    let db_lat = ping::db_latency(db).await;

    let final_embed = CreateEmbed::new()
        .title("🏓 Pong!")
        .colour(Colour::BLURPLE)
        .field("WebSocket", format!("`{}`", fmt_ms_opt(ws)), true)
        .field("API", format!("`{}`", fmt_ms(api)), true)
        .field("Database", format!("`{}`", fmt_ms_opt(db_lat)), true);

    sent.edit(&ctx.http, EditMessage::new().embed(final_embed))
        .await?;
    Ok(())
}

async fn debug_presence(ctx: &Context, msg: &Message) -> serenity::Result<()> {
    let Some(guild_id) = msg.guild_id else {
        msg.channel_id.say(&ctx.http, "DMs not supported.").await?;
        return Ok(());
    };

    let guild_in_cache = ctx.cache.guild(guild_id).is_some();
    let presence = ctx
        .cache
        .guild(guild_id)
        .and_then(|g| g.presences.get(&msg.author.id).cloned());

    let mut lines = vec![
        format!("**Guild in cache:** {guild_in_cache}"),
        format!("**Presence found:** {}", presence.is_some()),
    ];

    if let Some(p) = &presence {
        lines.push(format!("**Status:** {:?}", p.status));
        lines.push(format!("**Activities:** {}", p.activities.len()));
        for (i, a) in p.activities.iter().enumerate() {
            lines.push(format!(
                "  [{i}] kind={:?} name={} details={:?} state={:?}",
                a.kind, a.name, a.details, a.state
            ));
        }
    } else {
        lines.push("No presence data. Possible causes:".to_string());
        lines.push("• Bot joined this server after startup (restart to fix)".to_string());
        lines.push("• You're invisible".to_string());
        lines.push("• Presence Intent not enabled in dev portal".to_string());
        lines.push("• Your Discord activity privacy is off for this server".to_string());
    }

    msg.channel_id.say(&ctx.http, lines.join("\n")).await?;
    Ok(())
}
