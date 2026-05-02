//! Slash-command definitions + dispatch.

use std::time::Instant;

use serenity::all::Colour;
use serenity::builder::{
    CreateCommand, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse,
};
use serenity::prelude::Context;
use serenity::model::application::CommandInteraction;
use sqlx::PgPool;
use tracing::warn;

use crate::ping::{self, fmt_ms, fmt_ms_opt};

pub fn register() -> Vec<CreateCommand<'static>> {
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
            Ok(())
        }
    };

    if let Err(why) = result {
        warn!(?why, command = %cmd.data.name, "slash handler error");
    }
}

async fn ping_command(
    db: &PgPool,
    ctx: &Context,
    cmd: &CommandInteraction,
) -> serenity::Result<()> {
    let ws = ping::websocket_latency(ctx);

    let defer = CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new());
    let start = Instant::now();
    cmd.create_response(&ctx.http, defer).await?;
    let api = start.elapsed();

    let db_lat = ping::db_latency(db).await;

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
