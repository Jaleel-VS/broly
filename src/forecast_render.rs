//! Renders a 7-day forecast as a dark-themed horizontal PNG image.
//!
//! Uses resvg to render an SVG template to PNG. Weather icons are simple
//! SVG shapes (circle for sun, paths for clouds/rain) rather than emoji,
//! because emoji rendering in SVG is font-dependent and unreliable.

use crate::weather::DayForecast;

const CARD_W: u32 = 840;
const CARD_H: u32 = 200;
const COL_W: f32 = 120.0;
const BG_COLOR: &str = "#1e1e2e"; // dark background
const TEXT_COLOR: &str = "#cdd6f4"; // light text
const SUBTEXT_COLOR: &str = "#6c7086"; // muted text
const ACCENT_COLOR: &str = "#89b4fa"; // blue accent for location
const HIGH_COLOR: &str = "#f9e2af"; // warm yellow for high temp
const LOW_COLOR: &str = "#89b4fa"; // cool blue for low temp

/// Render a 7-day forecast card to PNG bytes.
pub fn render(location: &str, days: &[DayForecast]) -> anyhow::Result<Vec<u8>> {
    let svg = build_svg(location, days);
    svg_to_png(&svg)
}

fn build_svg(location: &str, days: &[DayForecast]) -> String {
    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{CARD_W}" height="{CARD_H}" viewBox="0 0 {CARD_W} {CARD_H}">
  <rect width="{CARD_W}" height="{CARD_H}" rx="16" fill="{BG_COLOR}"/>
  <text x="20" y="30" font-family="sans-serif" font-size="14" fill="{ACCENT_COLOR}" font-weight="bold">📍 {location}</text>
  <text x="{}" y="30" font-family="sans-serif" font-size="11" fill="{SUBTEXT_COLOR}" text-anchor="end">7-Day Forecast</text>
  <line x1="0" y1="42" x2="{CARD_W}" y2="42" stroke="#313244" stroke-width="1"/>
"##,
        CARD_W - 20
    );

    for (i, day) in days.iter().enumerate() {
        let x = i as f32 * COL_W;
        let cx = x + COL_W / 2.0;

        // Parse date for day name
        let day_label = day_of_week(&day.date);
        let date_short = &day.date[5..]; // "05-03"

        // Weather icon (simple SVG shapes)
        let icon = weather_icon(day.weather_code, cx, 100.0);

        // Separator between columns
        if i > 0 {
            svg.push_str(&format!(
                r##"  <line x1="{x}" y1="48" x2="{x}" y2="{}" stroke="#313244" stroke-width="1"/>"##,
                CARD_H - 10
            ));
            svg.push('\n');
        }

        svg.push_str(&format!(
            r##"  <text x="{cx}" y="65" font-family="sans-serif" font-size="13" fill="{TEXT_COLOR}" text-anchor="middle" font-weight="bold">{day_label}</text>
  <text x="{cx}" y="80" font-family="sans-serif" font-size="10" fill="{SUBTEXT_COLOR}" text-anchor="middle">{date_short}</text>
{icon}
  <text x="{cx}" y="150" font-family="sans-serif" font-size="18" fill="{HIGH_COLOR}" text-anchor="middle" font-weight="bold">{:.0}°</text>
  <text x="{cx}" y="170" font-family="sans-serif" font-size="13" fill="{LOW_COLOR}" text-anchor="middle">{:.0}°</text>
"##,
            day.temp_max, day.temp_min
        ));
    }

    svg.push_str("</svg>");
    svg
}

/// Simple SVG weather icons based on WMO code.
fn weather_icon(code: u8, cx: f32, cy: f32) -> String {
    match code {
        // Clear
        0 => sun(cx, cy),
        // Mainly clear / partly cloudy
        1 | 2 => sun_cloud(cx, cy),
        // Overcast
        3 => cloud(cx, cy),
        // Fog
        45 | 48 => fog(cx, cy),
        // Drizzle / rain
        51..=67 | 80..=82 => rain(cx, cy),
        // Snow
        71..=77 | 85 | 86 => snow(cx, cy),
        // Thunderstorm
        95..=99 => thunder(cx, cy),
        _ => cloud(cx, cy),
    }
}

