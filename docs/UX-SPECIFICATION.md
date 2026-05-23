# Pulse — UX Spec

*Maverick Fabroa ([@mavyfaby](https://github.com/mavyfaby))*

Re-implementation of the previous Pulse app.

## Principle

User holds a button. Help comes. Everything else is automatic or optional.

## Alerter flow

1. Hold SOS button (2.5s)
2. Countdown (10s, big cancel button)
3. Alert fires with location + device info
4. Active alert screen shows responders
5. "I'm safe now" button resolves

No emergency type selection. No voice recording gesture. No detail entry. No confirmations.

## Home screen

- Large SOS button, centered, dominates
- Subtle pulse animation (brand metaphor)
- High contrast, icon + label "Hold for Emergency"
- Small menu icon in corner → settings
- Nothing else

## Hold-to-activate

- Touch → haptic, button scales down, ring fills over 2.5s
- Release before complete → return to idle, no penalty
- Complete → strong haptic + tone (overrides silent mode)

## Countdown screen

- Large countdown numeral
- "Alerting in X seconds"
- Massive cancel button (bottom half)
- Audio + haptic each second
- Cancel is instant, no confirmation

## Active alert screen

- "Help is coming"
- Responders appear as they accept
- Single "I'm safe now" button
- No required interaction

## Resolution

Tap "I'm safe now." Done.

## Responder flow

Setup:
1. Toggle "I want to help in emergencies"
2. Allow location + notifications
3. Optional: skills checkboxes (skippable)

Receiving:
- Notification overrides silent mode
- Tap → shows location, distance, two buttons: "I'm coming" / "Can't help"

En route:
- Alerter location updates live
- Voice/audio plays automatically (Phase 2)
- Buttons: "I'm on scene" → "Situation resolved"

## Onboarding

1. SOS screen immediately (usable in 2s)
2. Brief overlay: "Hold the button to send help."
3. Optional practice mode
4. Optional profile (skippable, can do later)

No phone verification. No account creation. Device generates a cryptographic identity automatically.

## Profile (optional)

- Name (optional)
- That's it for Phase 1

Medical info and emergency contacts deferred to Phase 3+ when professional responders join the platform. Until then, peer responders triage on arrival.

## Settings

- Profile
- Responder mode (toggle, radius hidden behind "Advanced")
- Privacy (export data, delete data, ambient recording toggle)
- About (practice mode, help, privacy policy, terms)

## Witness alerts

Phase 1: no distinction. Same button works for self or witness. Responder sorts it out on arrival.

Phase 2+: optional "Who is this for? [Me / Someone else]" prompt on active alert screen, after alert has fired. Skippable. Never blocks activation.

## Connection Status

The app runs a background probe periodically — even when not in use — to confirm the server is reachable before an emergency happens. The user sees a small status indicator on the home screen.

States:

- 🟢 **Ready** — last probe succeeded within 10 minutes
- 🟡 **Checking** — probe in progress or last probe was 10-30 minutes ago
- 🔴 **Unreachable** — last probe failed, alerts may not deliver
- ⚪ **Unknown** — fresh install, no probe run yet

The indicator is small and unobtrusive when green. It becomes prominent only when the state is red — the one moment the user needs to know.

Tapping the indicator shows: last successful connection time, current server status, and a manual "Check now" button.

"Ready" means specifically: the raw TCP path to the Pulse server is working right now, even on a no-subscription SIM. Not just "you have internet."

---



- SOS button is never disabled
- Cancel is instant; activate is deliberate (asymmetric friction)
- No modals during an emergency
- No mandatory profile fields
- Notifications only for emergencies
- No login flow blocks emergency use
- Override silent mode only for emergency events

## Screens in Phase 1

1. Onboarding (skippable)
2. Home / idle
3. Hold in progress
4. Countdown
5. Cancelled
6. Sending
7. Active alert
8. Resolved
9. Send failed
10. Practice mode
11. Profile (optional)
12. Settings
13. Responder onboarding
14. Responder incoming alert
15. Responder en route

If a screen isn't on this list, it isn't in Phase 1.

## Not in Phase 1

- Emergency type selection
- Hold-to-record voice (ambient is automatic)
- Medical info / emergency contacts
- Phone verification
- Account tiers
- Organizational accounts / command centers
- ID verification
- In-app chat
- Silent / duress mode
- Witness alert UI

## Acceptance criteria

- Alert sendable in <13s from app open, fresh install
- Home screen has nothing competing with SOS button
- No emergency screen requires reading or decisions
- Onboarding ≤30s, skippable in ≤10s
- App fully usable with zero personal data entered
- Practice mode works
- 15 screens total, no more
- Accidental activation <1% in field testing
