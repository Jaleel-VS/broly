# Broly

A Discord bot in Rust. Built on [Serenity](https://github.com/serenity-rs/serenity) + [sqlx](https://github.com/launchbadge/sqlx) + [tokio](https://tokio.rs).

Codename. Named after the Legendary Super Saiyan. Actual bot display name is configured separately in the Discord developer portal.

## Layout

```
src/
  main.rs            entrypoint: tracing, config, db, shard startup, graceful shutdown
  config.rs          env-var loader
  db.rs              Postgres pool + migration runner (disabled by default)
  ping.rs            shared 3-latency measurement (WS / API / DB)
  handlers/
    mod.rs           EventHandler impl (router + slash registration)
    general.rs       text commands (rs.ping, …)
    slash.rs         slash commands (/ping, …)
migrations/          sqlx migrations (folder exists, currently empty)
Dockerfile           multi-stage release build
.env.example         documented env vars
```

## Quick start

```bash
cp .env.example .env
# edit .env — set DISCORD_TOKEN and DATABASE_URL
# optionally set DEV_GUILD_ID for instant slash-command propagation

cargo run
```

The first run takes ~30-60s (compiles serenity + rustls). After that, `cargo run` is a few seconds.

### Slash commands in dev

Global slash commands take up to an hour to propagate. For fast iteration:

```bash
export DEV_GUILD_ID=123456789012345678   # your test server
cargo run
```

The bot registers commands to that guild only, so they appear instantly.

### Discord dev portal — required toggles

For text commands (`rs.ping`, `rs.whatever`) to fire, you must enable the **Message Content Intent** under **Bot → Privileged Gateway Intents** in the Discord dev portal. Slash commands work without it.

## Adding a command

**Text command:**

1. Add a match arm in `src/handlers/general.rs`.

**Slash command:**

1. Push a `CreateCommand` into `register()` in `src/handlers/slash.rs`.
2. Add a match arm in `dispatch()`.
3. Implement the handler function.

## Database

Broly opens a Postgres connection pool at startup. Auto-migrations are **disabled** by default because Broly is designed to read from a shared database. To enable:

- Put migrations in `./migrations/` (use `sqlx migrate add -r <name>`)
- Uncomment the `db::run_migrations(&pool).await?` call in `src/main.rs`

Compile-time query checking needs either a live `DATABASE_URL` at build time or offline mode:

```bash
cargo install sqlx-cli --no-default-features --features postgres
cargo sqlx prepare    # generates .sqlx/ with query metadata
# commit .sqlx/
# builds with SQLX_OFFLINE=true work without a DB
```

## Docker

```bash
docker build -t broly .
docker run --rm -it \
  -e DISCORD_TOKEN=... \
  -e DATABASE_URL=... \
  broly
```

## Logs

Structured via `tracing`. Control verbosity with `RUST_LOG`:

```bash
RUST_LOG=info,broly=debug,serenity=warn cargo run
```

## License

MIT. See [LICENSE](./LICENSE).
