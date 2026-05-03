use crate::{Context, Error};

/// Dump presence data for debugging activity detection.
#[poise::command(prefix_command, guild_only, owners_only)]
pub async fn debug(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let sctx = ctx.serenity_context();

    let guild_in_cache = sctx.cache.guild(guild_id).is_some();
    let presence = sctx
        .cache
        .guild(guild_id)
        .and_then(|g| g.presences.get(&ctx.author().id).cloned());

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

    ctx.say(lines.join("\n")).await?;
    Ok(())
}
