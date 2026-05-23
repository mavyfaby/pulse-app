# Pulse — Technical Spec

*Maverick Fabroa ([@mavyfaby](https://github.com/mavyfaby))*

Transmission and architecture constraints.

## Stack

- **Backend:** Rust (Tokio)
- **Mobile:** Kotlin (Android, native)
- **Storage:** PostgreSQL + Redis
- **Voice:** S3-compatible object storage (Phase 3)

### Mobile implementation notes

Raw TCP socket via `java.net.Socket` directly — not OkHttp or Retrofit, which are HTTP-only. Full socket control is required for the connect-send-close cycle and the 3-send-per-connection workaround.

Background execution via Android `ForegroundService` with persistent notification to prevent OS process killing.

Ed25519 keypair generation and storage via Android Keystore System. Keys never leave secure hardware on supported devices.

Location via `FusedLocationProviderClient`. Significant location change API for background updates without continuous GPS drain.

## Wire protocol

Custom binary over raw TCP. HTTP avoided for alert path (DNS gating, handshake overhead).

Frame:
```
[1 byte]   Protocol version (0x01)
[1 byte]   Message type
[4 bytes]  Payload length (u32 BE)
[N bytes]  Payload (CBOR, canonical encoding)
[64 bytes] Ed25519 signature over preceding bytes
```

Signatures cover the raw bytes. Canonical CBOR required for deterministic signing.

## Message types

| Byte | Name | Direction | Phase |
|---|---|---|---|
| 0x01 | ALERT | client → server | 2 |
| 0x02 | ACK | server → client | 2 |
| 0x03 | ERROR | server → client | 2 |
| 0x04 | RESOLVE | client → server | 2 |
| 0x05 | VOICE_CHUNK | client → server | 3 |
| 0x06 | LOCATION_UPDATE | client → server | 3 |
| 0x07 | RESPONDER_HEARTBEAT | client → server | 2 |
| 0x08 | DISPATCH | server → client (WebSocket) | 2 |

## Identity

Each device generates an Ed25519 keypair on first install. Public key registered with the server. Every message signed with the device's private key. No accounts, no JWT, no passwords in Phase 2.

## Replay protection

16-byte nonce per message. Server caches nonces 10 minutes. Duplicates rejected.

## Critical constraint: DNS gating

**Observed on Smart SIM, no subscription:** raw TCP to IP works; DNS resolution fails.

Consequences:
- Server addressing by IP, not domain, for alert path
- App ships with hardcoded server IP list
- Updated IP lists fetched via HTTPS when data is available
- All published IPs must be static
- Minimum 3 IPs at launch, 90-day deprecation window

## Critical constraint: TCP send limit

**Observed on Smart SIM, no subscription:** single TCP connection transmits 3 times max. 4th send fails silently.

Workaround: connect → send → ACK → close per transmission. No long-lived connections on the constrained path.

Performance target: full cycle <1s on healthy network.

Cause unknown. Must validate on Globe and DITO carriers.

## Pre-deployment testing

Compatibility matrix required before launch. Per carrier (Smart, Globe, DITO) × per SIM state (full balance, zero balance, expired, suspended, data disabled):

- TCP to IP works?
- DNS resolution works?
- Max sends per connection
- Max reliable payload size
- Latency (median, p95)
- ACK returns?

Minimum 9 SIMs (3 carriers × 3 states), iOS + Android.

## Voice transmission (Phase 3)

Opus codec, tiered:
- Normal data: 16-24 kbps
- Constrained path: 6-8 kbps
- Survival mode (Phase 6): Codec2 1.6 kbps

Chunked into ~1000-byte VOICE_CHUNK messages. Server reassembles. Partial recordings still usable (Opus tolerates gaps).

Two paths:
- TCP chunked (constrained)
- HTTPS upload (when data available, full quality)

Local copy is source of truth. Transmission is best-effort.

## Server architecture

- **Phase 1-2:** single Rust binary, TCP + HTTPS as concurrent Tokio tasks
- **Phase 3+:** split into separate binaries when failure isolation matters

## Security (Phase 2)

- Ed25519 signatures on every message
- Replay protection via nonce cache
- Server ACKs signed with server key (client verifies)
- No TLS on raw TCP path (handshake overhead, signature-based integrity sufficient)
- HTTPS API uses TLS 1.3

Phase 6 may add TLS over TCP after empirical testing.

## Privacy by design

- Data minimization throughout
- Short retention (30 days for alerts, 90 days for voice)
- No third-party analytics
- No advertising IDs
- User can export and delete all data

Full privacy spec is a planned separate document.

## Mesh / P2P (Phase 6)

Bluetooth LE / WiFi Direct relay through nearby Pulse users. Deferred until user density justifies it. Architecture decisions that affect this even now:
- Self-contained UUIDs as identifiers
- End-to-end signatures from day one
- Small core payload (~200 bytes) for relay-friendliness

## Background Connection Probe

The mobile client periodically probes the server even when the app is not in use. This confirms the no-subscription TCP path is working before an emergency occurs — not just that the device has internet.

### Probe behavior

- Open TCP connection to first available server IP
- Send HEARTBEAT message (~50 bytes)
- Wait for ACK (3 second timeout)
- Close connection
- Update connection status indicator on home screen

### Probe interval

- Every 10-15 minutes during normal background operation
- Immediately on network change (WiFi → cellular, cellular → WiFi)
- Immediately when app is foregrounded
- Once on first app launch

Conservative interval reduces risk of exhausting carrier "free TCP" budget through repeated background probes before a real emergency.

### Platform implementation

**Android:** `WorkManager` periodic task. Battery-efficient, survives app close.

**iOS:** `BGAppRefreshTask`. iOS controls the exact schedule — less reliable than Android for background probing.

### Rate limiting concern

Background probes on a no-subscription SIM use the same "free TCP" budget as real alerts. Must be tested: run probes at 10-minute intervals for 24 hours on a zero-balance Smart SIM and confirm alerts still transmit reliably afterward.

---

## Open questions

- DNS-over-TCP fallback viability
- Voice transmission sustainability on constrained path
- Globe and DITO behavior (unverified)
- Carrier partnership for official zero-rating
- RA 10173 regulatory positioning
