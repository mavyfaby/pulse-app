# Pulse — Backend Spec

*Maverick Fabroa ([@mavyfaby](https://github.com/mavyfaby))*

Solo developer, Rust, single VM in Phase 2. Complexity added only when forced.

## Phases

### Phase 1 — Empirical foundation (1-2 weeks)

Deploy minimal Rust TCP echo server with static IP. Build mobile test harness. Verify on Smart, Globe, DITO across SIM states. Record compatibility matrix.

Deliverable: signed-off carrier matrix.

### Phase 2 — Minimum viable Pulse (4-6 weeks)

One Rust binary. TCP listener (port 7000) + HTTPS API (port 443). PostgreSQL + Redis. Single VM.

Scope:
- ALERT, ACK, ERROR, RESOLVE, RESPONDER_HEARTBEAT messages
- Ed25519 signature verification
- Device registration
- Optional profile (name only)
- Responder mode toggle, default 1km radius
- Naive responder matching (linear scan)
- WebSocket push to responders

Acceptance:
- End-to-end alert works on no-subscription Smart SIM
- ACK within 1s on healthy network
- Service auto-restarts within 5s of crash
- 24-hour stability test passes

### Phase 3 — Voice and live updates (3-4 weeks)

- VOICE_CHUNK, LOCATION_UPDATE messages
- Opus codec, multiple bitrates
- S3-compatible voice storage
- Live location during active alert

### Phase 4 — Scale (triggered by real growth)

- Split TCP and HTTPS into separate binaries
- Load balancer + multiple instances
- Redis GEO index for responder matching
- Multiple static IPs with failover

### Phase 5 — Optional account features (demand-driven)

Only if users ask for it:
- Phone verification
- ID verification
- Medical info / emergency contacts
- Organizational accounts
- Command center dashboards
- Emergency type selection

Don't build speculatively.

### Phase 6 — Advanced

- Mesh / P2P transmission
- TLS over TCP
- Server-side voice transcription
- Multi-region

## Architecture (Phase 2)

```
Mobile Client
   ↓ (TCP) ↓ (HTTPS/WSS)
Pulse Backend (single Rust binary)
   ↓        ↓
PostgreSQL  Redis
```

One crate, modules organize code. No workspace until Phase 4.

```
pulse-backend/
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── state.rs
│   ├── protocol/   (codec, messages, signature)
│   ├── tcp/        (server, handler)
│   ├── http/       (server, routes)
│   ├── domain/     (device, alert, location)
│   ├── db/         (sqlx queries)
│   ├── cache/      (redis access)
│   ├── routing/    (matcher)
│   └── error.rs
├── migrations/
└── tests/
```

## Main entry (illustrative)

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::load()?;
    let db = db::connect(&config.database).await?;
    let redis = cache::connect(&config.redis).await?;
    sqlx::migrate!("./migrations").run(&db).await?;

    let state = AppState::new(db, redis, config.clone());
    tokio::try_join!(
        tcp::run(state.clone(), config.tcp_addr),
        http::run(state.clone(), config.http_addr),
    )?;
    Ok(())
}
```

## Database Migration Strategy

### Phase 2 — Automatic (run in `main.rs`)

Migrations run automatically on server startup via `sqlx::migrate!`. The server will not start if a migration fails — the error appears immediately in logs.

Justification: solo developer, single VM, fast iteration. sqlx acquires a lock before running migrations so concurrent starts (e.g., systemd restart during a running instance) are safe.

### Phase 4+ — Manual (decouple from startup)

Once multiple server instances run behind a load balancer, migrations must be decoupled from binary startup. The pattern becomes:

1. Run migration as a separate step in the deploy pipeline
2. Verify database state
3. Roll out new binary

To switch: remove `sqlx::migrate!` from `main.rs` and add to deploy script:

```bash
sqlx migrate run --database-url $DATABASE_URL
```

### Rules for all phases

- Migrations are always **additive** — add columns, add tables, add indexes
- Never drop columns or tables in the same release that removes code using them — do it in a follow-up release
- Never rename columns — add a new column, migrate data, drop the old one in a later release
- Every migration is tested on a copy of production data before deploying to production

## HTTPS API (Phase 2)

```
POST   /v1/devices/register         register Ed25519 public key
GET    /v1/devices/profile          fetch profile
PATCH  /v1/devices/profile          update profile (optional fields)
PUT    /v1/devices/responder-mode   toggle responder availability + radius
GET    /v1/infrastructure/server-ips signed JSON IP list
GET    /v1/infrastructure/status    public status
WSS    /v1/responder/stream         responder dispatch push
```

Auth: each request signed with device's Ed25519 key. `X-Pulse-Signature` and `X-Pulse-Device-ID` headers. No JWT.

Rate limits (per device):
- Alerts: 20/hr, 100/day
- Profile updates: 60/hr
- Status: 600/hr

## Schema (Phase 2)

> ⚠️ **Draft.** This schema is a starting point, not a final design. Fields will be added, removed, or modified as development and carrier testing reveal real requirements. Do not treat this as immutable until Phase 3 begins.

```sql
CREATE TABLE devices (
    id                  UUID PRIMARY KEY,
    public_key          BYTEA NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at        TIMESTAMPTZ,
    profile             JSONB,
    responder_mode      BOOLEAN NOT NULL DEFAULT FALSE,
    responder_radius_m  INTEGER NOT NULL DEFAULT 1000,
    responder_skills    TEXT[] NOT NULL DEFAULT '{}'
);

CREATE TABLE alerts (
    id                  UUID PRIMARY KEY,
    device_id           UUID NOT NULL REFERENCES devices(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status              TEXT NOT NULL DEFAULT 'active',
    resolved_at         TIMESTAMPTZ,
    location_lat        DOUBLE PRECISION NOT NULL,
    location_lng        DOUBLE PRECISION NOT NULL,
    location_accuracy_m DOUBLE PRECISION,
    device_info         JSONB
);

CREATE INDEX idx_alerts_active ON alerts(status) WHERE status = 'active';
CREATE INDEX idx_alerts_device_id ON alerts(device_id);

CREATE TABLE alert_dispatches (
    id                  UUID PRIMARY KEY,
    alert_id            UUID NOT NULL REFERENCES alerts(id),
    responder_device_id UUID NOT NULL REFERENCES devices(id),
    dispatched_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status              TEXT NOT NULL DEFAULT 'notified',
    status_updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE audit_log (
    id          BIGSERIAL PRIMARY KEY,
    actor_id    UUID,
    action      TEXT NOT NULL,
    target_id   UUID,
    metadata    JSONB,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

4 tables. No accounts, no organizations, no medical info, no contacts.

## Redis keys

```
nonce:<base64>                     TTL 10min
responder:<device-id>              TTL 30min (lat, lng, radius, skills, last_update)
alert:<alert-id>                   TTL 24h
ratelimit:<bucket>:<id>            INCR/EXPIRE

Pub/sub:
alerts:new
responders:<device-id>:dispatch
```

## Responder matching (Phase 2)

```
1. Get all devices with responder_mode = true (cached in Redis)
2. For each:
   - Skip if location stale (>30 min)
   - Compute Haversine distance to alert
   - Skip if distance > radius * 1.1
3. Sort by distance, take top 20
4. INSERT dispatches, publish to Redis, WebSocket push
```

Linear scan is fine up to thousands of responders. Geo index added in Phase 4.

## Background Probe and Heartbeat Handling

The server handles two types of heartbeat messages:

**HEARTBEAT (client → server)** — sent by any device (alerter or responder) as a background probe to confirm the TCP path is alive. The server responds with ACK immediately. No state is written to the database. Redis `devices:<id>` last_seen timestamp is updated.

**RESPONDER_HEARTBEAT (client → server)** — sent by responder-mode devices. Carries current location and availability. Server updates Redis responder state and GEO index. Also serves as a connection probe for responders.

### Server-side behavior

```
HEARTBEAT received:
  1. Verify signature
  2. Check nonce (replay protection)
  3. Update devices last_seen_at in Redis (TTL 30 min)
  4. Send ACK
  5. Close (client will close after ACK)

RESPONDER_HEARTBEAT received:
  1. Verify signature
  2. Check nonce
  3. Update responder state in Redis (lat, lng, available, last_update)
  4. Send ACK
  5. Close
```

### Rate limiting

HEARTBEAT messages are rate-limited separately from alerts:

- Max 10 heartbeats per hour per device (one every ~6 minutes minimum)
- Prevents probe flooding from misbehaving clients
- Stricter than the probe interval (10-15 min) to allow retries without hitting limits

---



### Config
Env vars via `figment`, typed structs, validated at startup. Refuse to start on invalid config.

### Logging
Structured JSON via `tracing`. Service, request/alert ID, device ID in every line. Sensitive fields never logged.

### Metrics (Prometheus)
```
pulse_alerts_ingested_total
pulse_alert_ingest_latency_seconds
pulse_signature_failures_total{reason}
pulse_routing_latency_seconds
pulse_responders_matched_per_alert
pulse_db_query_latency_seconds
pulse_active_websocket_connections
```

### Shutdown
SIGTERM → drain (30s timeout) → exit. systemd manages.

### Deployment
Single VM, systemd-supervised, restart on crash. Managed PostgreSQL + Redis.

### Backup
- PostgreSQL: WAL archiving + daily snapshots, 30-day retention
- Redis: AOF for nonce cache, rest is ephemeral
- IP list: versioned in object storage

## Security

- TLS 1.3 on HTTPS API
- DB and Redis connections over TLS
- Server signing key in cloud KMS
- All messages signature-verified before processing
- Nonce cache for replay protection
- No TLS on raw TCP (justified by handshake overhead + e2e signatures)

### Abuse prevention
- Per-device alert rate limits (20/hr, 100/day)
- Geographic anomaly detection
- False-alarm tracking via responder feedback

## Privacy

- Phase 2 collects: device key, optional name, location during alerts
- Retention: 30 days for alerts, anonymize after
- No third-party analytics, no advertising IDs
- Export and delete on user request, completed within 30 days

Full privacy spec planned separately.

## Open questions

- Cloud provider (AWS / GCP / Philippine local)
- Managed vs self-hosted PostgreSQL
- SMS provider (only matters Phase 5+)
- Object storage choice (Phase 3)
- APNs / FCM for push
- Geographic data residency under RA 10173
- Root cause of 3-send Smart limit
- Globe and DITO behavior

## Stack

### Platform

| Component | Technology |
|---|---|
| Backend | Rust (stable, edition 2021) |
| Mobile client | Kotlin (Android, native) |
| Async runtime | Tokio |
| Database | PostgreSQL 15+ |
| Cache / pub-sub | Redis 7+ |
| Voice storage | S3-compatible object storage (Phase 3) |

### Rust crates

| Crate | Purpose |
|---|---|
| `tokio` | Async runtime |
| `tokio-util` | Codec for binary protocol |
| `axum` | HTTPS API |
| `tower-http` | Middleware |
| `sqlx` | PostgreSQL |
| `fred` | Redis |
| `serde` + `ciborium` | CBOR serialization |
| `ed25519-dalek` | Signatures |
| `rustls` + `tokio-rustls` | TLS |
| `tracing` | Logging |
| `thiserror` + `anyhow` | Errors |
| `figment` | Config |

### Kotlin (Android) key APIs

| API | Purpose |
|---|---|
| `java.net.Socket` | Raw TCP socket (not OkHttp — HTTP-only) |
| Android Keystore System | Ed25519 keypair generation and secure storage |
| `ForegroundService` | Background execution, prevents OS process killing |
| `FusedLocationProviderClient` | Location with battery-efficient background updates |
| `BluetoothLeScanner` | BLE for future mesh networking (Phase 6) |
