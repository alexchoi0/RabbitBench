.PHONY: help dev dev-api dev-ui db db-up db-down db-logs db-reset migrate build build-api build-ui check check-api check-ui clean install

# Default target
help:
	@echo "Driftwatch Development Commands"
	@echo ""
	@echo "Development:"
	@echo "  make dev          - Start all services (db, api, ui)"
	@echo "  make dev-api      - Start Rust API server only"
	@echo "  make dev-ui       - Start SvelteKit dev server only"
	@echo ""
	@echo "Database:"
	@echo "  make db           - Start PostgreSQL (alias for db-up)"
	@echo "  make db-up        - Start PostgreSQL container"
	@echo "  make db-down      - Stop PostgreSQL container"
	@echo "  make db-logs      - View PostgreSQL logs"
	@echo "  make db-reset     - Reset database (destroy and recreate)"
	@echo "  make db-init      - Initialize database tables"
	@echo "  make db-shell     - Open psql shell"
	@echo ""
	@echo "Migrations:"
	@echo "  make migrate      - Run Prisma migrations"
	@echo "  make migrate-gen  - Generate Prisma client"
	@echo ""
	@echo "Build:"
	@echo "  make build        - Build all projects"
	@echo "  make build-api    - Build Rust API (release)"
	@echo "  make build-ui     - Build SvelteKit"
	@echo ""
	@echo "Quality:"
	@echo "  make check        - Type check all projects"
	@echo "  make check-api    - Check Rust API"
	@echo "  make check-ui     - Check SvelteKit"
	@echo "  make fmt          - Format all code"
	@echo ""
	@echo "Setup:"
	@echo "  make install      - Install all dependencies"
	@echo "  make clean        - Clean build artifacts"

# =============================================================================
# Development
# =============================================================================

dev: db-up
	@echo "Starting development servers..."
	@trap 'kill 0' INT; \
		(cd api && cargo run) & \
		(cd server-svelte && npm run dev) & \
		wait

dev-api:
	cd api && cargo run

dev-api-watch:
	cd api && cargo watch -x run

dev-ui:
	cd server-svelte && npm run dev

# =============================================================================
# Database
# =============================================================================

db: db-up

db-up:
	docker compose up -d postgres
	@echo "Waiting for PostgreSQL to be ready..."
	@until docker exec driftwatch-db pg_isready -U driftwatch > /dev/null 2>&1; do \
		sleep 1; \
	done
	@echo "PostgreSQL is ready!"

db-down:
	docker compose down

db-logs:
	docker compose logs -f postgres

db-reset:
	docker compose down -v
	$(MAKE) db-up
	@sleep 2
	$(MAKE) db-init

db-init:
	cd server-svelte && npx prisma db push
	docker exec -i driftwatch-db psql -U driftwatch -d driftwatch < scripts/init-db.sql

db-shell:
	docker exec -it driftwatch-db psql -U driftwatch -d driftwatch

# =============================================================================
# Migrations
# =============================================================================

migrate:
	cd server-svelte && npx prisma migrate deploy

migrate-dev:
	cd server-svelte && npx prisma migrate dev

migrate-gen:
	cd server-svelte && npx prisma generate

# =============================================================================
# Build
# =============================================================================

build: build-api build-ui

build-api:
	cd api && cargo build --release

build-ui:
	cd server-svelte && npm run build

# =============================================================================
# Quality Checks
# =============================================================================

check: check-api check-ui

check-api:
	cd api && cargo check
	cd api && cargo clippy -- -D warnings

check-ui:
	cd server-svelte && npm run check

fmt:
	cd api && cargo fmt
	cd server-svelte && npm run format 2>/dev/null || true

test:
	cd api && cargo test
	cd server-svelte && npm test 2>/dev/null || true
	@docker ps -aq --filter "label=org.testcontainers.managed-by=testcontainers" 2>/dev/null | xargs -r docker rm -f 2>/dev/null || true

# =============================================================================
# Setup & Cleanup
# =============================================================================

install:
	cd server-svelte && npm install
	cd api && cargo fetch

clean:
	cd api && cargo clean
	cd server-svelte && rm -rf .svelte-kit node_modules/.vite

# =============================================================================
# Production
# =============================================================================

prod-build: build
	@echo "Production build complete"
	@echo "  API binary: api/target/release/driftwatch-api"
	@echo "  UI build: server-svelte/build/"
