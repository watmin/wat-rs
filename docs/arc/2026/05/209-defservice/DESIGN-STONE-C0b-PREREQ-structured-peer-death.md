# DESIGN-STONE — structured peer death: the prime crash path carries `Failure`, not a flattened `String`

> Prerequisite for **C0b.1b** (`select'` learns the `Listener'` — its `:Crashed` variant carries
> a `Failure` value). True lineage: arc 214's unified `Peer'`/`recv'`/`select'` — the prime path
> that regressed the structured death the old channel path had. Surfaced + gated in the arc 209
> defservice campaign. Inquisitor draws; Shadowdancer (sonnet) executes; Inquisitor weighs.

## Why this exists — a diagnostics regression, proven on the disk

The structured death type **already exists** and is fully built:
- `:wat::kernel::ThreadDiedError` / `ProcessDiedError` — the death enums (accessors
  `*/message`, `*/to-failure`, `runtime.rs:4376`).
- `:wat::kernel::Failure` — the **unified** death record (`failure-from-thread-died
  [chain <- Vector<ThreadDiedError>] -> Failure`; `failure-from-process-died` for the other
  tier; `sandbox.wat:53`).
- Built from `AssertionPayload` (message + actual + expected + location + frames +
  upstream_chain + thread_name; `assertion.rs`) via existing builders —
  `thread_died_error_panic(message, assertion)` (`runtime.rs:19913`) →
  `single_died_chain`/`conj_died_chain` (`:20055`).
- The **old** channel `recv` / `join-result` returns `Result<_, Vector<DiedError>>` — *structured,
  with the cascade chain* (`runtime.rs:18922`).

The **prime** path flattened all of it:
- `peer.rs:86` — `crash: Receiver<String>` (the thread death-time channel carries only a String).
- `spawn.rs:472` — `let (message, _assertion) = extract_panic_payload(payload); let _ =
  crash_tx.send(message);` — **the structured `AssertionPayload` is extracted and discarded**;
  only the message String is sent.
- `PeerRecvError::Crashed(String)` (`spawn.rs:150`) — the crash carrier is a String.
- `recv'` raises a `MalformedForm { reason: <that String> }` (`runtime.rs:22752`); the 1-arg
  `select'` doesn't even read the crash channel — it raises a generic `"peer closed / thread
  exited"` (`:23180`), losing the reason entirely.

So `recv'` on a crashed peer **today** loses actual/expected — a live regression vs. the path it
replaced. This stone restores the structure the prime path dropped. We do **not** mint a new
type — minting a `CrashReason` would duplicate `Failure`/`DiedError` (a `solvere` duplication).

## What it delivers

The prime crash path carries the **structured death**, and the consumers surface
`:wat::kernel::Failure` (the unified type — hides thread-vs-process, wraps the cascade chain):

- **`select'`** delivers the `Failure` as a **first-class value** — this is the load-bearing
  win (C0b.1b's `:Crashed [idx <- :i64  reason <- :wat::kernel::Failure]`; the loop inspects it
  as data).
- **`recv'` / `try-recv'`** raise/return with the structured reason. Per `raise!`'s own doctrine
  (*"the message field IS the data field… the EDN-rendered form of whatever the panic carried"*,
  `runtime.rs:10566`), the raise reason is the **EDN-rendered `Failure`** — recoverable by a
  receiver via `(:wat::edn::read …)`. The "raises on crash" contract is unchanged; the reason
  gets honest.

## The one contract decision (pinned)

`Failure` is the carrier at every surface (not `CrashReason`, not tier-specific
`ThreadDiedError`) — it is the substrate's unified death record and already wraps the
`Vector<DiedError>` cascade chain. The crash channel carries enough structure
(`(message, Option<AssertionPayload>)`) for the recv/select layer to build the `Failure` via the
existing builders. A clean (non-crash) disconnect stays `Disconnected` → graceful close (no
`Failure`).

