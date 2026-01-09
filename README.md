# Radio Airplay Raw Data Collector

A system to collect and view raw "Now Playing" metadata from radio stations.

## Tech Stack
- **Backend**: Rust (Axum, SeaORM, PostgreSQL, Tokio)
- **Frontend**: Next.js (App Router, Tailwind CSS)
- **Database**: PostgreSQL

## Features
1. **Station Management**: Add and manage radio stations.
2. **Connection Management**: Configure polling connections (URL, interval, headers).
3. **Background Poller**: Continuously polls enabled connections and stores raw payloads.
4. **Deduplication**: Only stores new events if the payload has changed.
5. **Raw Event Viewer**: View collected events and their full raw payloads.

## Getting Started

### Prerequisites
- Docker and Docker Compose
- Rust (optional, for local development)
- Node.js (optional, for local development)

### Quick Start with Docker
```bash
docker-compose up --build
```
- Frontend: [http://localhost:3015](http://localhost:3015)
- Backend API: [http://localhost:8015](http://localhost:8015)
- Database (Host): `localhost:5433`

### Local Development

1. **Database**:
   ```bash
   make run-db
   ```

2. **Backend**:
   ```bash
   make run-backend
   ```
   (Ensure `.env` is configured correctly)

3. **Frontend**:
   ```bash
   make run-frontend
   ```

## Domain Model
- `stations`: Basic info about radio stations.
- `now_playing_connections`: Configuration for how to fetch data for a station.
- `raw_now_playing_events`: The actual collected data, stored exactly as received.

## API Endpoints
- `GET /api/stations`: List stations
- `POST /api/stations`: Create station
- `GET /api/connections`: List connections
- `POST /api/connections`: Create connection
- `POST /api/connections/:id/test`: Fetch and return current payload without storing
- `GET /api/events`: List raw events
- `GET /api/events/:id`: View event details including full raw payload

## Recommended Headers for Connections
When adding a new connection, some stations require specific headers to bypass basic bot protection or to specify the content type.

### Example Headers (JSON)
For stations using standard web APIs, you might need headers similar to these:
```json
{
  "User-Agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36",
  "Accept": "*/*",
  "Referer": "https://kcynfm.com/",
  "Accept-Language": "en-US,en;q=0.9"
}
```

### Common Headers
- `User-Agent`: Mimicking a real browser is often necessary.
- `Referer`: Some APIs check if the request comes from their own website.
- `Accept`: Usually `application/json` or `*/*`.
