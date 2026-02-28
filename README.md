# PixelWar

A distributed pixel art competition platform built with Rust microservices. Inspired by Reddit's r/place — 10,000 people simultaneously draw on a shared 10k×10k canvas, competing for the best artwork.

## Architecture

```
┌─────────────┐     ┌──────────────────────────────────────────────┐
│   Clients   │────▶│              API Gateway                     │
│  (Browser)  │◀────│  :3000  (REST + WebSocket)                   │
└─────────────┘     └──────┬───────┬───────┬───────┬───────┬───────┘
                           │       │       │       │       │
                    ┌──────▼──┐ ┌──▼───┐ ┌─▼────┐ ┌▼─────┐┌▼──────┐
                    │  Auth   │ │Canvas│ │ Chat │ │Voting││ Group │
                    │ :3001   │ │:3002 │ │:3003 │ │:3004 ││ :3005 │
                    └────┬────┘ └──┬───┘ └──┬───┘ └──┬───┘└──┬────┘
                         │        │        │        │       │
                    ┌────▼────────▼────────▼────────▼───────▼────┐
                    │              NATS (Event Bus)               │
                    └────────────────────┬───────────────────────┘
                                         │
                                   ┌─────▼──────┐
                                   │  Scheduler  │
                                   │   :3006     │
                                   └─────────────┘

┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────┐
│ PostgreSQL  │  │    Redis    │  │  Observability Stack         │
│   :5432     │  │   :6379     │  │  Jaeger :16686 | Prom :9090  │
│             │  │  (canvas +  │  │  Grafana :3100               │
│  (all data) │  │   cache)    │  └─────────────────────────────┘
└─────────────┘  └─────────────┘
```

## Services

| Service       | Port | Description                                           |
|---------------|------|-------------------------------------------------------|
| api-gateway   | 3000 | REST API + WebSocket proxy, auth middleware, rate limiting |
| auth-service  | 3001 | Registration, login, JWT token management             |
| canvas-service| 3002 | Parcel claims, pixel updates, canvas snapshots        |
| chat-service  | 3003 | Global chat, group chat, whisper (DM) messaging       |
| voting-service| 3004 | Vote casting and result tallying                      |
| group-service | 3005 | Group creation, invites, membership (max 10)          |
| scheduler     | 3006 | Activity checks, round lifecycle, voting windows      |

## Game Rules

- **Canvas**: 10,000 × 10,000 pixel grid
- **Parcels**: Each player claims exactly 10,000 contiguous pixels
- **Drawing period**: 1 month per round
- **Activity**: Must draw at least once every 3 days or get disqualified
- **Groups**: Up to 10 players can team up by selecting adjacent parcels
- **Voting**: 3-day window after drawing ends; vote for best individual or group art
- **Parcels are permanent**: Once claimed, selection cannot be changed

## Tech Stack

- **Language**: Rust 2021 edition
- **Web framework**: Axum 0.7 (async, tower-based)
- **Database**: PostgreSQL 16 via sqlx
- **Cache/Canvas store**: Redis 7 (bitmaps for pixel data)
- **Messaging**: NATS with JetStream (event-driven microservices)
- **Auth**: JWT + Argon2 password hashing
- **Observability**: OpenTelemetry → Jaeger (traces), Prometheus + Grafana (metrics)
- **Containerization**: Docker Compose

## Getting Started

### Prerequisites

- Rust toolchain (1.75+)
- Docker & Docker Compose

### 1. Start infrastructure

```bash
docker compose up -d
```

This starts PostgreSQL, Redis, NATS, Jaeger, Prometheus, and Grafana.

### 2. Set up environment

```bash
cp .env.example .env
# Edit .env with your settings
```

### 3. Run database migrations

```bash
sqlx migrate run --source migrations
```

### 4. Run a service

```bash
# Run any individual service
PORT=3001 cargo run -p auth-service
PORT=3002 cargo run -p canvas-service
PORT=3000 cargo run -p api-gateway

# Or run all services (use separate terminals or a process manager)
```

### 5. Access observability

- **Jaeger UI** (traces): http://localhost:16686
- **Prometheus** (metrics): http://localhost:9090
- **Grafana** (dashboards): http://localhost:3100 (admin/admin)
- **NATS monitoring**: http://localhost:8222

## Project Structure

```
PixelWar/
├── Cargo.toml              # Workspace root
├── docker-compose.yml      # Infrastructure stack
├── .env.example            # Environment template
├── config/                 # TOML configs + Prometheus
├── migrations/             # SQL migrations
├── proto/                  # Protobuf definitions (future gRPC)
├── crates/                 # Shared libraries
│   ├── shared-common/      # Models, errors, events, config
│   ├── shared-db/          # PostgreSQL + Redis connections
│   ├── shared-observability/ # Tracing, metrics, health checks
│   └── shared-messaging/   # NATS client + event subjects
└── services/               # Microservices
    ├── api-gateway/        # HTTP/WS entry point
    ├── auth-service/       # Authentication
    ├── canvas-service/     # Canvas & pixel management
    ├── chat-service/       # Real-time messaging
    ├── voting-service/     # Voting system
    ├── group-service/      # Team management
    └── scheduler-service/  # Cron-like periodic jobs
```

## Development Priorities (3-day sprint)

### Day 1 — Core Loop
- [ ] Auth service: register + login with JWT
- [ ] Canvas service: parcel claiming with contiguity validation
- [ ] Canvas service: pixel updates via Redis bitmaps
- [ ] Database migrations and connection pooling

### Day 2 — Social + Real-time
- [ ] WebSocket connections for live pixel updates
- [ ] Chat service: global + whisper messaging via NATS
- [ ] Group service: create, invite, accept
- [ ] Canvas snapshot endpoint

### Day 3 — Game Logic + Polish
- [ ] Scheduler: activity checks (3-day rule)
- [ ] Scheduler: round lifecycle management
- [ ] Voting service: cast + tally
- [ ] Rate limiting middleware
- [ ] Grafana dashboards for monitoring
