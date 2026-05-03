use std::time::Instant;

use poise::serenity_prelude::{self as serenity, Colour};
use serenity::builder::{CreateEmbed, CreateMessage, EditMessage};

use crate::ping::{self, fmt_ms, fmt_ms_opt};
use crate::{Context, Error};

/// Check bot latency: WebSocket, API round-trip, and database.
#[poise::command(prefix_command, slash_command, aliases("p"))]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let ws = ping::websocket_latency(ctx.serenity_context());
    let db = &ctx.data().db;

    match ctx {
        poise::Context::Prefix(prefix_ctx) => {
            // Text command: send placeholder, measure API, edit in place.
            let placeholder = CreateEmbed::new()
                .title("🏓 Pinging...")
                .colour(Colour::from_rgb(87, 242, 135));

            let start = Instant::now();
            let mut sent = prefix_ctx
                .msg
                .channel_id
                .send_message(&ctx.http(), CreateMessage::new().embed(placeholder))
                .await?;
            let api = start.elapsed();

            let db_lat = ping::db_latency(db).await;

            let embed = pong_embed(ws, api, db_lat);
            sent.edit(&ctx.http(), EditMessage::new().embed(embed)).await?;
        }
        poise::Context::Application(_) => {
            // Slash command: defer, measure, edit response.
            ctx.defer().await?;
            let api = Instant::now(); // approximate — defer is the API call
            let db_lat = ping::db_latency(db).await;
            let elapsed = api.elapsed();

            let embed = pong_embed(ws, elapsed, db_lat);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
    }

    Ok(())
}

fn pong_embed(
    ws: Option<std::time::Duration>,
    api: std::time::Duration,
    db: Option<std::time::Duration>,
) -> CreateEmbed<'static> {
    CreateEmbed::new()
        .title("🏓 Pong!")
        .colour(Colour::BLURPLE)
        .field("WebSocket", format!("`{}`", fmt_ms_opt(ws)), true)
        .field("API", format!("`{}`", fmt_ms(api)), true)
        .field("Database", format!("`{}`", fmt_ms_opt(db)), true)
}
