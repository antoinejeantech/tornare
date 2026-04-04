# Tornare

A full-stack platform for organizing competitive Overwatch events — pick-up games and tournaments — with a Discord bot that posts live announcements to your servers.

**Stack:** Rust (Axum) · PostgreSQL · Vue 3 · Discord Interactions API · Docker

---

## Features

### Event management
Create **PUG** (pick-up game) or **TOURNEY** events. Add players with roles and preferred positions, build teams, run matches, track winners, and generate tournament brackets. An event has a full lifecycle: `DRAFT → ACTIVE → ENDED`.

### Smart roster & team tools
- **Auto-balance**: distributes players across teams by role (Tank / DPS / Support) respecting Overwatch slot quotas.
- **Auto-solo teams**: one team per player for free-for-all formats.
- **Public signup links**: shareable tokenized URLs let outsiders request to join — owners accept or decline from the dashboard.

### Discord bot
The bot runs as a separate process, polling the database on a configurable interval and posting rich embeds to announcement channels when new events go live.

Guilds are self-served through slash commands:

| Command | What it does |
|---|---|
| `/setup [channel]` | Connect a Discord server — requires **Administrator** permission and a linked Tornare account |
| `/unsetup` | Disconnect the server |
| `/help` | Show available commands |

The backend receives slash-command interactions via a **ed25519-signed webhook** (`POST /api/discord/interactions`). Signatures are verified on every request; an unconfigured public key fails closed (401).

**Guild ownership protection**: a guild registered by one user cannot be claimed by another via `/setup` as long as it remains active. Removing a guild is a soft-delete — the announcement history (`discord_guild_posts`) is preserved, and the guild ID can be re-claimed after removal.

### Auth
JWT-based session with refresh tokens. Supports email/password and OAuth via **Battle.net** and **Discord**. Lockout prevention: disconnecting an OAuth provider is blocked if it's the user's only login method.

---

## Quick start

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

## Environment variables

### `backend/.env`

| Variable | Description |
|---|---|
| `DATABASE_URL` | Postgres connection string |
| `JWT_SECRET` | Signing secret for auth tokens |
| `CORS_ALLOWED_ORIGINS` | Comma-separated allowed origins (e.g. `https://app.example.com,http://localhost:5173`) |
| `DISCORD_BOT_PUBLIC_KEY` | Ed25519 public key from the Discord developer portal — **required** for slash commands |
| `DISCORD_BOT_TOKEN` | Bot token — used to verify channel permissions at `/setup` time |
| `DISCORD_CLIENT_ID` | Discord OAuth app client ID |
| `DISCORD_CLIENT_SECRET` | Discord OAuth app client secret |
| `DISCORD_REDIRECT_URI` | Discord OAuth redirect URI |
| `BATTLENET_CLIENT_ID` | Battle.net OAuth app client ID |
| `BATTLENET_CLIENT_SECRET` | Battle.net OAuth app client secret |
| `BATTLENET_REDIRECT_URI` | Battle.net OAuth redirect URI |

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

## Make commands

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

## Testing

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

## Project layout

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

## API reference

### Auth

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

### Events

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

### Discord bot

```
POST   /api/discord/interactions           Slash-command webhook (ed25519-signed)
GET    /api/discord/invite                 Bot invite URL
GET    /api/discord/guilds                 List guilds owned by the authenticated user
PUT    /api/discord/guild                  Register or update a guild
DELETE /api/discord/guild/:guild_id        Soft-delete a guild
PATCH  /api/discord/guild/:guild_id/announcements   Toggle announcements on/off
GET    /api/discord/guild/:guild_id/members
POST   /api/discord/guild/:guild_id/members
DELETE /api/discord/guild/:guild_id/members/:user_id
```

### Misc

```
GET    /health
GET    /api/users/:id
PUT    /api/users/:id
DELETE /api/users/:id   (admin only)
GET    /api/matches
GET    /api/matches/:id
DELETE /api/matches/:id
```

> **Timestamp contract**: all `start_date` fields must be RFC3339 with an explicit timezone offset (e.g. `2026-03-17T19:30:00Z` or `2026-03-17T20:30:00+01:00`). Raw `datetime-local` strings (`2026-03-17T19:30`) are rejected with `400`.