fn sun(cx: f32, cy: f32) -> String {
    format!(r##"  <circle cx="{cx}" cy="{cy}" r="12" fill="#f9e2af" opacity="0.9"/>"##)
}

fn cloud(cx: f32, cy: f32) -> String {
    format!(
        r##"  <ellipse cx="{cx}" cy="{cy}" rx="16" ry="10" fill="#9399b2" opacity="0.8"/>"##
    )
}

fn sun_cloud(cx: f32, cy: f32) -> String {
    format!(
        r##"  <circle cx="{}" cy="{}" r="10" fill="#f9e2af" opacity="0.8"/>
  <ellipse cx="{}" cy="{}" rx="14" ry="9" fill="#9399b2" opacity="0.8"/>"##,
        cx - 6.0,
        cy - 4.0,
        cx + 4.0,
        cy + 2.0
    )
}

fn rain(cx: f32, cy: f32) -> String {
    format!(
        r##"  <ellipse cx="{cx}" cy="{}" rx="14" ry="9" fill="#9399b2" opacity="0.8"/>
  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#89b4fa" stroke-width="2" stroke-linecap="round"/>
  <line x1="{cx}" y1="{}" x2="{cx}" y2="{}" stroke="#89b4fa" stroke-width="2" stroke-linecap="round"/>
  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#89b4fa" stroke-width="2" stroke-linecap="round"/>"##,
        cy - 3.0,
        cx - 6.0, cy + 6.0, cx - 8.0, cy + 14.0,
        cy + 6.0, cy + 14.0,
        cx + 6.0, cy + 6.0, cx + 4.0, cy + 14.0
    )
}

fn snow(cx: f32, cy: f32) -> String {
    format!(
        r##"  <ellipse cx="{cx}" cy="{}" rx="14" ry="9" fill="#9399b2" opacity="0.8"/>
  <circle cx="{}" cy="{}" r="2" fill="#cdd6f4"/>
  <circle cx="{cx}" cy="{}" r="2" fill="#cdd6f4"/>
  <circle cx="{}" cy="{}" r="2" fill="#cdd6f4"/>"##,
        cy - 3.0,
        cx - 6.0, cy + 10.0,
        cy + 14.0,
        cx + 6.0, cy + 10.0
    )
}

fn fog(cx: f32, cy: f32) -> String {
    format!(
        r##"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#9399b2" stroke-width="2" stroke-linecap="round"/>
  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#9399b2" stroke-width="2" stroke-linecap="round" opacity="0.6"/>
  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#9399b2" stroke-width="2" stroke-linecap="round" opacity="0.4"/>"##,
        cx - 14.0, cy - 4.0, cx + 14.0, cy - 4.0,
        cx - 12.0, cy + 2.0, cx + 12.0, cy + 2.0,
        cx - 14.0, cy + 8.0, cx + 14.0, cy + 8.0
    )
}

fn thunder(cx: f32, cy: f32) -> String {
    format!(
        r##"  <ellipse cx="{cx}" cy="{}" rx="14" ry="9" fill="#585b70" opacity="0.9"/>
  <polygon points="{},{} {},{} {},{} {},{} {},{} {},{}" fill="#f9e2af"/>"##,
        cy - 3.0,
        cx, cy + 4.0,
        cx - 4.0, cy + 10.0,
        cx + 2.0, cy + 10.0,
        cx - 2.0, cy + 18.0,
        cx + 4.0, cy + 8.0,
        cx - 1.0, cy + 8.0
    )
}

fn day_of_week(date_str: &str) -> &'static str {
    // Parse "2026-05-03" and get day of week
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return "???";
    }
    let y: i32 = parts[0].parse().unwrap_or(2026);
    let m: u32 = parts[1].parse().unwrap_or(1);
    let d: u32 = parts[2].parse().unwrap_or(1);

    // Zeller's formula (Gregorian)
    let (m, y) = if m <= 2 { (m + 12, y - 1) } else { (m, y) };
    let q = d as i32;
    let k = y % 100;
    let j = y / 100;
    let h = (q + (13 * (m as i32 + 1)) / 5 + k + k / 4 + j / 4 - 2 * j) % 7;
    let h = ((h + 7) % 7) as usize;

    ["SAT", "SUN", "MON", "TUE", "WED", "THU", "FRI"][h]
}

fn svg_to_png(svg_str: &str) -> anyhow::Result<Vec<u8>> {
    let opts = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg_str, &opts)?;

    let size = tree.size();
    let w = size.width() as u32;
    let h = size.height() as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h)
        .ok_or_else(|| anyhow::anyhow!("failed to create pixmap"))?;

    resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());

    Ok(pixmap.encode_png()?)
}
