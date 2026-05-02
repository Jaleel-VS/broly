//! Shared latency-measurement logic used by both text-command `ping`
//! and slash-command `/ping`. Measures gateway WebSocket latency, API
//! round-trip latency, and database latency.

use std::time::{Duration, Instant};

use serenity::prelude::Context;
use sqlx::PgPool;

/// Read the current shard's WebSocket latency from the runner info.
pub fn websocket_latency(ctx: &Context) -> Option<Duration> {
    ctx.runner_info.read().latency
}

/// Run `SELECT 1` against the pool and return how long it took.
pub async fn db_latency(pool: &PgPool) -> Option<Duration> {
    let start = Instant::now();
    match sqlx::query("SELECT 1").fetch_one(pool).await {
        Ok(_) => Some(start.elapsed()),
        Err(_) => None,
    }
}

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
