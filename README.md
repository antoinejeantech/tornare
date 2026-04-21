# Tornare

A full-stack platform for organizing competitive Overwatch events — pick-up games and tournaments — with a Discord bot that posts live announcements to your servers.

**Stack:** Rust (Axum) · PostgreSQL · Vue 3 · Discord Interactions API · Docker

---

## ✨ Features

### 🎮 Event management
Create **PUG** (pick-up game) or **TOURNEY** events. Add players with roles and preferred positions, build teams, run matches, track winners, and generate tournament brackets. An event has a full lifecycle: `DRAFT → ACTIVE → ENDED`.

### 🧠 Smart roster & team tools
- ⚖️ **Auto-balance**: distributes players across teams by role (Tank / DPS / Support) respecting Overwatch slot quotas.
- 🏆 **Tournament brackets**: one-click round generation from registered teams via `tourney/generate` — the bracket is re-computable and clearable at any time.
- 🔗 **Public signup links**: shareable tokenized URLs let outsiders request to join — rotating the token instantly invalidates any previously shared link.
- 📋 **Role-aware signups**: players declare their role and rank preferences when requesting to join; the event owner accepts or declines from the dashboard, and those preferences feed directly into auto-balance.

### 🤖 Discord bot
The bot runs as a separate process, polling the database on a configurable interval and posting rich embeds to announcement channels when new events go live.

Guilds are self-served through slash commands:

| Command | What it does |
|---|---|
| `/setup [channel]` | Connect a Discord server — requires **Administrator** permission and a linked Tornare account |
| `/unsetup` | Disconnect the server |
| `/help` | Show available commands |

The backend receives slash-command interactions via an **ed25519-signed webhook** (`POST /api/discord/interactions`). Signatures are verified on every request; an unconfigured public key fails closed (401).

🔒 **Guild ownership protection**: a guild registered by one user cannot be claimed by another via `/setup` as long as it remains active. Removing a guild is a soft-delete — the announcement history (`discord_guild_posts`) is preserved, and the guild ID can be re-claimed after removal.

### 🔐 Auth
JWT-based session with refresh tokens. Supports email/password and OAuth via **Battle.net** and **Discord**. Lockout prevention: disconnecting an OAuth provider is blocked if it's the user's only login method.

---

## 🚀 Quick start

```bash
# 1. Copy env files
make bootstrap

# 2. Start everything (postgres + backend + frontend + bot)
make up
```

| Service | URL |
|---|---|
| Frontend | http://localhost:5173 |
| Backend API | http://localhost:8000 |
| Health check | http://localhost:8000/health |

---

## ⚙️ Environment variables

### `backend/.env`

| Variable | Description |
|---|---|
| `DATABASE_URL` | Postgres connection string |
| `JWT_SECRET` | Signing secret for auth tokens |
| `CORS_ALLOWED_ORIGINS` | Comma-separated allowed origins (e.g. `https://app.example.com,http://localhost:5173`) |
| `APP_ENV` | Runtime mode (`development` by default, `production`/`prod` enables stricter checks) |
| `DISCORD_BOT_PUBLIC_KEY` | Ed25519 public key from the Discord developer portal — **required** for slash commands |
| `DISCORD_BOT_TOKEN` | Bot token — used to verify channel permissions at `/setup` time |
| `DISCORD_CLIENT_ID` | Discord OAuth app client ID |
| `DISCORD_CLIENT_SECRET` | Discord OAuth app client secret |
| `DISCORD_REDIRECT_URI` | Discord OAuth redirect URI |
| `BATTLENET_CLIENT_ID` | Battle.net OAuth app client ID |
| `BATTLENET_CLIENT_SECRET` | Battle.net OAuth app client secret |
| `BATTLENET_REDIRECT_URI` | Battle.net OAuth redirect URI |
| `FRONTEND_URL` | Frontend base URL used in email links |
| `EMAIL_DRIVER` | Email backend: `smtp` (default) or `resend` |
| `FROM_EMAIL` | Sender address for verification/reset emails |
| `RESEND_API_KEY` | Required when `EMAIL_DRIVER=resend` |
| `SMTP_HOST` | Required when `EMAIL_DRIVER=smtp` |
| `SMTP_PORT` | SMTP port (default: `1025`) |
| `SMTP_TLS_MODE` | SMTP TLS mode: `none`, `starttls`, or `implicit` |
| `SMTP_USERNAME` | Optional SMTP username (authenticated relays like Gmail) |
| `SMTP_PASSWORD` | Optional SMTP password/app password |

### Mailer configuration

Tornare supports 2 mailer backends:

1. `smtp` (default): direct SMTP transport. Best for local dev with Mailpit.
2. `resend`: Resend REST API. Typical production setup.

SMTP TLS modes:

1. `none`: no TLS upgrade (ideal for local Mailpit).
2. `starttls`: STARTTLS upgrade (recommended for Gmail on port 587).
3. `implicit`: TLS from connection start (common on port 465).

#### Dev with Mailpit (Docker compose)

Use these values in `backend/.env` when backend runs in Docker compose (`make up` / `make dev`):

  EMAIL_DRIVER=smtp
  FROM_EMAIL=noreply@tornare.gg
  SMTP_HOST=mailpit
  SMTP_PORT=1025
  SMTP_TLS_MODE=none

Mailpit endpoints:

1. SMTP: `mailpit:1025` from containers
2. Web UI from host: http://localhost:8025

#### Dev with Mailpit (backend running on host)

If you run backend directly on macOS/Linux host (not inside compose), use:

  EMAIL_DRIVER=smtp
  FROM_EMAIL=noreply@tornare.gg
  SMTP_HOST=localhost
  SMTP_PORT=1025
  SMTP_TLS_MODE=none

#### Resend (production-style)

  APP_ENV=production
  EMAIL_DRIVER=resend
  FROM_EMAIL=noreply@your-domain.com
  RESEND_API_KEY=re_xxxxxxxxxxxx

When `EMAIL_DRIVER=resend`, SMTP variables can stay unset.

#### Gmail SMTP (optional)

  EMAIL_DRIVER=smtp
  FROM_EMAIL=your_gmail@gmail.com
  SMTP_HOST=smtp.gmail.com
  SMTP_PORT=587
  SMTP_TLS_MODE=starttls
  SMTP_USERNAME=your_gmail@gmail.com
  SMTP_PASSWORD=your_16_char_app_password

Note: in non-production, Tornare protects against accidental leakage when using Gmail SMTP by redirecting outgoing emails to a fixed safety inbox.

### `frontend/.env`

| Variable | Description |
|---|---|
| `VITE_API_URL` | Backend base URL |

### `bot/.env`

| Variable | Description |
|---|---|
| `DATABASE_URL` | Postgres connection string (same as backend) |
| `DISCORD_BOT_TOKEN` | Bot token from the Discord developer portal |
| `DISCORD_BOT_APPLICATION_ID` | Application ID — required for `REGISTER_COMMANDS=true` |
| `FRONTEND_URL` | Base URL of the frontend (used for event deep-links in embeds) |
| `POLL_INTERVAL_SECS` | How often the bot polls for new events (default: `60`) |
| `REGISTER_COMMANDS` | Set to `true` once to register slash commands globally on Discord |

---

## 🛠️ Make commands

```
make bootstrap       Create local .env files from examples
make up              Build and run all services (production image)
make dev             Start postgres + backend hot-reload (cargo-watch) + frontend
make dev-no-migrate  Same as dev but skips DB migrations on start
make test            Run all backend tests
make test-e2e        Run the cross-domain end-to-end flow only
make bot-test        Run bot tests
make bot-dev         Start the bot with hot-reload
make bot-check       Cargo check the bot crate
make check           Cargo check the backend
make db-up           Start postgres only
make db-logs         Tail postgres logs
make shell           Open bash shell in the rust-dev container
make node-shell      Open shell in the Node dev container
make node-build      Build the frontend
make status          Show compose service status
make restart         Restart all services
make down            Stop and remove all services
```

---

## 🧪 Testing

Backend and bot tests are integration tests that run against a real PostgreSQL database. `sqlx::test` spins up a temporary schema per test — no manual setup needed as long as `DATABASE_URL` points to a live Postgres instance.

### Run everything

```bash
make test       # backend
make bot-test   # bot
```

### Run specific suites

```bash
# From the repo root or backend/
cargo test -p tornare --test users
cargo test -p tornare --test events
cargo test -p tornare --test discord
cargo test -p tornare --test battlenet
cargo test -p tornare --test e2e -- --test-threads=1

# Bot
cargo test -p tornare-bot --test announcements
```

### Test suites

| Suite | What it covers |
|---|---|
| `backend/tests/users.rs` | Registration, profile editing, admin account deletion |
| `backend/tests/events.rs` | Event lifecycle, roster/teams, auto-balance, signups, visibility |
| `backend/tests/discord.rs` | OAuth disconnect guard, guild REST API, takeover protection, soft-delete, interaction signature verification |
| `backend/tests/battlenet.rs` | Battle.net connect/disconnect and reconnect edge cases |
| `backend/tests/e2e.rs` | One full happy-path flow crossing auth + events |
| `bot/tests/announcements.rs` | `fetch_guilds` filters, pending event deduplication |

