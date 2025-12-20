.PHONY: help dev db db-up db-down db-logs db-reset build check clean

# Default target
help:
	@echo "Driftwatch Development Commands"
	@echo ""
	@echo "Development:"
	@echo "  make dev          - Start API server with database"
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
	@echo "Build:"
	@echo "  make build        - Build API (release)"
	@echo ""
	@echo "Quality:"
	@echo "  make check        - Check and lint code"
	@echo "  make fmt          - Format code"
	@echo "  make test         - Run tests"
	@echo ""
	@echo "Setup:"
	@echo "  make clean        - Clean build artifacts"

# =============================================================================
# Development
# =============================================================================

dev: db-up
	@echo "Starting API server..."
	cd api && cargo run

dev-watch: db-up
	cd api && cargo watch -x run

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
	docker exec -i driftwatch-db psql -U driftwatch -d driftwatch < scripts/init-db.sql

db-shell:
	docker exec -it driftwatch-db psql -U driftwatch -d driftwatch

# =============================================================================
# Build
# =============================================================================

build:
	cd api && cargo build --release

# =============================================================================
# Quality Checks
# =============================================================================

check:
	cd api && cargo check
	cd api && cargo clippy -- -D warnings

fmt:
	cd api && cargo fmt

test:
	cd api && cargo test
	@docker ps -aq --filter "label=org.testcontainers.managed-by=testcontainers" 2>/dev/null | xargs -r docker rm -f 2>/dev/null || true

# =============================================================================
# Cleanup
# =============================================================================

clean:
	cd api && cargo clean