## Decomposition (both ship — ordered, NOT deferred)

- **Sub-stone A — thread tier (the C0b.1b prerequisite).** `peer.rs` `crash` channel carries
  `(String, Option<AssertionPayload>)` (or a small `CrashInfo` newtype) instead of `String`;
  `spawn.rs:472` sends the full `extract_panic_payload` result (stop discarding `_assertion`);
  `PeerRecvError::Crashed` holds it; `Thread::recv` returns it; `recv'`/`try-recv'` build the
  `Failure` value (reuse `thread_died_error_panic` + `single_died_chain` + `to-failure`) and
  raise EDN-rendered; the **1-arg `select'`** reads `.crash` on a closed peer and raises with the
  structured reason (it ignores it entirely today).
- **Sub-stone B — process tier (class alignment).** The process bundle already carries a richer
  `#wat.kernel/ProcessPanics [...]` **EDN envelope** over its Err channel (`spawn.rs:150` doc) —
  more structured than the thread String. Align process `recv'`/`try-recv'` to parse that
  envelope into the same `Failure` value (`edn::read` → reconstruct), so both tiers surface one
  shape. No new wire format — the envelope already exists.

C0b.1b consumes Sub-stone A (thread tier). Sub-stone B closes the class so `recv'` is honest on
both tiers. Both ship in this campaign.

## ⚙ PROBE REFINEMENT (2026-06-12) — carry the EDN envelope, not a struct

The RED probe (`probe_arc209_structured_peer_death`) revealed a cleaner fix than the channel
re-typing below. At death, the full structure is **already rendered as an EDN envelope** —
`#wat.kernel/AssertionFailure {:message … :actual … :expected … :frames …}` (printed by
`panic_hook.rs:126` `write_assertion_failure`, via the `pub(crate)` `payload_to_edn`). The
thread crash-send site just throws it away and sends the bare message. The **process tier
already** sends its `#wat.kernel/ProcessPanics` envelope over the Err channel.

So the refined fix (smaller, and it unifies both tiers on "carry the EDN envelope"):
- **The crash channel STAYS `Receiver<String>`** — it carries the *envelope*, not the bare
  message. No `PeerRecvError`/`CrashInfo` re-typing.
- **`spawn.rs:472`**: when an `AssertionPayload` is present, send
  `format!("#wat.kernel/AssertionFailure {}", wat_edn::write(&payload_to_edn(&assertion)))`
  (factor a `String`-returning helper out of `write_assertion_failure`); else send the message.
- **`recv'`/`try-recv'` are UNCHANGED** — they already surface the crash String; it is now the
  rich envelope. The probe goes green from the `spawn.rs:472` change alone.
- **1-arg `select'`** reads `.crash` on a closed peer and surfaces the envelope (today it maps to
  a generic `"peer closed"`, losing the reason — the one `select'`-side change).
- **`Failure` as a first-class VALUE** is C0b.1b's job: its `:Crashed` arm `edn::read`s the
  envelope into a `Failure` value. This stone makes the envelope *travel*; C0b.1b *reconstructs*
  the value. (Consistent with `raise!`'s doctrine: the reason String is EDN, recoverable via
  `edn::read`.)

The mechanics below (channel re-typing) are superseded by this envelope approach — kept for the
reasoning trail.

## Mechanics (the strike path — fill it, don't invent the shape)

**Sub-stone A:**
1. `peer.rs` — `Thread.crash: Receiver<String>` → `Receiver<CrashInfo>` where
   `CrashInfo = (String, Option<AssertionPayload>)` (a tuple or a tiny struct in `kernel/peer.rs`
   or `assertion.rs`). Mirror the `crash_tx`/`crash_rx` pair type in `spawn.rs:358`.
2. `spawn.rs:472` — `let (message, assertion) = extract_panic_payload(payload); let _ =
   crash_tx.send(CrashInfo { message, assertion });` (stop discarding).