---

## Deployment

See [DEPLOY.md](DEPLOY.md) for a step-by-step guide targeting **Neon** (Postgres), **Render** (backend + bot), and **Vercel** (frontend).


- PostgreSQL database for persistence
- Rust backend API (Axum) on port `8000`
- Vue frontend (Vite) on port `5173`
- Discord bot — posts event embeds when events are published
- Everything runs in Docker
- Create events (PUG/TOURNEY), build one event roster/teams, then manage multiple matches with shared teams

## Quick start

```bash
make up
```

Open:

- Frontend: http://localhost:5173
- Backend health: http://localhost:8000/health
- Matches API: http://localhost:8000/api/matches

## Environment setup

Create local env files from the provided examples:

```bash
cp backend/.env.example backend/.env
cp frontend/.env.example frontend/.env
cp bot/.env.example bot/.env
```

Or use `make bootstrap` to create all three at once.

Backend variables (`backend/.env`):

- `DATABASE_URL`: Postgres connection string
- `JWT_SECRET`: signing secret for auth tokens
- `CORS_ALLOWED_ORIGINS`: comma-separated allowed frontend origins (e.g. `https://app.example.com,http://localhost:5173`)

Frontend variables (`frontend/.env`):

- `VITE_API_URL`: backend base URL used by the frontend

Discord bot variables (`bot/.env`):

- `DATABASE_URL`: Postgres connection string (same as backend)
- `DISCORD_BOT_TOKEN`: bot token from the Discord developer portal
- `DISCORD_CHANNEL_ID`: ID of the channel where event announcements are posted
- `FRONTEND_URL`: base URL of the frontend (used to build event deep-links in embeds)

## Make commands

```bash
make help
make bootstrap
make up
make db-up
make db-logs
make backend
make frontend
make dev-up
make shell
make run
make check
make test
make test-e2e
make node-shell
make node-install
make node-build
make bot-dev
make bot-shell
make bot-check
make status
make restart
make down
```

## Compile and run

- Build + run backend only: `make backend`
- Start frontend only: `make frontend`
- Create missing env files from examples: `make bootstrap`
- Start Discord bot with hot-reload: `make bot-dev`
- Open shell in bot container: `make bot-shell`
- Cargo check on the bot crate: `make bot-check`
- Run Rust app in dev container: `make run`
- Compile checks: `make check`
- Run all backend tests: `make test`
- Run the cross-domain end-to-end flow only: `make test-e2e`
- Build frontend in node-dev container: `make node-build`

## Testing

Backend integration tests are split by domain:

- `backend/tests/users.rs`: registration, profile editing, admin account deletion
- `backend/tests/events.rs`: event lifecycle, roster/teams, signups, auto-balance, visibility
- `backend/tests/battlenet.rs`: Battle.net connect/disconnect and reconnect edge cases
- `backend/tests/e2e.rs`: one full happy-path flow that crosses auth + events

Run everything:

```bash
make test
```

`make test` starts `postgres` and `rust-dev` first, then runs the full backend test suite with the container-safe `DATABASE_URL`.

Run the dedicated end-to-end flow only:

```bash
make test-e2e
```

Run directly with Cargo from `backend/`:

```bash
cargo test
cargo test --test users
cargo test --test events
cargo test --test battlenet
cargo test --test e2e -- --test-threads=1
```

If you are not running inside Docker, export a valid `DATABASE_URL` before running integration tests.

## Backend endpoints

Timestamp contract:

- `start_date` for event create/update and match create/update must be sent as RFC3339 / ISO8601 with an explicit timezone offset, for example `2026-03-17T19:30:00Z` or `2026-03-17T20:30:00+01:00`.
- The API normalizes accepted timestamps to UTC before storage.
- Event and match responses return stable UTC RFC3339 / ISO8601 timestamp strings, for example `2026-03-17T19:30:00Z` or `2026-03-17T19:30:00.123456Z`.
- Do not send raw `datetime-local` values like `2026-03-17T19:30`; they are rejected as `400 Bad Request`.

- `GET /health` health check
- `GET /api/hello` basic API smoke test

