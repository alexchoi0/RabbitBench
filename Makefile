.PHONY: help dev db db-up db-down db-logs db-reset build check clean

help:
	@echo "Driftwatch Development Commands"
	@echo ""
	@echo "Development:"
	@echo "  make dev          - Start API server with database"
	@echo "  make dev-watch    - Start API server with hot reload"
	@echo ""
	@echo "Database:"
	@echo "  make db           - Start PostgreSQL (alias for db-up)"
	@echo "  make db-up        - Start PostgreSQL container"
	@echo "  make db-down      - Stop PostgreSQL container"
	@echo "  make db-logs      - View PostgreSQL logs"
	@echo "  make db-reset     - Reset database (destroy and recreate)"
	@echo "  make db-shell     - Open psql shell"
	@echo ""
	@echo "Build:"
	@echo "  make build        - Build release binary"
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
	cargo run -p driftwatch -- serve

dev-watch: db-up
	cargo watch -x "run -p driftwatch -- serve"

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

db-shell:
	docker exec -it driftwatch-db psql -U driftwatch -d driftwatch

# =============================================================================
# Build
# =============================================================================

build:
	cargo build --release

# =============================================================================
# Quality Checks
# =============================================================================

check:
	cargo check --all-targets
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt --all

test:
	cargo test --all-targets
	@docker ps -aq --filter "label=org.testcontainers.managed-by=testcontainers" 2>/dev/null | xargs -r docker rm -f 2>/dev/null || true

# =============================================================================
# Cleanup
# =============================================================================

clean:
	cargo clean
