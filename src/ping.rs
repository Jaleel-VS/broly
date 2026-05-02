//! Shared latency-measurement logic used by both text-command `ping`
//! and slash-command `/ping`. Measures gateway WebSocket latency, API
//! round-trip latency, and database latency.

use std::sync::Arc;
use std::time::{Duration, Instant};

use serenity::client::Context;
use serenity::gateway::ShardManager;
use serenity::prelude::TypeMapKey;
use sqlx::PgPool;

/// TypeMapKey for stashing the shard manager in `client.data`.
/// Required so command handlers can look up the WebSocket latency
/// (which is tracked per-shard by the gateway runner).
pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

/// Read the current shard's WebSocket latency from the ShardManager.
pub async fn websocket_latency(ctx: &Context) -> Option<Duration> {
    let data = ctx.data.read().await;
    let shard_manager = data.get::<ShardManagerContainer>()?;
    let runners = shard_manager.runners.lock().await;
    runners.get(&ctx.shard_id).and_then(|r| r.latency)
}

/// Run `SELECT 1` against the pool and return how long it took.
/// Returns None if the query failed (so the caller can render "error"
/// instead of pretending everything is fine).
pub async fn db_latency(pool: &PgPool) -> Option<Duration> {
    let start = Instant::now();
    match sqlx::query("SELECT 1").fetch_one(pool).await {
        Ok(_) => Some(start.elapsed()),
        Err(_) => None,
    }
}

/// Format a Duration as milliseconds with two decimal places,
/// matching the Python bot's output style (e.g. `42.37ms`).
pub fn fmt_ms(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    format!("{ms:.2}ms")
}

pub fn fmt_ms_opt(d: Option<Duration>) -> String {
    match d {
        Some(d) => fmt_ms(d),
        None => "n/a".to_string(),
    }
}
