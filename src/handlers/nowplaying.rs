//! `rs.np` / `rs.nowplaying` — Spotify now-playing card using Components v2.
//!
//! Mirrors the Python bot's spotify_cog layout:
//!   Container (accent colour) → Section (text + album art thumbnail)
//!   → Separator → ActionRow (Listen on Spotify link button)

use serenity::all::Colour;
use serenity::builder::{
    CreateActionRow, CreateButton, CreateComponent, CreateContainer, CreateContainerComponent,
    CreateMessage, CreateSection, CreateSectionAccessory, CreateSectionComponent, CreateSeparator,
    CreateTextDisplay, CreateThumbnail, CreateUnfurledMediaItem,
};
use serenity::model::channel::{Message, MessageFlags};
use serenity::model::gateway::{ActivityType, Presence};
use serenity::model::id::{GuildId, UserId};
use serenity::prelude::Context;

const SPOTIFY_GREEN: u32 = 0x1D_B9_54;

pub async fn run(ctx: &Context, msg: &Message, args: &str) -> serenity::Result<()> {
    let Some(guild_id) = msg.guild_id else {
        msg.channel_id
            .say(&ctx.http, "`rs.nowplaying` only works in servers, not DMs.")
            .await?;
        return Ok(());
    };

    let target = resolve_target(msg, args);
    let display = display_name(ctx, guild_id, target);
    let presence = fetch_presence(ctx, guild_id, target);

    let Some(presence) = presence else {
        msg.channel_id
            .say(
                &ctx.http,
                format!(
                    "I don't have presence data for **{display}**. They may be offline, \
                     invisible, or haven't been active since I started."
                ),
            )
            .await?;
        return Ok(());
    };

    let spotify = presence
        .activities
        .iter()
        .find(|a| a.kind == ActivityType::Listening && a.name == "Spotify");

    let Some(spotify) = spotify else {
        let who = if target == msg.author.id {
            "You're".to_string()
        } else {
            format!("**{display}** is")
        };
        msg.channel_id
            .say(
                &ctx.http,
                format!(
                    "{who} not listening to Spotify right now!\n\n\
                     Make sure you have:\n\
                     • **Display current activity** enabled for this server \
                     (Settings → Activity Privacy)\n\
                     • Spotify linked in **Settings → Connections** with \
                     \"Display Spotify as your status\" on"
                ),
            )
            .await?;
        return Ok(());
    };

    // Extract Spotify metadata.
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

    // Build the text block matching the Python layout:
    //   -# {display} is listening to
    //   ### 🎵 {title}
    //   **{artist}**
    //   -# {album}
    let text = format!(
        "-# {display} is listening to\n### 🎵 {track}\n**{artist}**\n-# {album}"
    );

    // Build the section: text on the left, album art thumbnail on the right.
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

    // Build container children.
    let mut children: Vec<CreateContainerComponent<'_>> = vec![
        CreateContainerComponent::Section(section),
    ];

    // Add "Listen on Spotify" link button if we have the track ID.
    if let Some(track_id) = &spotify.sync_id {
        let track_url = format!("https://open.spotify.com/track/{track_id}");
        children.push(CreateContainerComponent::Separator(
            CreateSeparator::new().divider(true),
        ));
        children.push(CreateContainerComponent::ActionRow(
            CreateActionRow::buttons(vec![
                CreateButton::new_link(track_url).label("Listen on Spotify").emoji("<:spotify:1492008393198538824>".parse::<serenity::all::ReactionType>().unwrap()),
            ]),
        ));
    }

    // Container with Spotify green accent.
    let container = CreateContainer::new(children)
        .accent_colour(Colour::new(SPOTIFY_GREEN));

    let message = CreateMessage::new()
        .flags(MessageFlags::IS_COMPONENTS_V2)
        .components(vec![CreateComponent::Container(container)]);

    msg.channel_id.send_message(&ctx.http, message).await?;
    Ok(())
}

fn resolve_target(msg: &Message, args: &str) -> UserId {
    if let Some(user) = msg.mentions.first() {
        return user.id;
    }
    if let Some(first_arg) = args.split_whitespace().next() {
        if let Ok(id) = first_arg.parse::<u64>() {
            return UserId::new(id);
        }
    }
    msg.author.id
}

fn fetch_presence(ctx: &Context, guild_id: GuildId, user: UserId) -> Option<Presence> {
    let guild = ctx.cache.guild(guild_id)?;
    guild.presences.get(&user).cloned()
}

fn display_name(ctx: &Context, guild_id: GuildId, user_id: UserId) -> String {
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
