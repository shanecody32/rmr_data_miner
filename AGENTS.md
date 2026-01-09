# Repository Guidelines

## Project Structure & Module Organization
- `backend/`: Rust Axum API, background poller, SeaORM entities (`backend/src/api`, `backend/src/poller`, `backend/src/entities`).
- `migration/`: SeaORM migrations and Migrator (`migration/src`).
- `frontend/`: Next.js App Router UI (`frontend/app`), shared styles (`frontend/app/globals.css`), assets (`frontend/public`).
- `docker-compose.yml`: Local stack (API, UI, Postgres).

## Build, Test, and Development Commands
- `make run-db`: Start Postgres via Docker.
- `make run-backend`: Run the Rust API on port 8015.
- `make run-frontend`: Run the Next.js dev server on port 3015.
- `make docker-up`: Build and start the full stack.
- `cd frontend && npm run build`: Production build for the UI.
- `cd frontend && npm run lint`: Run ESLint.

## Coding Style & Naming Conventions
- Rust: 4-space indentation, idiomatic `rustfmt` defaults, modules grouped by feature (`api`, `poller`, `entities`).
- TypeScript/React: 2-space indentation, double quotes, PascalCase components (e.g., `Layout.tsx`).
- Routes live under `frontend/app/.../page.tsx`; keep file names aligned with route paths.

## Testing Guidelines
- No dedicated test suite yet; add unit tests alongside Rust modules using `#[cfg(test)]`.
- Use `cargo test` for backend tests and `npm run lint` for frontend checks.
- Prefer naming new tests by behavior (e.g., `stores_new_payload_on_change`).

## Commit & Pull Request Guidelines
- Git history is minimal; no formal convention yet. Use concise, imperative subjects (e.g., “Add stations API pagination”).
- PRs should include a brief description, relevant screenshots for UI changes, and any local run notes (`make run-backend`, `make run-frontend`).

## Configuration & Environment
- Backend expects `DATABASE_URL` in `.env` (see `backend/src/main.rs`).
- Default local ports: API 8015, UI 3015, Postgres 5433.
