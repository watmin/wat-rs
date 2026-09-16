# DESIGN — every Status arm is named

**Drawn 2026-09-15.** Builder's ordering: *"1, then 3, then B"* — this is **1**, the highest-leverage
instance of the blindness class `the-blind-sends-face-their-outcome` (piece A, `e03fc7f46`) proved for
`SendOutcome`. A is send-side and hand-written; this is **recv-side and macro-generated**, so it ships
into **every service**.

## Why a sweep never found these

A textual wildcard sweep certifies the corpus clean and **cannot see these forms**. Their arm heads are
unquoted symbols inside quasiquoted macro bodies — `~status-stopped-kw`, `~status-faulted-kw`,
`~reply-variant-kw` — so no regex can attribute them to an enum. Measured on the live tree: **1168 match
forms whose arm heads name no literal enum, 394 of them wildcarded**, and `wat/service.wat` alone holds
**14**, of which **12 are macro-generated**. The instrument that says "clean" is blind exactly where the
code is highest-leverage.

⚠ **And the sweep that found them has a measured defect of its own**, stated so nobody quotes its
headline: its `head()` does not recurse into parameterized patterns, so a match whose arms are *all*
`((Enum::Variant x) …)` with no nullary arm also lands in the invisible bucket. Its "one live wildcard"
figure is **provisional**. The 9 sites below were read in context, not taken from that number.

## The measurement — 9 Status sites, and a pattern in four pairs

`Status` is a fixed six-variant enum (`wat/service.wat:1604`): `Started(addr)` · `Stopped(resp)` ·
`Hibernated(snapshot)` · `PeersAllowed` · `PeersDenied` · `Faulted(cause)`. So these matches **can** be
exhaustive — unlike the per-surface `Reply` sites, see §Scope.

| site | names | falls into `_` | the `_` says |
|---|---|---|---|
| `:1657` extract-addr | Started | the other **5** | "unexpected Status variant (expected Started)" |
| `:3145` stop, 1st recv | Stopped, **Faulted** | 4 | "expected Status::Stopped" |
| `:3166` stop, 2nd recv | Stopped | 5 **incl. Faulted** | "expected Status::Stopped" |
| `:3235` hibernate, 1st | Hibernated, **Faulted** | 4 | "expected Status::Hibernated" |
| `:3252` hibernate, 2nd | Hibernated | 5 **incl. Faulted** | "expected Status::Hibernated" |
| `:3323` grant, 1st | PeersAllowed, **Faulted** | 4 | "expected Status::PeersAllowed" |
| `:3329` grant, 2nd | PeersAllowed | 5 **incl. Faulted** | "expected Status::PeersAllowed" |
| `:3381` revoke, 1st | PeersDenied, **Faulted** | 4 | "expected Status::PeersDenied" |
| `:3387` revoke, 2nd | PeersDenied | 5 **incl. Faulted** | "expected Status::PeersDenied" |

## ⭐ THE FINDING: this LOCATES the open unsolicited-frame residue, in code

Four pairs, one shape: **the first recv handles `Faulted`; the second — the one-level drain D1-a added —
does not.** BREADCRUMB OPEN item 2 says it in prose: *"mitigated by a one-level drain in
/stop//hibernate//grant//revoke, and **two queued faults still break it**."* Here is the mechanism, and
`wat/service.wat:3160` documents it against itself: *"A second unexpected variant still raises."*

⛔ **So a second queued `Status::Faulted` kills the owner with the message "defservice stop: expected
Status::Stopped"** — which is a lie: a `Faulted` *did* arrive, and the message names the thing that
didn't. Two different worlds print one line
(`[[feedback_a_fallback_that_collapses_failures_reports_nothing]]`), and the crash is on a **recoverable**
error, which the builder's standing rule forbids: *"no one is allowed to crash on a recoverable error."*

## The change

**At all 9 sites, name all six variants. No wildcard survives.**

1. **The success variant** keeps its existing body, byte-identical.
2. **`Faulted` on a FIRST recv** keeps its existing one-level drain, byte-identical.
3. ⭐ **`Faulted` on a SECOND recv STOPS RAISING.** It returns a faced outcome naming what happened —
   `StopOutcome::GaveUp` for stop/hibernate, `GateOutcome::GaveUp` for grant/revoke — with `waited-ms`
   computed from the method's existing `~*-t0-sym` and a `last` string saying a second `Faulted` arrived
   and the fault stream outran the one-level drain. **This is the only behaviour change in the stone**,
   and it converts a forbidden crash into a faced value.
4. **The remaining variants** keep raising — they are genuine protocol violations at that point — but
   **each names the variant that actually arrived**, so two worlds cannot print one line.

## Scope — what is deliberately NOT touched

- **`:2759` (`~reply-variant-kw`/`~reply-failed-kw`), `:2940` (`~accepted-ctor-kw`), `:3040`
  (`~success-ctor-kw`).** Their variant sets are **per-surface and generated**, so "name every variant"
  has no fixed meaning; the wildcard there is the desync detector (a reply for a DIFFERENT op). ⭑ They
  should each gain a comment saying the wildcard is DELIBERATE and why — an unmarked wildcard and a
  reasoned one look identical to the next reader, which is how these 9 survived. The macro does know each
  surface's ops and *could* splice exhaustive arms; that is a different stone.
- **An unbounded fault drain.** Rule 3 bounds the damage at two; it does not make the drain a loop. "How
  many unsolicited notices may queue" is the real question and it belongs to the unsolicited-frame class
  (OPEN item 2), whose actual fix is separating notifications from the request/reply channel.
- **`src/`.** Nothing here reaches Rust. The checker lint that would make this class impossible is the
  builder's item **3**, next.

## Trap-doors

1. ⛔ **These are quasiquoted macro bodies.** Every added arm is generated code: use the `~kw` symbols
   already bound in scope (`wat/service.wat:1526`–`1559`), never a literal `:wat::service::Status::…`,
   which does not exist — `Status` is per-service.
2. **Hygiene.** `_cause` / `resp` / `snapshot` in existing arms are match-arm binders the checker skips;
   copy that spelling rather than inventing new binder names.
3. **`waited-ms` must come from the method's own `t0` sym**, which is already in scope at each site
   (`~stop-t0-sym` and its siblings). Do not mint a second clock.
4. **The floor is the proof.** Every service in the corpus is generated by this macro, so a hygiene error
   or a type error surfaces as a broad red, not a subtle one.
5. **Count with the reader, not by eye**: after the change, **zero** wildcarded Status matches must remain
   in `wat/service.wat`, and the three per-surface sites must still be exactly three.