- `POST /api/auth/register` register a user
- `POST /api/auth/login` log in
- `GET /api/auth/me` fetch the authenticated user
- `POST /api/auth/refresh` refresh auth tokens
- `POST /api/auth/logout` log out
- `GET /api/auth/battlenet/authorize` start Battle.net login
- `GET /api/auth/battlenet/callback` handle Battle.net OAuth callback
- `POST /api/auth/battlenet/complete` finish Battle.net signup after email collection
- `GET /api/auth/battlenet/connect-init` start Battle.net connect flow for an existing account (browser redirect, pass `?token=<access_token>`)
- `DELETE /api/auth/battlenet/disconnect` disconnect the linked Battle.net account

- `GET /api/users/:user_id` fetch a user profile
- `PUT /api/users/:user_id` update a user profile
- `DELETE /api/users/:user_id` delete a user profile (admin only)

- `GET /api/events` list events
- `POST /api/events` create an event (`PUG` or `TOURNEY`, optional `start_date` must follow the timestamp contract above)
- `GET /api/events/kpi` fetch event KPI aggregates
- `GET /api/events/featured` fetch the featured event
- `GET /api/events/:event_id` fetch one event with roster, teams, and matches
- `PUT /api/events/:event_id` update an event (`start_date` follows the same timestamp contract)
- `DELETE /api/events/:event_id` delete an event

- `POST /api/events/:event_id/matches` create a match in an event (optional `start_date` follows the same timestamp contract)
- `POST /api/events/:event_id/matches/:match_id/matchup` set or clear the two teams for a match
- `POST /api/events/:event_id/matches/:match_id/winner` report a match winner
- `POST /api/events/:event_id/matches/:match_id/winner/cancel` cancel a reported winner
- `POST /api/events/:event_id/matches/:match_id/start-date` set or clear a match start date (`start_date` follows the same timestamp contract)

- `POST /api/events/:event_id/players` add a player to the event roster
- `PUT /api/events/:event_id/players/:player_id` update an event player
- `DELETE /api/events/:event_id/players/:player_id` remove a player from the event roster

- `POST /api/events/:event_id/teams` create a team in an event
- `POST /api/events/:event_id/teams/auto-solo` create solo teams automatically
- `POST /api/events/:event_id/teams/auto-balance` auto-balance teams
- `PUT /api/events/:event_id/teams/:team_id` update a team
- `DELETE /api/events/:event_id/teams/:team_id` delete a team
- `POST /api/events/:event_id/team-members` assign or clear a player's team in an event

- `POST /api/events/:event_id/tourney/generate` generate the tournament bracket
- `POST /api/events/:event_id/tourney/clear` clear the tournament bracket

- `GET /api/events/:event_id/signup-link` fetch the event signup link
- `POST /api/events/:event_id/signup-link/rotate` rotate the signup token
- `PUT /api/events/:event_id/signup-visibility` enable or disable public signup
- `PUT /api/events/:event_id/featured` mark or unmark an event as featured
- `GET /api/events/:event_id/signup-requests` list pending signup requests
- `POST /api/events/:event_id/signup-requests/:request_id/accept` accept a signup request
- `POST /api/events/:event_id/signup-requests/:request_id/decline` decline a signup request

- `GET /api/public/event-signups/:signup_token` fetch public signup page info
- `POST /api/public/event-signups/:signup_token/requests` create a public signup request

- `GET /api/matches` list visible matches
- `GET /api/matches/:match_id` fetch one match
- `DELETE /api/matches/:match_id` delete one match

## Database

- Service name: `postgres`
- DB: `tornare`
- User/password: `postgres` / `postgres`
- Connection string inside Docker: `postgres://postgres:postgres@postgres:5432/tornare`

## Project layout

- `backend/src/main.rs`: Rust backend server
- `backend/Cargo.toml`: Rust dependencies and crate config
- `bot/src/main.rs`: Discord bot (Postgres LISTEN/NOTIFY → Discord embeds)
- `bot/Cargo.toml`: bot crate dependencies
- `frontend/`: Vue app
- `docker-compose.yml`: service wiring
- `Makefile`: shortcut commands
