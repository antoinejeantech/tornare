# Deploy Guide (Vercel + Render + Neon)

## 1. Neon (Postgres)

1. Create a Neon project and database.
2. Copy a connection string with SSL enabled — ensure the URL includes `sslmode=require`.
3. Use the **pooled** connection URL for the backend and bot (handles concurrent connections).
4. Use the **direct** connection URL for running migrations locally or via CI.

Example:

```
postgresql://USER:PASSWORD@HOST/DB?sslmode=require
```

## 2. Discord application setup

The backend verifies slash-command interactions using ed25519 signatures. This requires a Discord application with a bot user.

1. Go to https://discord.com/developers/applications and create a new application.
2. Under **Bot**, copy the **bot token** (`DISCORD_BOT_TOKEN`).
3. Under **General Information**, copy the **Application ID** (`DISCORD_BOT_APPLICATION_ID`) and **Public Key** (`DISCORD_BOT_PUBLIC_KEY`).
4. Under **OAuth2**, add a redirect URI matching your backend URL: `https://<backend>/api/auth/discord/callback`. Copy the **Client ID** and **Client Secret**.
5. Under **Installation → Guild Install**, add the `bot` and `applications.commands` scopes plus the `Administrator` permission (required for `/setup` to verify the invoking member has admin rights).
6. Set the **Interactions Endpoint URL** to `https://<backend>/api/discord/interactions` — Discord will verify the endpoint with a `PING` before saving.

> The interactions endpoint will reject every request until `DISCORD_BOT_PUBLIC_KEY` is set correctly. The service fails closed: a missing or wrong key returns 401.

## 3. Battle.net OAuth (optional)

1. Go to https://develop.battle.net and create an OAuth client.
2. Add a redirect URI: `https://<backend>/api/auth/battlenet/callback`.
3. Copy the **Client ID** and **Client Secret**.

## 4. Render (Backend)

Create a **Web Service** from `backend/`.

| Setting | Value |
|---|---|
| Build command | `cargo build --release` |
| Start command | `./target/release/tornare` |
| Health check path | `/health` |

Environment variables:

| Variable | Value |
|---|---|
| `APP_ENV` | `production` |
| `DATABASE_URL` | Neon pooled connection URL |
| `JWT_SECRET` | Long random secret (generate with `openssl rand -hex 64`) |
| `CORS_ALLOWED_ORIGINS` | Comma-separated frontend origins |
| `DISCORD_BOT_PUBLIC_KEY` | Ed25519 public key from Discord developer portal |
| `DISCORD_BOT_TOKEN` | Bot token |
| `DISCORD_CLIENT_ID` | Discord OAuth client ID |
| `DISCORD_CLIENT_SECRET` | Discord OAuth client secret |
| `DISCORD_REDIRECT_URI` | `https://<backend>/api/auth/discord/callback` |
| `BATTLENET_CLIENT_ID` | Battle.net client ID (if using Battle.net OAuth) |
| `BATTLENET_CLIENT_SECRET` | Battle.net client secret |
| `BATTLENET_REDIRECT_URI` | `https://<backend>/api/auth/battlenet/callback` |

Example `CORS_ALLOWED_ORIGINS`:

```
https://tornare.vercel.app,https://tornare-git-main-yourteam.vercel.app
```

> The backend fails fast on startup if any required env var is missing in production mode.

## 5. Render (Bot)

Create a **Background Worker** from `bot/`.

| Setting | Value |
|---|---|
| Build command | `cargo build --release` |
| Start command | `./target/release/tornare-bot` |

Environment variables:

| Variable | Value |
|---|---|
| `DATABASE_URL` | Neon pooled connection URL (same DB as backend) |
| `DISCORD_BOT_TOKEN` | Bot token |
| `DISCORD_BOT_APPLICATION_ID` | Application ID (required to register slash commands) |
| `FRONTEND_URL` | Frontend base URL — used for event deep-links in embeds |
| `POLL_INTERVAL_SECS` | How often the bot polls for new events (default: `60`) |
| `REGISTER_COMMANDS` | Set to `true` on the first deploy only, then remove — registers slash commands globally on Discord |

> The bot and backend share the same Postgres database. Run the bot as a separate Render service so it can be scaled or restarted independently.

## 6. Vercel (Frontend)

Create a **Vercel project** from `frontend/`.

| Setting | Value |
|---|---|
| Framework preset | Vite |
| Build command | `npm run build` |
| Output directory | `dist` |

Environment variable:

| Variable | Value |
|---|---|
| `VITE_API_URL` | Backend Render URL, e.g. `https://tornare-api.onrender.com` |

`frontend/vercel.json` rewrites all routes to `index.html` for Vue Router history mode.

## 7. First deploy validation

1. Open the frontend URL and confirm the app loads.
2. Check the backend health endpoint: `GET /health`.
3. Verify the auth flow: register → login → refresh → logout.
4. Open the Discord guild page and confirm the bot invite URL is reachable.
5. In a test Discord server, run `/setup` in a channel — verify the guild appears in the frontend.
6. Confirm event announcements are posted when an event is set to `ACTIVE`.
7. Check no CORS errors in the browser network panel.

## 8. Post-launch hardening

- Move from `localStorage` auth token storage to `HttpOnly` cookies.
- Add structured logging with request IDs to the backend.
- Set up uptime monitors for the backend health endpoint and bot worker.
- Rotate `JWT_SECRET` and `DISCORD_BOT_TOKEN` on a schedule or after any suspected exposure.
