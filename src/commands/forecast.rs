use poise::serenity_prelude as serenity;
use serenity::builder::{CreateAttachment, CreateMessage};

use crate::{Context, Error};

/// Show a 7-day weather forecast for a location.
#[poise::command(prefix_command, slash_command, aliases("weather", "wt"))]
pub async fn forecast(
    ctx: Context<'_>,
    #[description = "Place name"]
    #[rest]
    place: String,
) -> Result<(), Error> {
    let query = place.trim();
    if query.is_empty() {
        ctx.say("Usage: `rs.forecast <place>`").await?;
        return Ok(());
    }

    let http_client = reqwest::Client::new();

    let locations = crate::weather::geocode(&http_client, query).await?;

    if locations.is_empty() {
        ctx.say(format!("No results for **{query}**")).await?;
        return Ok(());
    }

    // TODO: disambiguation buttons when multiple matches.
    let loc = &locations[0];

    let days = crate::weather::forecast(&http_client, loc.latitude, loc.longitude).await?;

    // Render the forecast card image.
    let png_bytes = crate::forecast_render::render(&loc.display_name(), &days)?;

    let attachment = CreateAttachment::bytes(png_bytes, "forecast.png");
    let message = CreateMessage::new().add_file(attachment);

    ctx.channel_id()
        .send_message(&ctx.http(), message)
        .await?;

    Ok(())
}
