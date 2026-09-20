# Actix Supabase API

An asynchronous Rust HTTP API using Actix Web, Tokio, SQLx, and Supabase PostgreSQL.

## Run locally

1. Copy `.env.example` to `.env` and set `SUPABASE_DATABASE_URL`.
2. Start the API:

```sh
cargo run
```

The server listens on `http://localhost:8080` by default. SQLx runs the migration in `migrations/` at startup. `DATABASE_URL` is also accepted as a local-development fallback.

## Run with Docker

```sh
cp .env.example .env
# Set DATABASE_URL in .env
docker compose up --build
```

The runtime image includes CA certificates and libssl for the Supabase connection. The builder uses a dependency-only Cargo layer so source-only changes reuse compiled dependencies.

## Endpoints

- `GET /health`
- `POST /api/v1/users`
- `GET /api/v1/users`
- `GET /api/v1/users/{id}`
- `PUT /api/v1/users/{id}`
- `DELETE /api/v1/users/{id}`

Example:

```sh
curl -X POST http://localhost:8080/api/v1/users \
  -H 'content-type: application/json' \
  -d '{"name":"Ada Lovelace","email":"ada@example.com"}'
```