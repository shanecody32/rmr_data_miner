.PHONY: run-db run-backend run-frontend migrate

run-db:
	docker compose up -d db

run-backend:
	cd backend && cargo run

run-frontend:
	cd frontend && npm run dev

migrate:
	cd backend && cargo run -- --migrate # If I added a migrate flag, but I'll use Migrator::up in main.rs
	# Alternatively, if using sea-orm-cli:
	# sea-orm-cli migrate up

setup:
	docker compose up -d db
	@echo "Database started. Run 'make run-backend' and 'make run-frontend' in separate terminals."

docker-up:
	docker compose up --build
