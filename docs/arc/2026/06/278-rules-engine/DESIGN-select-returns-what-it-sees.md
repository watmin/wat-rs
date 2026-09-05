# DESIGN — select returns what it sees

**`select` stops raising on the peer wire.** `src/runtime.rs`. Correctness of a substrate
primitive. No perf work.

## WHY — a raise unwinds past the reader

`rst_peer_notify_baseline` states the contract it exists to hold:

> *"a genuine handler panic must surface to the client as a matchable `RecvOutcome::Lost`
> **VALUE** (never a raise, which would unwind past the reader)"*

A caller holds a `select` with arms ready — `Message`, `Closed`, `Lost`, `Shutdown`, `Malformed`.
**A raise skips all of them.** It is not lossy, it is *unhandleable*.

★ And its twins already return. `recv:25047` turns an undecodable frame into `RecvOutcome::Lost`
— its own comment explains why: *"the crash reason is the full `#wat.kernel/ProcessPanics [...]`
envelope text."* `poll:27194` turns the same failure into `ServiceEvent::Malformed`. **`select`
is the only reader on that wire that escapes.**

★★ **It is latent, not new.** `select` has only ever been pointed at peers that go silent or get
severed. Nothing had selected on a *panicking* process peer until arc 278's generated client
method moved from `recv` to `select` — which pointed it at every peer in the tree at once.

★★★ And under the totality work to come, **every raise is a place a match cannot be exhaustive.**
This is not new debt; it is found debt.

## ⛔ EVERY RAISE ALREADY HAS A VARIANT. NO NEW TYPE.

| `select` raises | should return | exemplar |
|---|---|---|
| `EDN decode failed` (`:26095`, `:26345`) | `ServiceEvent::Malformed [idx, cause]` | **`poll:27194`**, same file |
| `peer message is not valid UTF-8` | `ServiceEvent::Malformed` | **`poll:27194`** |
| `io_uring error` (`:26320`) | `ServiceEvent::Lost [cause]` | **`recv:25047`** |
| `interrupted by shutdown` (`:25919`, `:26109`, `:26251`) | `ServiceEvent::Shutdown` | **`select:26067` — itself** |

★★★★ **`ServiceEvent::Malformed` is constructed at 2 sites in the whole runtime and neither is
in `select`** — the variant exists for precisely this condition, callers match it, and the
primitive that should produce it raises instead.

★ **`select` already handles `SelectOutcome::Shutdown` two ways**: it returns
`ServiceEvent::Shutdown` at `:26067` and raises at three other sites. That is an inconsistency
inside one function, not a design.

## ⛔ THE ONE CONTRACT DECISION

**The scrub rides along.** Arc 294's ruling, `runtime.rs:6558`: *"a client learns no server
internals."* `Lost` and `Malformed` both carry a `Failure`, so every value this stone newly
returns must be reason-free — `message_only_failure`, the canonical constructor — exactly as the
client-facing `::Lost` arm already does.

★ Returning a richer value **without** the scrub would hand clients panic text through a side
door. `recv` scrubs; `select` has never had to, because it never got far enough to carry a cause.

## NO NEW PROBE, AND THAT IS A DECISION

Every target has a working exemplar in this file — `poll:27194`, `recv:25047`, `select:26067`.
Nothing here is a mechanism to establish; it is four sites made to agree with three that already
work.

## ⛔ WHAT THIS DOES NOT FIX

**The floor goes from 2 reds to 1, not to green.** `an_owner_drop_reaches_the_client_as_severed`
is the `CallOutcome::PeerGone` merge — our own debt from the CallOutcome stone — and it is a
different stone. **Do not report 1 red as a failure; report it as the expected remainder.**

## FILES

`src/runtime.rs` only.

## OUT OF SCOPE = REJECTED

- **Splitting `CallOutcome::PeerGone`** — the severed red. Named, separate, ours.
- **Changing `ServiceEvent`'s variant set.** Every condition already has a home.
- **`poll:27124`** (admin channel). An undecodable *admin* message is plausibly a protocol
  violation, not a death. Different channel, different question, not this stone.
- All perf work, and the rung-3 migration already checkpointed at `276f989dc`.