3. `spawn.rs:150` — `PeerRecvError::Crashed(String)` → `Crashed(CrashInfo)`.
4. `peer.rs:121` (`Thread::recv`) — on output-EOF, `self.crash.recv()` →
   `Ok(info) => Crashed(info)` / `Err(_) => Disconnected` (unchanged logic, richer payload).
5. `recv'` / `try-recv'` (`runtime.rs:22752`) — on `Crashed(info)`, build the `Failure` value
   (`thread_died_error_panic(info.message, info.assertion)` → `single_died_chain` → the
   `to-failure` Rust helper backing `ThreadDiedError/to-failure`), EDN-render it into the raise
   reason. (`try-recv'`: surface in its Err arm — confirm its current shape at probe time.)
6. 1-arg `select'` (`runtime.rs:23180`) — on a peer's output-EOF (`Err`), read that peer's
   `.crash` (the guards Vec is in hand) and raise with the structured reason, exactly as `recv'`.

**Sub-stone B:** the process `recv'` arm (`runtime.rs:~22800`) and `Process::recv` —
`Crashed(envelope)` where the envelope is EDN → `edn::read` → reconstruct the `Failure` value
(the `ProcessDiedError` chain is already the envelope's shape; `process_died_error_panic` +
`to-failure` mirror the thread path).

**The synthesis is a reuse** — every builder exists; this stone reconnects the prime path to
them. The thread tier is the C0b.1b dependency; do A first, B second.

## Files touched

| File | Change |
|---|---|
| `src/kernel/peer.rs` | `crash` channel + `PeerRecvError::Crashed` carry `CrashInfo`; `Thread::recv` payload |
| `src/kernel/spawn.rs` | `spawn.rs:472` send the full payload (stop the `_assertion` discard); `crash_tx`/`crash_rx` type |
| `src/runtime.rs` | `recv'`/`try-recv'`/1-arg `select'` build + surface `Failure` (thread A + process B arms) |
| `tests/nursery/probe_arc209_structured_peer_death.rs` | RED probe (Inquisitor writes it STRIKE-READY) |

Blast radius: the prime peer-death path only. The old channel `recv`/`join-result`/`Failure`
builders are **reused, not changed**. `send'`, the connection verbs, and the 2-arg `select'`
(C0b.1b) are not touched here.

## Out of scope = affirmatively rejected

- **A new `CrashReason` type** — REJECTED: `Failure`/`ThreadDiedError`/`ProcessDiedError` already
  are the structured death; a parallel type is duplication.
- **Changing the "`recv'` raises on crash" contract** — unchanged; only the raise *reason* gets
  structured (EDN-rendered `Failure`).
- **The 2-arg `select'` + `SelectEvent`** — that's C0b.1b; this stone only makes the `Failure`
  value reachable so `:Crashed` can carry it.
- **`AssertionPayload` field changes / arc-260 kwargs** — out; this stone carries the payload
  intact, it does not reshape it.

## Gate (Inquisitor re-runs each; Shadowdancer reports, Inquisitor weighs)

1. `cargo test --release -p wat --test nursery probe_arc209_structured_peer_death -- --test-threads=1`
   → GREEN. The probe: a thread peer dies via `assert-eq` (mismatched); `recv'` on it surfaces a
   reason from which actual/expected are **recoverable** (today they're gone). A process peer the
   same.
2. `cargo test --release -p wat --test nursery -- --test-threads=1` → only the 4 known reds, zero new.
3. `cargo test --release --test test 2>&1 | tail -3` → wat-tests unbroken (the deftest'/death
   surface still green — the structured reason is additive).
4. `cargo build --release` clean; `cargo clippy` clean in the touched homes.

## Estimate

~150–250 lines Rust across the two sub-stones (channel type + send site + the recv/select
surfacing on both tiers). Every builder reused. One Shadowdancer strike per sub-stone behind a
committed RED probe, or one strike for both — orchestrator's call at fire time.
