use poise::serenity_prelude::{self as serenity, Colour};
use serenity::builder::{
    CreateActionRow, CreateButton, CreateComponent, CreateContainer, CreateContainerComponent,
    CreateMessage, CreateSection, CreateSectionAccessory, CreateSectionComponent, CreateSeparator,
    CreateTextDisplay, CreateThumbnail, CreateUnfurledMediaItem,
};
use serenity::model::channel::MessageFlags;
use serenity::model::gateway::ActivityType;
use serenity::model::id::{GuildId, UserId};

use crate::{Context, Error};

const SPOTIFY_GREEN: u32 = 0x1D_B9_54;

/// Show what you or someone else is listening to on Spotify.
#[poise::command(prefix_command, slash_command, aliases("np"), guild_only)]
pub async fn nowplaying(
    ctx: Context<'_>,
    #[description = "User to check (defaults to you)"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap(); // guild_only guarantees this
    let target_id = user.as_ref().map(|u| u.id).unwrap_or(ctx.author().id);
    let sctx = ctx.serenity_context();

    let display = display_name(sctx, guild_id, target_id);
    let presence = sctx
        .cache
        .guild(guild_id)
        .and_then(|g| g.presences.get(&target_id).cloned());

    let Some(presence) = presence else {
        ctx.say(format!(
            "I don't have presence data for **{display}**. They may be offline, \
             invisible, or haven't been active since I started."
        ))
        .await?;
        return Ok(());
    };

    let spotify = presence
        .activities
        .iter()
        .find(|a| a.kind == ActivityType::Listening && a.name == "Spotify");

    let Some(spotify) = spotify else {
        let who = if target_id == ctx.author().id {
            "You're".to_string()
        } else {
            format!("**{display}** is")
        };
        ctx.say(format!(
            "{who} not listening to Spotify right now!\n\n\
             Make sure you have:\n\
             • **Display current activity** enabled for this server \
             (Settings → Activity Privacy)\n\
             • Spotify linked in **Settings → Connections** with \
             \"Display Spotify as your status\" on"
        ))
        .await?;
        return Ok(());
    };

    let track = spotify.details.as_deref().unwrap_or("Unknown track");
    let artist = spotify.state.as_deref().unwrap_or("Unknown artist");
    let album = spotify
        .assets
        .as_ref()
        .and_then(|a| a.large_text.as_deref())
        .unwrap_or("Unknown album");

    let album_art_url = spotify
        .assets
        .as_ref()
        .and_then(|a| a.large_image.as_deref())
        .and_then(|id| id.strip_prefix("spotify:"))
        .map(|id| format!("https://i.scdn.co/image/{id}"));

    let text = format!("-# {display} is listening to\n### 🎵 {track}\n**{artist}**\n-# {album}");

    let thumbnail_url = album_art_url
        .as_deref()
        .unwrap_or("https://i.scdn.co/image/ab67616d0000b273");

    let section = CreateSection::new(
        vec![CreateSectionComponent::TextDisplay(
            CreateTextDisplay::new(text),
        )],
        CreateSectionAccessory::Thumbnail(CreateThumbnail::new(
            CreateUnfurledMediaItem::new(thumbnail_url.to_string()),
        )),
    );

    let mut children: Vec<CreateContainerComponent<'_>> = vec![
        CreateContainerComponent::Section(section),
    ];

    // Add "Listen on Spotify" button if we have the track ID.
    if let Some(track_id) = &spotify.sync_id {
        let track_url = format!("https://open.spotify.com/track/{track_id}");
        children.push(CreateContainerComponent::Separator(
            CreateSeparator::new().divider(true),
        ));
        children.push(CreateContainerComponent::ActionRow(
            CreateActionRow::buttons(vec![
                CreateButton::new_link(track_url)
                    .label("Listen on Spotify")
                    .emoji("<:spotify:1492008393198538824>".parse::<serenity::all::ReactionType>().unwrap()),
            ]),
        ));
    }

    let container = CreateContainer::new(children)
        .accent_colour(Colour::new(SPOTIFY_GREEN));

    // Poise doesn't have a direct way to send Components v2 messages with flags,
    // so we use the serenity channel directly.
    let message = CreateMessage::new()
        .flags(MessageFlags::IS_COMPONENTS_V2)
        .components(vec![CreateComponent::Container(container)]);

    ctx.channel_id()
        .send_message(&ctx.http(), message)
        .await?;

    Ok(())
}

fn display_name(ctx: &serenity::prelude::Context, guild_id: GuildId, user_id: UserId) -> String {
    if let Some(guild) = ctx.cache.guild(guild_id) {
        if let Some(member) = guild.members.get(&user_id) {
            if let Some(nick) = &member.nick {
                return nick.to_string();
            }
            if let Some(global) = &member.user.global_name {
                return global.to_string();
            }
            return member.user.name.to_string();
        }
    }
    format!("<@{}>", user_id.get())
}
