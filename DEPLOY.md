# Deploy Guide (Vercel + Render + Neon)

## 1. Neon (Postgres)

1. Create a Neon project and database.
2. Copy a connection string with SSL enabled.
3. Ensure the URL includes `sslmode=require`.
4. Prefer Neon pooled connection URL for production app traffic.

Example:

`postgresql://USER:PASSWORD@HOST/DB?sslmode=require`

## 2. Render (Backend)

Create a Web Service from `backend/`.

- Build command: `cargo build --release`
- Start command: `./target/release/tornare`
- Health check path: `/health`

Set environment variables:

- `APP_ENV=production`
- `DATABASE_URL=<neon_connection_url>`
- `JWT_SECRET=<long_random_secret>`
- `CORS_ALLOWED_ORIGINS=<comma_separated_frontend_origins>`

Example:

`CORS_ALLOWED_ORIGINS=https://tornare.vercel.app,https://tornare-git-main-yourteam.vercel.app`

Notes:

- In production mode the backend now fails fast if required env vars are missing.
- Keep `JWT_SECRET` unique per environment.

## 3. Vercel (Frontend)

Create a Vercel project from `frontend/`.

- Framework preset: Vite
- Build command: `npm run build`
- Output directory: `dist`

Set environment variable:

- `VITE_API_URL=<render_backend_url>`

Example:

`VITE_API_URL=https://tornare-api.onrender.com`

`frontend/vercel.json` is included to rewrite all routes to `index.html` for Vue Router history mode.

## 4. First Deploy Validation

1. Open frontend URL and confirm app loads.
2. Check backend health endpoint: `/health`.
3. Verify auth flow (register/login/refresh/logout).
4. Verify event list/create/edit works with Neon-backed data.
5. Confirm no CORS errors in browser network panel.

## 5. Post-Launch Hardening

- Replace startup schema-init with explicit SQL migrations.
- Move from localStorage auth token handling to HttpOnly cookies.
- Add structured logging and request IDs.
- Add uptime checks for backend and DB connectivity.
