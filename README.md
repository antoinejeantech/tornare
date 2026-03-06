# Overwatch Pickup Game Manager (Rust + Vue + Docker)

This workspace is a minimal full-stack setup for pickup games:

- PostgreSQL database for persistence
- Rust backend API (Axum) on port `8000`
- Vue frontend (Vite) on port `5173`
- Everything runs in Docker
- Create events (PUG/TOURNEY), build one event roster/teams, then play multiple games with shared teams

## Quick start

```bash
make up
```

Open:

- Frontend: http://localhost:5173
- Backend health: http://localhost:8000/health
- Games API: http://localhost:8000/api/games

## Environment setup

Create local env files from the provided examples:

```bash
cp backend/.env.example backend/.env
cp frontend/.env.example frontend/.env
```

Backend variables (`backend/.env`):

- `DATABASE_URL`: Postgres connection string
- `JWT_SECRET`: signing secret for auth tokens

Frontend variables (`frontend/.env`):

- `VITE_API_URL`: backend base URL used by the frontend

## Make commands

```bash
make help
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
make down
```

## Compile and run

- Build + run backend only: `make backend`
- Start frontend only: `make frontend`
- Run Rust app in dev container: `make run`
- Compile checks: `make check`
- Run backend tests: `make test`

## Backend endpoints

- `GET /api/events` list all events
- `POST /api/events` create an event (`PUG` or `TOURNEY`)
- `GET /api/events/:event_id` get one event with all games
- `DELETE /api/events/:event_id` delete an event and all its games
- `POST /api/events/:event_id/games` create a game in an event
- `POST /api/events/:event_id/players` add player to event roster
- `DELETE /api/events/:event_id/players/:player_id` remove player from event roster
- `POST /api/events/:event_id/teams` create a team in an event
- `DELETE /api/events/:event_id/teams/:team_id` delete a team from an event
- `POST /api/events/:event_id/team-members` assign or clear a player's team in an event
- `POST /api/events/:event_id/games/:game_id/matchup` set two competing teams for a game
- `GET /api/games` list all pickup games
- `POST /api/games` create a game
- `GET /api/games/:game_id` get one game
- `DELETE /api/games/:game_id` delete one game
- `POST /api/games/:game_id/players` add player to game
- `DELETE /api/games/:game_id/players/:player_id` remove player from game
- `POST /api/games/:game_id/teams` assign or clear player team

For games linked to an event, game player endpoints operate on the shared event roster.

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
