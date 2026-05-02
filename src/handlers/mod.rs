//! Event handling.
//!
//! Rust analog of discord.py "cogs". Each submodule owns one feature
//! and exposes a pure function that `Handler` dispatches to. The `Handler`
//! struct itself is thin: it holds shared state (db pool) and routes events.

mod general;
mod slash;

use serenity::async_trait;
use serenity::model::application::{Command, Interaction};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use sqlx::PgPool;
use tracing::{info, warn};

pub struct Handler {
    pub db: PgPool,
    pub prefix: String,
}

impl Handler {
    pub fn new(db: PgPool, prefix: String) -> Self {
        Self { db, prefix }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        general::on_message(&self.db, &self.prefix, &ctx, &msg).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(cmd) = interaction {
            slash::dispatch(&self.db, &ctx, &cmd).await;
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!(
            user = %ready.user.name,
            guilds = ready.guilds.len(),
            "gateway ready"
        );

        // Register slash commands.
        //
        // GUILD commands propagate instantly (good for local dev).
        // GLOBAL commands can take up to an hour (use in production).
        //
        // Strategy: if DEV_GUILD_ID is set, register to that guild only;
        // otherwise register globally.
        if let Ok(guild_id_raw) = std::env::var("DEV_GUILD_ID") {
            match guild_id_raw.parse::<u64>() {
                Ok(id) => {
                    let guild = GuildId::new(id);
                    if let Err(why) = guild.set_commands(&ctx.http, slash::register()).await {
                        warn!(?why, "failed to register guild commands");
                    } else {
                        info!(guild = id, "registered guild slash commands");
                    }
                }
                Err(_) => warn!("DEV_GUILD_ID is not a valid u64"),
            }
        } else {
            for cmd in slash::register() {
                if let Err(why) = Command::create_global_command(&ctx.http, cmd).await {
                    warn!(?why, "failed to register global command");
                }
            }
            info!("registered global slash commands");
        }
    }
}
