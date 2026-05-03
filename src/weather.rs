//! Open-Meteo API client for geocoding and 7-day forecast.
//!
//! No API key required. Free, no rate limits.
//! Geocoding: https://open-meteo.com/en/docs/geocoding-api
//! Forecast:  https://open-meteo.com/en/docs

use serde::Deserialize;

const GEOCODE_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";
const FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";

// ── Geocoding ──

#[derive(Debug, Deserialize)]
pub struct GeoResult {
    pub results: Option<Vec<GeoLocation>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeoLocation {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country: Option<String>,
    pub admin1: Option<String>,
    pub timezone: Option<String>,
}

impl GeoLocation {
    /// "Cape Town, Western Cape, South Africa"
    pub fn display_name(&self) -> String {
        let mut parts = vec![self.name.clone()];
        if let Some(admin) = &self.admin1 {
            parts.push(admin.clone());
        }
        if let Some(country) = &self.country {
            parts.push(country.clone());
        }
        parts.join(", ")
    }
}

pub async fn geocode(client: &reqwest::Client, query: &str) -> anyhow::Result<Vec<GeoLocation>> {
    let resp: GeoResult = client
        .get(GEOCODE_URL)
        .query(&[("name", query), ("count", "5"), ("language", "en")])
        .send()
        .await?
        .json()
        .await?;

    Ok(resp.results.unwrap_or_default())
}

// ── Forecast ──

#[derive(Debug, Deserialize)]
pub struct ForecastResponse {
    pub daily: DailyForecast,
    pub timezone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DailyForecast {
    pub time: Vec<String>,
    pub temperature_2m_max: Vec<f64>,
    pub temperature_2m_min: Vec<f64>,
    pub weather_code: Vec<u8>,
}

/// One day's forecast, unpacked for easy rendering.
#[derive(Debug, Clone)]
pub struct DayForecast {
    pub date: String,       // "2026-05-03"
    pub temp_max: f64,
    pub temp_min: f64,
    pub weather_code: u8,   // WMO code
}

pub async fn forecast(
    client: &reqwest::Client,
    lat: f64,
    lon: f64,
) -> anyhow::Result<Vec<DayForecast>> {
    let resp: ForecastResponse = client
        .get(FORECAST_URL)
        .query(&[
            ("latitude", &lat.to_string()),
            ("longitude", &lon.to_string()),
            ("daily", &"temperature_2m_max,temperature_2m_min,weather_code".to_string()),
            ("timezone", &"auto".to_string()),
            ("forecast_days", &"7".to_string()),
        ])
        .send()
        .await?
        .json()
        .await?;

    let days = resp
        .daily
        .time
        .iter()
        .enumerate()
        .map(|(i, date)| DayForecast {
            date: date.clone(),
            temp_max: resp.daily.temperature_2m_max[i],
            temp_min: resp.daily.temperature_2m_min[i],
            weather_code: resp.daily.weather_code[i],
        })
        .collect();

    Ok(days)
}

/// Map WMO weather code to a short description + emoji.
pub fn weather_description(code: u8) -> (&'static str, &'static str) {
    match code {
        0 => ("Clear sky", "☀️"),
        1 => ("Mainly clear", "🌤"),
        2 => ("Partly cloudy", "⛅"),
        3 => ("Overcast", "☁️"),
        45 | 48 => ("Fog", "🌫"),
        51 | 53 | 55 => ("Drizzle", "🌦"),
        56 | 57 => ("Freezing drizzle", "🌧"),
        61 | 63 | 65 => ("Rain", "🌧"),
        66 | 67 => ("Freezing rain", "🌧"),
        71 | 73 | 75 => ("Snow", "❄️"),
        77 => ("Snow grains", "❄️"),
        80 | 81 | 82 => ("Rain showers", "🌦"),
        85 | 86 => ("Snow showers", "🌨"),
        95 => ("Thunderstorm", "⛈"),
        96 | 99 => ("Thunderstorm + hail", "⛈"),
        _ => ("Unknown", "❓"),
    }
}
