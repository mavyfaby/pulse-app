# Pulse — Bayanihan Emergency Network

![Pulse — Bayanihan Emergency Network](images/cover.png)

> An open-source community emergency response network that works even without a mobile data subscription.

Built for the Philippines, designed for anywhere with connectivity gaps.

**Author:** Maverick Fabroa ([@mavyfaby](https://github.com/mavyfaby))

---

## Links

| | |
|---|---|
| 🌐 Website | [pulse.mavyfaby.com](https://pulse.mavyfaby.com) |
| 🖥️ Operations Center | [pulse.mavyfaby.com/ops](https://pulse.mavyfaby.com/ops) |
| 👤 Author | [mavyfaby.com](https://mavyfaby.com) |

---



Pulse is a mobile emergency response app that lets you summon nearby help with a single button hold. When you activate it, your location is automatically transmitted to nearby volunteer responders — even if your prepaid SIM has no remaining balance, no active data subscription, or degraded signal.

**Pulse fills the gaps that 911 leaves.**

It doesn't replace official emergency services. It exists for the specific situations where calling 911 alone isn't enough:

- Your prepaid load ran out
- You can't safely make a voice call
- A neighbor 200 meters away could arrive faster than official dispatch
- You're injured and can't speak
- You're a bystander reporting someone else's emergency

---

## How it works

1. Hold the SOS button for 2.5 seconds
2. A 10-second countdown begins (tap cancel anytime)
3. Alert fires automatically with your location and ambient audio
4. Nearby responders are notified instantly
5. Tap "I'm safe now" when the situation is resolved

No typing. No forms. No decisions during the emergency.

---

## The killer feature

Most emergency apps stop working the moment your mobile data subscription runs out. Pulse transmits alerts via raw TCP directly to server IP addresses — bypassing the DNS gating that carriers enforce on no-subscription SIMs. Small alert payloads get through even when normal data doesn't.

This behavior has been observed and tested on Smart Communications (Philippines) SIMs with zero remaining balance. Pre-deployment testing across Globe and DITO is in progress.

---

## Status

🔬 **Currently in research and design phase.**

- [x] Problem research and value proposition
- [x] UX specification
- [x] Technical specification
- [x] Backend specification (Rust / Tokio)
- [x] Research paper
- [ ] Phase 1 — Carrier compatibility testing (Smart, Globe, DITO)
- [ ] Phase 2 — Minimum viable alert (single binary, end-to-end flow)
- [ ] Phase 3 — Voice transmission
- [ ] Phase 4 — Scale and resilience
- [ ] Phase 5 — LGU partnerships and command center integration
- [ ] Phase 6 — Mesh / peer-to-peer transmission

---

## Documentation

| Document | Description |
|---|---|
| [Research Paper](docs/RESEARCH.md) | Problem statement, solution overview, technical approach, system flow |
| [UX Specification](docs/UX-SPECIFICATION.md) | User interaction design, screens, flows |
| [Technical Specification](docs/TECHNICAL-SPECIFICATION.md) | Protocol design, transmission constraints, carrier behavior |
| [Backend Specification](docs/BACKEND-SPECIFICATION.md) | Rust server architecture, data model, API, deployment phases |

---

## Tech stack

- **Backend:** Rust (Tokio, Axum, sqlx)
- **Mobile:** Kotlin (Android, native)
- **Protocol:** Custom binary over raw TCP (CBOR payload, Ed25519 signatures)
- **Storage:** PostgreSQL + Redis

---

## Why open source?

Pulse is built as civic infrastructure, not a commercial product. Emergency response tools should be transparent, auditable, and community-maintained. Anyone can inspect the code, contribute improvements, adapt it for their community, or deploy their own instance.

The goal is for Pulse to become part of how Filipino communities respond to emergencies — not to be a product that one person owns.

---

## Testing

The backend test suite is pure — no environment variable mutation, no serial execution required, so tests run in parallel at full speed.

```bash
just server-test
```

---

## Contributing

Pulse is in early stages and welcomes contributions in all forms:

- **Developers** — protocol implementation, mobile client, backend services
- **Designers** — UX refinement, accessibility, visual design
- **Researchers** — carrier behavior testing, emergency response systems research
- **Community organizers** — LGU partnerships, responder network building, pilot coordination
- **Translators** — Filipino, Cebuano, and other Philippine languages

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to get involved.

If you're a Philippine developer, civic tech organization, LGU, or emergency response organization interested in piloting or partnering, reach out via GitHub Issues or Discussions.

---

## License

AGPL v3 — see [LICENSE](LICENSE) for details.

You are free to use, modify, and deploy this software. If you distribute a modified version or run a modified version as a network service, you must publish the source code of your modifications under the same license. This ensures Pulse and all derivatives remain open infrastructure for the community, forever.

---

## Acknowledgments

Inspired by international peer-response systems including PulsePoint (US), GoodSAM (UK), Hatzalah, and myResponder (Singapore), and by the genuine need for better emergency response infrastructure in the Philippines.

---

## Sponsor

Pulse is free, open-source, and built for community benefit. If you'd like to support the development and infrastructure costs, consider sponsoring:

[![Sponsor mavyfaby](https://img.shields.io/badge/Sponsor-mavyfaby-ea4aaa?logo=github-sponsors&logoColor=white)](https://github.com/sponsors/mavyfaby)

Every contribution helps keep Pulse running and improving.

---

*"Pulse fills the gaps that 911 leaves."*
