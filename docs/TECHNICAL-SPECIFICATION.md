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

## Domains

| URL | Purpose |
|---|---|
| `mavyfaby.com` | Personal portfolio, links to Pulse |
| `pulse.mavyfaby.com` | Public landing page (pulse-web) |
| `pulse.mavyfaby.com/ops` | Emergency operations center — dispatch, alerts, responders (behind auth) |
| `api.pulse.mavyfaby.com` | HTTPS API (pulse-server) |
| `api.pulse.mavyfaby.com/v1/infrastructure/server-ips` | Signed server IP list for mobile clients |

Note: The raw TCP alert path connects directly to server IPs — never to a domain. Domains are only used for the HTTPS API, operations center, and public web.

---



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
- Updated IP lists fetched via HTTPS when data is available from `https://api.pulse.mavyfaby.com/v1/infrastructure/server-ips`
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

## Server IP Strategy

The mobile client connects to the alert server by IP address directly — never by domain. This means the public-facing IP must be stable, secure, and not expose the real underlying server infrastructure.

### Phase 1-3 — AWS Elastic IP

A single Elastic IP attached directly to the EC2 instance. Free within AWS. Simple to set up.

- Real EC2 instance IP is not exposed — Elastic IP is the public face
- If the EC2 instance is replaced, re-attach the Elastic IP to the new instance — no app update needed
- Keep the Elastic IP out of public-facing materials (website, docs, DNS)

Limitation: if the Elastic IP itself is DDoSed, the server is unreachable. Acceptable at small scale where targeted attacks are unlikely.

### Phase 4+ — AWS Global Accelerator

Two static anycast IPs that route to your EC2 instances. Your real instance IPs remain private.

- IPs never change regardless of underlying infrastructure changes
- Built-in health checks and automatic failover between instances
- Routes clients to the nearest AWS edge location (lower latency)
- Hides real EC2 IPs completely
- Reduces reliance on the IP list update mechanism since accelerator IPs are permanent
- Cost: ~$18/month base + data transfer

This is the recommended production setup for Pulse once real users depend on it.

### Alternative: Cloudflare Spectrum

Cloudflare Spectrum proxies raw TCP through Cloudflare's network. Clients connect to Cloudflare's anycast IPs; Cloudflare forwards to your server.

- Real server IP completely hidden
- Built-in DDoS protection
- Paid feature (not on Cloudflare free plan)

**Requires testing:** does Cloudflare Spectrum work on no-subscription SIMs the same way direct IP connections do? Since the client still connects to an IP (Cloudflare's), it should behave identically from the carrier's perspective. Add to Phase 1 carrier testing checklist.

### Alternative: Wireguard VPN tunnel

A cheap public VPS (~$5/month) accepts TCP connections and tunnels them to the real server via Wireguard. The real server has no public IP.

- Completely hides the real server
- If the VPS is DDoSed or compromised, spin up a new one and update the IP list
- Adds one extra network hop (small latency cost)

Viable for teams that want maximum server IP privacy at low cost.

---



### Why multiple static IPs

Multiple server IPs provide high availability — if one server is unreachable (hardware failure, network issue, DDoS), the client automatically tries the next IP in its list. No single point of failure.

### The exploit concern

Static IPs must be known to clients, which means they can be discovered by attackers. Realistic threats:

- **DDoS** — flood the IP with traffic to make it unreachable
- **Alert flooding** — send thousands of fake ALERT messages to exhaust server resources
- **Port scanning** — probe for vulnerabilities on open ports

### Mitigations (Phase 4+)

- **Ed25519 signatures required on every message** — unsigned or malformed messages are rejected immediately. Attackers cannot send valid alerts without a registered device key. This is the strongest application-layer protection.
- **Connection rate limiting per source IP** — firewall rules drop connections from IPs exceeding a threshold (e.g., >100 connections/hour). Legitimate users open ~20 connections/hour.
- **IPs not advertised publicly** — distributed only to registered devices via `GET /v1/infrastructure/server-ips`. Not in DNS, not on the website, not in docs.
- **Cloud provider DDoS protection** — network-layer mitigation from the hosting provider (AWS Shield, GCP Cloud Armor, etc.).
- **Anycast routing** (Phase 6, if needed) — same IP routes to multiple physical servers. DDoS traffic is distributed and absorbed rather than hitting one target.

### Phase 1-3 reality

At small scale, targeted attacks are unlikely. A buggy client that reconnects infinitely is the realistic threat, handled by basic connection rate limiting. Design for serious threats in Phase 4 when real traffic data informs the decisions.

---



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
