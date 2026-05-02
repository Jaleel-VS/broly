//! Event handling — routes gateway events to feature modules.

mod general;
mod nowplaying;
mod slash;

use serenity::async_trait;
use serenity::model::application::{Command, Interaction};
use serenity::model::event::FullEvent;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use tracing::{info, warn};

use crate::AppState;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        match event {
            FullEvent::Message { new_message, .. } => {
                if new_message.author.bot() {
                    return;
                }
                let state = ctx.data::<AppState>();
                // Check primary prefix, then any extra prefixes.
                let all_prefixes = std::iter::once(state.prefix.as_str())
                    .chain(state.extra_prefixes.iter().map(String::as_str));
                for prefix in all_prefixes {
                    if new_message.content.starts_with(prefix) {
                        general::on_message(&state.db, prefix, ctx, new_message).await;
                        return;
                    }
                }
            }
            FullEvent::InteractionCreate { interaction, .. } => {
                if let Interaction::Command(cmd) = interaction {
                    let state = ctx.data::<AppState>();
                    slash::dispatch(&state.db, ctx, cmd).await;
                }
            }
            FullEvent::Ready { data_about_bot, .. } => {
                info!(
                    user = %data_about_bot.user.name,
                    guilds = data_about_bot.guilds.len(),
                    "gateway ready"
                );

                if let Ok(guild_id_raw) = std::env::var("DEV_GUILD_ID") {
                    match guild_id_raw.parse::<u64>() {
                        Ok(id) => {
                            let guild = GuildId::new(id);
                            if let Err(why) =
                                guild.set_commands(&ctx.http, &slash::register()).await
                            {
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
            _ => {}
        }
    }
}
