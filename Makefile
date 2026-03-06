.PHONY: help up backend frontend db-up db-logs dev-up shell run check test node-shell node-install down

help:
	@echo "Available targets:"
	@echo "  make up          - Build and run backend + frontend"
	@echo "  make db-up       - Start postgres only"
	@echo "  make db-logs     - Tail postgres logs"
	@echo "  make backend     - Build and run backend API"
	@echo "  make frontend    - Run Vue frontend"
	@echo "  make dev-up  - Start dev container in background"
	@echo "  make shell   - Open bash shell in dev container"
	@echo "  make run     - Run cargo run in dev container"
	@echo "  make check   - Run cargo check in dev container"
	@echo "  make test    - Run cargo test in dev container"
	@echo "  make node-shell  - Open shell in Node dev container"
	@echo "  make node-install - Install frontend deps"
	@echo "  make down    - Stop and remove compose services"

up:
	docker compose up --build backend frontend

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
	docker compose exec rust-dev cargo test

down:
	docker compose down --remove-orphans

node-shell:
	docker compose exec node-dev bash

node-install:
	docker compose exec node-dev npm install