---

## 📁 Project layout

```
backend/
  migrations/          SQL migrations (applied in order at startup)
  src/
    app/               Axum router, state, security (rate limiting, JWT)
    features/
      auth/            JWT, OAuth (Battle.net, Discord), session management
      discord/         Guild management, slash-command interactions, REST handlers
      events/          Event lifecycle, roster, teams, matches, signups, brackets
      matches/         Match listing
      users/           User profiles
    shared/            DB helpers, error types, validation, crypto
  tests/               Integration test suites

bot/
  src/
    announcements.rs   Poll loop, pending event queries, embed posting
    discord.rs         Discord HTTP client, embed builder
    commands.rs        Slash command registration
    lib.rs             Public re-exports (for integration tests)
    main.rs            Entry point, config, poll interval

frontend/
  src/
    pages/             Route-level Vue components
    components/        Shared UI components
    composables/       Shared Vue composition logic
    stores/            Pinia state stores
    lib/               API client, utilities
    locales/           i18n strings (en, fr)
```

---

## 📡 API reference

### 🔑 Auth

```
POST   /api/auth/register
POST   /api/auth/login
GET    /api/auth/me
POST   /api/auth/refresh
POST   /api/auth/logout

GET    /api/auth/battlenet/authorize
GET    /api/auth/battlenet/callback
POST   /api/auth/battlenet/complete
GET    /api/auth/battlenet/connect-init
DELETE /api/auth/battlenet/disconnect

GET    /api/auth/discord/authorize
GET    /api/auth/discord/callback
GET    /api/auth/discord/connect-init
DELETE /api/auth/discord/disconnect
```

### 👤 Users

```
GET    /api/users/:id
PUT    /api/users/:id
DELETE /api/users/:id   (admin only)
```

### 🏆 Events

```
GET    /api/events
POST   /api/events
GET    /api/events/kpi
GET    /api/events/featured
GET    /api/events/:id
PUT    /api/events/:id
DELETE /api/events/:id

POST   /api/events/:id/players
PUT    /api/events/:id/players/:pid
DELETE /api/events/:id/players/:pid

POST   /api/events/:id/teams
POST   /api/events/:id/teams/auto-solo
POST   /api/events/:id/teams/auto-balance
PUT    /api/events/:id/teams/:tid
DELETE /api/events/:id/teams/:tid
POST   /api/events/:id/team-members

POST   /api/events/:id/matches
POST   /api/events/:id/matches/:mid/matchup
POST   /api/events/:id/matches/:mid/winner
POST   /api/events/:id/matches/:mid/winner/cancel
POST   /api/events/:id/matches/:mid/start-date

POST   /api/events/:id/tourney/generate
POST   /api/events/:id/tourney/clear

GET    /api/events/:id/signup-link
POST   /api/events/:id/signup-link/rotate
PUT    /api/events/:id/signup-visibility
PUT    /api/events/:id/featured
GET    /api/events/:id/signup-requests
POST   /api/events/:id/signup-requests/:rid/accept
POST   /api/events/:id/signup-requests/:rid/decline

GET    /api/public/event-signups/:token
POST   /api/public/event-signups/:token/requests
```

### 🤖 Discord

```
POST   /api/discord/interactions                          Slash-command webhook (ed25519-signed)
GET    /api/discord/invite                                Bot invite URL
GET    /api/discord/guilds                                List guilds owned by the authenticated user
PUT    /api/discord/guild                                 Register or update a guild
DELETE /api/discord/guild/:guild_id                       Soft-delete a guild
PATCH  /api/discord/guild/:guild_id/announcements         Toggle announcements on/off
GET    /api/discord/guild/:guild_id/members
POST   /api/discord/guild/:guild_id/members
DELETE /api/discord/guild/:guild_id/members/:user_id
```

### 🔍 Misc

```
GET    /health
GET    /api/matches
GET    /api/matches/:id
DELETE /api/matches/:id
```

> **Timestamp contract**: all `start_date` fields must be RFC3339 with an explicit timezone offset (e.g. `2026-03-17T19:30:00Z` or `2026-03-17T20:30:00+01:00`). Raw `datetime-local` strings (`2026-03-17T19:30`) are rejected with `400`.

---

## 🚢 Deployment

See [DEPLOY.md](DEPLOY.md) for a step-by-step guide targeting **Neon** (Postgres), **Render** (backend + bot), and **Vercel** (frontend).
