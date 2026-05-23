# Pulse — Bayanihan Emergency Network

*A research and design study for the Philippine context*

**Author:** Maverick Fabroa ([@mavyfaby](https://github.com/mavyfaby))
**Date:** May 2026

---

## Abstract

Pulse is an emergency response system designed to function in scenarios where conventional emergency communication fails. The system enables users to summon nearby peer responders and (in later phases) official emergency services through a single-button activation, with the distinguishing capability of transmitting alerts over cellular networks even when the user has no active mobile data subscription. This document describes the problem space, the technical and design approach, the system flow, and the path toward open-source civic deployment.

---

## 1. Problem Statement

### 1.1 The emergency response gap

Emergency response in the Philippines, as in many developing contexts, has several structural gaps that the existing emergency call system (911) does not fully address:

**Connectivity gap.** A significant portion of mobile users operate on prepaid SIMs with intermittent or depleted balances. When data is unavailable, most modern safety and emergency tools cease to function. Voice calls require functional cellular voice service, which may be degraded in low-balance states. SMS may be limited or unavailable. The user in distress has no reliable digital channel.

**Silence gap.** Some emergency situations — domestic violence, intruder presence, stalking, hostage-like scenarios — make voice calls dangerous because they alert the perpetrator. Existing solutions require speech or visible interaction with a phone, neither of which is safe in such contexts.

**Speed gap.** Official emergency response (police, ambulance, fire) frequently takes 15-30 minutes or longer to arrive in Philippine urban contexts, and substantially longer in rural areas. For time-critical incidents such as cardiac arrest, severe bleeding, or assault, this delay is often the difference between life and death. A nearby trained civilian could often arrive within minutes.

**Incapacitation gap.** Users who are injured, unconscious, or in severe distress cannot communicate effectively with emergency operators. They cannot describe their location, condition, or what happened. The traditional call-based emergency model assumes a coherent, communicative caller.

**Witness gap.** Bystanders who observe emergencies happening to strangers often lack a quick, reliable channel to summon help. The witness may not know the victim's identity, may be uncertain about official procedures, or may be in a context where extended phone calls are impractical.

**Community gap.** Certain populations — marginalized communities, undocumented persons, victims of police misconduct — may legitimately mistrust official emergency services. They lack a non-official channel for help during emergencies.

### 1.2 Limitations of existing approaches

Existing emergency tools fall short in addressing these gaps:

The Philippine national emergency hotline (911) provides authoritative dispatch but suffers from inconsistent response times, geographic coverage limitations, and dependence on functional voice service.

Mainstream safety apps (e.g., Citizen, bSafe) require active mobile data and provide either passive incident reporting or simple alarms without coordinated peer response. None operate effectively in no-subscription scenarios.

SMS-based emergency systems are limited by carrier delivery delays, character limits, and the difficulty of conveying location precisely in text.

Existing peer-response models internationally (PulsePoint in the United States, GoodSAM in the United Kingdom, Hatzalah in multiple countries) demonstrate that peer-response architectures can save lives, but these systems all require functional mobile data and have not been adapted for connectivity-constrained markets.

### 1.3 The opportunity

A system that combines peer-response coordination with subscription-independent transmission would fill a specific and demonstrable gap in the Philippine emergency response landscape. Such a system would not replace 911, but would complement it by extending help to situations where official services are insufficient, inaccessible, or unsafe to engage.

---

## 2. Proposed Solution

### 2.1 System overview

Pulse is a mobile application paired with a server-side coordination platform. The user interaction consists of holding a single button to activate an emergency alert. The system then automatically transmits the alert to nearby registered responders (volunteer civilians who have opted in to receive emergency notifications) and, in later phases, to official emergency services.

The system's distinguishing characteristic is its transmission protocol: alerts are sent via raw TCP to known server IP addresses, which functions on cellular networks even when the user has no active mobile data subscription, no remaining prepaid balance, or no functional DNS resolution.

### 2.2 Value proposition

Pulse fills gaps that 911 leaves. It is not a replacement for official emergency services. Specifically, Pulse provides value in the following scenarios:

- The user has no mobile data subscription, depleted balance, or suspended account
- The user cannot safely make a voice call
- A nearby civilian could arrive faster than official dispatch
- The user is incapacitated and cannot communicate
- A bystander is reporting an emergency involving a third party
- Cellular voice or SMS is degraded due to network congestion, weak signal, or carrier-level issues
- The user has reasons not to engage official services as the primary response

For emergencies where 911 is reachable, appropriate, and sufficient, Pulse honestly directs the user to 911 first.

### 2.3 Core design principles

**Minimal interaction.** The user holds one button. Help is summoned. Everything else is automatic or optional. The application avoids all friction that would impede an injured, panicked, or incapacitated user.

**Honest fallback.** When transmission fails, the application clearly informs the user and directs them to alternative channels (typically calling 911). It never falsely indicates successful transmission.

**No required identification.** The application functions without requiring the user to provide a name, phone number, medical information, or any personal data. A device-level cryptographic identity is sufficient.

**Privacy by design.** Data minimization, short retention, no third-party analytics, no advertising identifiers. The system collects only what is necessary for emergency coordination.

**Open infrastructure.** The protocol, server code, and design are open-source. The system is built to be civic infrastructure that any community can adopt and contribute to.

---

## 3. Technical Approach

### 3.1 Transmission protocol

Pulse uses a custom binary protocol over raw TCP. The protocol is designed for:

- Small payload sizes that fit within single TCP packets
- Cryptographic message integrity (Ed25519 signatures)
- Replay protection (nonce-based)
- Functioning on subscription-depleted cellular connections
- Connect-send-close transmission patterns (working within observed carrier constraints)

HTTP is not used for the critical alert path because it requires DNS resolution, which is gated by carrier infrastructure on no-subscription SIMs. Domain names cannot be relied upon; the application addresses servers by IP address directly.

### 3.2 Empirical foundation

The transmission approach rests on two empirical findings observed in initial testing on Smart Communications (Philippines) SIMs:

1. Raw TCP connections to a server IP address establish successfully and deliver small payloads even when the SIM has no active subscription and no remaining balance. Domain name resolution under the same conditions fails.

2. A single TCP connection reliably transmits up to three payloads in the no-subscription state. Subsequent sends on the same connection fail silently. The workaround is to open a new TCP connection for each transmission.

These findings have not yet been validated across other Philippine carriers (Globe Telecom, DITO Telecommunity) or across the full range of SIM states. A pre-deployment validation phase will produce a compatibility matrix establishing the precise behavior of each carrier and SIM state combination.

### 3.3 System architecture

The server-side implementation is written in Rust using the Tokio asynchronous runtime. In initial phases, the server runs as a single binary providing both the raw TCP listener (for alerts) and an HTTPS API (for non-critical operations). The architecture is designed to grow into a distributed multi-service deployment as scale requires.

Storage is provided by PostgreSQL for durable state and Redis for ephemeral state and pub/sub messaging. Voice recordings are stored in S3-compatible object storage.

Static IP addresses are required for the alert servers, with multiple addresses provisioned for redundancy. Updated IP lists are distributed to clients via the HTTPS API when devices have working connectivity.

### 3.4 Identity and authentication

Each device generates an Ed25519 keypair on first launch, stored in the device's secure keystore. The public key is registered with the server. All messages from the device are signed with the device's private key, providing message authenticity without requiring user accounts, phone numbers, or other identifying information.

This design choice has significant privacy benefits. The application can be used fully anonymously. Users who have reasons to remain unidentified (domestic violence survivors, vulnerable populations, those distrustful of official systems) can use Pulse without leaving an identity trail.

### 3.5 Responder coordination

Users who opt in to be responders share their location and availability with the server. When an alert is received, the server identifies nearby available responders and pushes notifications to them via WebSocket connections. Responders accept or decline alerts and, if accepting, navigate to the alerter's location.

The matching algorithm uses geographic radius (default 1 kilometer for individual responders, configurable) with stale-location filtering. In larger deployments, geographic indexing (Redis GEO) provides scalable matching.

---

## 4. System Flow

### 4.1 Alerter activation flow

The complete flow from emergency activation to resolution:

1. User holds the SOS button on the application home screen.
2. A progress ring fills around the button over 2.5 seconds, accompanied by haptic feedback.
3. Upon completion, the screen transitions to a 10-second countdown with a prominent cancel button.
4. The user may cancel the countdown at any point with a single tap, returning the application to its idle state.
5. If not cancelled, the countdown completes. The application begins transmitting the alert.
6. Location, device information, and a cryptographically signed alert message are constructed and sent via raw TCP to the first available server IP address.
7. If the transmission succeeds (acknowledgment received within 3 seconds), the application transitions to an active alert state, displaying "Help is coming" to the user.
8. If the initial transmission fails, the application tries the next server IP. If all hardcoded IPs fail, the application continues retrying in the background while honestly informing the user that the alert has not yet been delivered, and providing a prominent option to call 911 directly.
9. The application begins recording ambient audio locally and transmits it to the server in small chunks as connectivity allows.
10. The server identifies nearby available responders and dispatches the alert to them.
11. Responders receive notifications, view the location and context, and choose to accept or decline.
12. As each responder accepts, the alerter's screen updates with their name and estimated arrival.
13. The alerter waits. No further interaction is required.
14. When the situation is resolved, the alerter taps "I'm safe now." A resolve message is transmitted to the server, ambient recording stops, and the application returns to its idle state.

### 4.2 Responder reception flow

1. A user with responder mode enabled has periodic location heartbeats sending to the server.
2. When an alert arrives within the responder's configured radius, the server pushes a notification via WebSocket.
3. The notification appears on the responder's device, overriding silent mode if necessary.
4. The responder taps the notification, viewing the alert location, distance, and contextual information.
5. The responder taps "I'm coming" to accept, or "Can't help" to decline.
6. If accepted, the application provides navigation guidance to the alert location. Ambient audio from the alerter plays automatically if available.
7. Upon arrival, the responder taps "I'm on scene." This information is relayed to the alerter.
8. The responder assesses the situation in person and takes appropriate action (including calling official services if needed).
9. When the situation is resolved, the responder taps "Situation resolved," completing the alert lifecycle.

### 4.3 Degraded transmission scenarios

The system's value depends on graceful behavior when transmission is degraded.

**Depleted SIM (Pulse's signature case).** The user has cellular signal but no working mobile data. The application's raw TCP transmission succeeds where conventional applications would fail. The user experience is indistinguishable from having data available.

**Weak signal.** Repeated transmission attempts may be needed. The application retries automatically with each new TCP connection.

**Network congestion.** Small TCP packets often succeed when voice or larger data transfers fail. The application benefits from the relatively low bandwidth requirement.

**Complete loss of signal.** Outside Pulse's current capability. The application honestly informs the user and surfaces alternatives. Future work on mesh networking (Phase 6) may address this.

---

## 5. Privacy and Ethical Considerations

### 5.1 Data minimization

Pulse collects only what is necessary for emergency coordination: device cryptographic identity, location during active alerts, and optionally ambient audio recordings during active alerts. The application does not collect names, phone numbers, medical information, or any other personal identifying information by default. Optional profile information may be added in later phases.

### 5.2 Retention

Alerts retain full data for 30 days for operational and audit purposes, after which they are anonymized. Voice recordings are retained for 90 days unless flagged as evidence in an ongoing case. Users can request export and deletion of their data at any time, with deletion completed within 30 days.

### 5.3 Third-party data

When a witness reports an emergency involving a third party who has not consented to be in the system, special care is required. The system minimizes data retention about the patient, does not associate them with the witness's account, and provides mechanisms for patients to request deletion of records if they later become aware.

### 5.4 Regulatory compliance

The system is designed to comply with the Philippine Data Privacy Act (Republic Act 10173). A Data Protection Officer will be designated, a Privacy Impact Assessment conducted, and the system registered with the National Privacy Commission as appropriate. Compliance with the Anti-Wiretapping Act (Republic Act 4200) requires user consent for ambient audio recording, which is obtained during onboarding and toggleable in settings.

### 5.5 Abuse prevention

Emergency systems are attractive targets for abuse: false reports, swatting-style attacks, harassment of specific addresses. Mitigations include per-device rate limiting, geographic anomaly detection, reputation tracking based on responder feedback, and clear terms of service with consequences for malicious use. Anonymous (Tier 0 in later phases) alerts route preferentially to official channels rather than peer responders, reducing the harm potential of unverified reports.

---

## 6. Development Approach

### 6.1 Phased development

Phase 1 (1-2 weeks): Empirical validation of carrier transmission behavior across Smart, Globe, and DITO SIMs in multiple subscription states. Production of a carrier compatibility matrix.

Phase 2 (4-6 weeks): Minimum viable system. Single Rust binary providing TCP alert ingest and HTTPS API. Basic responder matching by location. Pilot-ready for a single community.

Phase 3 (3-4 weeks): Voice transmission with chunked Opus encoding. Live location updates during active alerts.

Phase 4 (6-8 weeks): Service split for failure isolation. Geographic indexing for scalable responder matching. Multiple server IPs with client-side failover.

Phase 5 (demand-driven): Optional account features, organizational accounts, command center dashboards, official emergency service integration. Built only as real partnerships emerge.

Phase 6 (ongoing): Advanced capabilities including peer-to-peer mesh networking, server-side voice transcription, multi-region deployment.

### 6.2 Pilot deployment strategy

Initial deployment is intentionally narrow. A single Local Government Unit (LGU) partnership in a specific barangay, with a pre-recruited pool of trained responders (potentially drawn from barangay tanods, community health workers, or trained civilian volunteers), provides the controlled environment needed to validate the system's real-world effectiveness before broader deployment.

Pilot success criteria include: alert delivery success rate exceeding 95% in the target area; responder acceptance rate sufficient to meaningfully reduce response time compared to official dispatch; false alarm rate below 5%; user adoption among the target population.

### 6.3 Open-source governance

The project is open-source from inception under a permissive license (likely Apache 2.0). Code, documentation, and design artifacts are public. Contribution is welcomed from the broader Philippine developer community and civic tech organizations. Governance is currently maintainer-led with a path to community governance as the contributor base grows.

---

## 7. Related Work and Precedent

International precedents support the viability of peer-response emergency systems:

**PulsePoint** (United States) demonstrates that crowdsourced civilian response to cardiac arrests, where trained bystanders provide CPR before paramedics arrive, measurably improves survival rates. The system operates in hundreds of communities and has documented life-saving outcomes.

**GoodSAM** (United Kingdom) integrates with the NHS to dispatch trained responders to medical emergencies. The system is officially endorsed and integrated into formal dispatch.

**Hatzalah** (multiple countries) operates community-based volunteer emergency medical response, typically achieving response times below those of official ambulance services in their service areas.

**myResponder** (Singapore) combines peer response with official civil defense integration, providing a model for Pulse's potential evolution toward integrated official-and-civilian response.

These systems collectively demonstrate that peer response can save lives, that civilian responders can be coordinated through mobile applications, and that integration with official services is achievable through deliberate partnership work. None of these systems address the no-subscription transmission challenge that Pulse specifically tackles.

---

## 8. Open Questions and Future Work

Several questions remain open and are subjects of ongoing investigation:

- Whether the carrier transmission behaviors observed on Smart Communications apply equivalently to Globe Telecom and DITO Telecommunity. Field testing will produce this evidence.

- The precise root cause of the three-send transmission limit observed on Smart. Identification of the cause may permit higher transmission rates.

- Whether sustained voice transmission is feasible on subscription-depleted connections. Initial testing has not yet confirmed practical voice quality at sustained bandwidth.

- Optimal pilot community selection. Factors include responder availability, LGU partnership readiness, population density, and existing emergency response infrastructure gaps.

- Long-term sustainability of the operating costs. Server infrastructure, voice storage, and SMS gateway costs (in later phases) require either institutional partnership funding or sustainable cost coverage mechanisms.

- Integration pathways with official Philippine emergency services (911, NDRRMC, local DRRMOs, PNP, BFP). Achieving official integration is a multi-year relationship-building effort.

- Applicability of the system to mass-disaster scenarios versus its current focus on individual emergencies. The architecture is not currently designed for disaster-scale concurrent load.

---

## 9. Conclusion

Pulse addresses a specific, well-defined gap in Philippine emergency response: the inability of conventional digital emergency tools to function when users lack mobile data subscriptions, are unable to communicate verbally, or are in scenarios where peer assistance could arrive faster than official dispatch. The technical approach — raw TCP transmission to known server addresses, signed binary protocol, peer-response coordination — is grounded in empirically observed carrier behavior and proven international precedent.

The system is not positioned as a replacement for official emergency services. It is positioned as a complement that fills specific gaps that the 911 system alone does not address. This honest framing enables clear value communication, defensible scope, and the possibility of formal integration with official services rather than competition with them.

Development is intentionally phased and pilot-oriented. The system will be validated in a single community partnership before broader deployment. Open-source from inception, Pulse is built as civic infrastructure rather than as a commercial product, with the goal of being adopted, extended, and maintained by the broader Philippine civic technology community.

The next concrete step is empirical validation of carrier transmission behavior across all major Philippine carriers, producing a compatibility matrix that will inform the precise scope and reliability claims of the initial deployment.

---

## Appendices

### A. Technical specifications
Detailed protocol design, message formats, and server architecture are documented in the accompanying Backend Specification and Technical Specification documents.

### B. UX specifications
Complete user experience design, interaction flows, and screen specifications are documented in the accompanying UX Specification document.

### C. Compatibility matrix template
The carrier compatibility matrix to be produced during Phase 1 testing will document, for each combination of carrier and SIM state, the following metrics: TCP connection success rate, maximum reliable payload size, maximum sends per connection, latency percentiles, and acknowledgment reliability.

### D. Glossary

- **Alert** — A signal initiated by a user indicating an emergency situation requiring response.
- **Alerter** — A Pulse user who has initiated an alert.
- **Responder** — A Pulse user who has opted in to receive and respond to emergency alerts from nearby alerters.
- **Tier 0** — Anonymous use of Pulse without any registered account (Phase 4+ designation).
- **DRRMO** — Disaster Risk Reduction and Management Office, a Philippine local government unit.
- **LGU** — Local Government Unit, the Philippine designation for municipal-level government.
- **NDRRMC** — National Disaster Risk Reduction and Management Council, the Philippine national agency for disaster response coordination.
