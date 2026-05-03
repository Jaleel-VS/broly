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

    // Text-only for now. Image rendering comes in Layer 2.
    let mut lines = vec![format!("**📍 {}**\n", loc.display_name())];
    for day in &days {
        let (desc, emoji) = crate::weather::weather_description(day.weather_code);
        lines.push(format!(
            "{} {} {:.0}°/{:.0}° — {}",
            &day.date[5..],
            emoji,
            day.temp_max,
            day.temp_min,
            desc
        ));
    }

    ctx.say(lines.join("\n")).await?;
    Ok(())
}
