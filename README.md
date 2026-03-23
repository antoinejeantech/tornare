# Overwatch Match Manager (Rust + Vue + Docker)

This workspace is a minimal full-stack setup for Overwatch event matches:

- PostgreSQL database for persistence
- Rust backend API (Axum) on port `8000`
- Vue frontend (Vite) on port `5173`
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
```

Backend variables (`backend/.env`):

- `DATABASE_URL`: Postgres connection string
- `JWT_SECRET`: signing secret for auth tokens
- `CORS_ALLOWED_ORIGINS`: comma-separated allowed frontend origins (e.g. `https://app.example.com,http://localhost:5173`)

Frontend variables (`frontend/.env`):

- `VITE_API_URL`: backend base URL used by the frontend

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
make node-shell
make node-install
make node-build
make status
make restart
make down
```

## Compile and run

- Build + run backend only: `make backend`
- Start frontend only: `make frontend`
- Create missing env files from examples: `make bootstrap`
- Run Rust app in dev container: `make run`
- Compile checks: `make check`
- Run backend tests: `make test`
- Build frontend in node-dev container: `make node-build`

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

- `GET /api/users/:user_id` fetch a user profile
- `PUT /api/users/:user_id` update a user profile

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
- `frontend/`: Vue app
- `docker-compose.yml`: service wiring
- `Makefile`: shortcut commands
