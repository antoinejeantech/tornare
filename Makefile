.PHONY: help bootstrap up dev dev-no-migrate backend frontend db-up db-logs dev-up shell run check test test-e2e node-shell node-install node-build bot-dev bot-shell bot-check status restart down

help:
	@echo "Available targets:"
	@echo "  make bootstrap      - Create local .env files if missing"
	@echo "  make up             - Build and run backend + frontend (production image)"
	@echo "  make dev            - Start postgres + backend-dev (cargo-watch) + frontend"
	@echo "  make dev-no-migrate - Same as dev but skips database migrations on start"
	@echo "  make db-up          - Start postgres only"
	@echo "  make db-logs        - Tail postgres logs"
	@echo "  make backend        - Build and run backend API"
	@echo "  make frontend       - Run Vue frontend"
	@echo "  make dev-up         - Start rust-dev + node-dev containers"
	@echo "  make shell          - Open bash shell in rust-dev container"
	@echo "  make run            - Run cargo run in rust-dev container"
	@echo "  make check          - Run cargo check in rust-dev container"
	@echo "  make test           - Run all backend tests in rust-dev container"
	@echo "  make test-e2e       - Run end-to-end tests against an isolated temp DB on postgres"
	@echo "  make node-shell     - Open shell in Node dev container"
	@echo "  make node-install   - Install frontend deps"
	@echo "  make node-build     - Build frontend in node-dev container"
	@echo "  make bot-dev        - Start Discord bot with cargo-watch (hot-reload)"
	@echo "  make bot-shell      - Open bash shell in bot-dev container"
	@echo "  make bot-check      - Run cargo check on the bot crate"
	@echo "  make status         - Show compose service status"
	@echo "  make restart        - Restart backend + bot + frontend + postgres"
	@echo "  make down           - Stop and remove compose services"

bootstrap:
	@test -f backend/.env || cp backend/.env.example backend/.env
	@test -f frontend/.env || cp frontend/.env.example frontend/.env
	@test -f bot/.env || cp bot/.env.example bot/.env

up:
	docker compose up --build backend bot frontend

dev:
	docker compose stop backend 2>/dev/null || true
	docker compose up --force-recreate postgres backend-dev frontend

dev-no-migrate:
	docker compose stop backend 2>/dev/null || true
	SKIP_MIGRATIONS=1 docker compose up --force-recreate postgres backend-dev frontend

db-up:
	docker compose up -d postgres

db-logs:
	docker compose logs -f postgres

backend:
	docker compose up --build backend

frontend:
	docker compose up frontend

dev-up:
	docker compose up -d rust-dev node-dev

shell:
	docker compose exec rust-dev bash

run:
	docker compose exec rust-dev cargo run

check:
	docker compose exec rust-dev cargo check

test:
	docker compose up -d postgres rust-dev
	docker compose exec \
		-e DATABASE_URL=postgres://postgres:postgres@postgres:5432/postgres \
		rust-dev cargo test

test-e2e:
	docker compose up -d postgres rust-dev
	docker compose exec \
		-e DATABASE_URL=postgres://postgres:postgres@postgres:5432/postgres \
		rust-dev cargo test --test e2e -- --test-threads=1

status:
	docker compose ps

restart:
	docker compose up -d --build postgres backend bot frontend

down:
	docker compose down --remove-orphans

bot-dev:
	docker compose up --force-recreate bot-dev

bot-shell:
	docker compose exec bot-dev bash

bot-check:
	docker compose run --rm bot-dev cargo check

node-shell:
	docker compose exec node-dev bash

node-install:
	docker compose exec node-dev npm install

node-build:
	docker compose exec node-dev npm run build
