# DESIGN — Stone: the IPC frame-budget is a per-Receiver, owner-set contract

**Status: DESIGN (pin the surface before code).** Drawn 2026-06-21. This is an
IPC-contract decision, not a thread-plumbing task — the frame cap governs how *any*
byte-framing transport bounds its receive buffer, and remote rides the same path.

## ⚠️ GROUNDING UPDATE (2026-06-21, post-compaction recolligere)
Two corrections, grounded against the live disk before the strike:

1. **The name is `:max-message-bytes`, NOT `:max-frame-bytes`** (intueri cast
   `aa72b240714dc11df`, weighed + settled in the curare): "frame" reads as MTU to a
   user. Rust internals keep `frame`/`max_frame_bytes`; the wat surface is
   `:max-message-bytes`. Every `:max-frame-bytes` BELOW is stale — read it as
   `:max-message-bytes`.

2. **The cap is an ACCUMULATION bound today, not a message-size limit — the stone
   must make it semantics B.** Grounding `next_complete_frame` (edn_shim.rs:1064)
   showed `TooLarge` fires ONLY in the no-newline branch (line 1071): a COMPLETE
   ('\n'-terminated) frame is returned as `Frame(end)` (line 1092) **regardless of
   size**. So a 256-byte terminated message under a 64-byte budget is DELIVERED
   today. That makes `:max-message-bytes` a lie unless the stone ALSO size-caps
   complete frames. The four-questions decided it: under the settled name, the
   accumulation-only semantics fails Obvious + Simple + Honest; the message-size
   semantics (B) passes all four. **THE HEART of this stone is therefore a one-branch
   change to `next_complete_frame`** (size-cap the `Frame(end)` return), not mere
   plumbing. B subsumes the existing anti-flood check (the no-newline branch stays).

**The RED probe is on disk and verified RED:**
`wat-tests/spawn/recv-budget-override.wat` — a process peer spawned with a 64-byte
`:max-message-bytes` rejects a 256-byte COMPLETE message (`println`'d). RED at HEAD:
`unknown function: :wat::spawn::process/max-message-bytes`. The 256-byte sizing forces
genuine semantics-B honoring (a present-but-ignored budget would deliver the frame →
still red).

3. **The MODEL (grounded with the builder) and the SURFACE.** Pipes are bidirectional;
   each side defends its OWN read pipe with its OWN limit — independent, no handshake,
   not uniform ("don't DoS me"). Between parent and process child: child→parent
   (`output_rx`, read by the parent's `recv'`) is **the gap**; parent→child (child
   stdin, read by `readln`) is **already defended** by `readln`'s `:max-buffer-bytes`.
   So this stone only adds the parent's `recv'` read-budget. The SURFACE is a 3rd field
   on the `ProcessOpts` **locus env record** (where spawn options live) — NOT an arg on
   `spawn-program'` (stays 2-arg) and NOT a new scoped form. There is NO composition
   wart: `process`/`process/env`/`process/post-spawn` are `defn` HELPERS over the full
   `(:wat::spawn::ProcessOpts …)` record constructor; N non-default fields = call the
   constructor with N values or write a helper. The sections BELOW that put
   `:max-frame-bytes` on connect'/listener' are deferred follow-ons (socket tier),
   not this stone.

## The flaw
The receive-side frame cap (`DEFAULT_MAX_FRAME_BYTES = 512 KiB`, edn_shim.rs:1008) is
**hardcoded on the peer path**: `take_frame` (comms/process.rs:884) passes the const, so
`recv'` / `select'` / `poll'` reject any frame > 512 KiB with no way for the owner to
raise (or lower) it. `readln` got a per-call escape hatch (`:max-buffer-bytes`,
io.rs:926); the persistent peer/connection never did. When we can't know users' message
sizes, the budget must be theirs to set. (Builder: *"recv needs to be overrideable… I
don't know what users need."*)

## The recognition — this is IPC-at-large, not threads
The cap is owned where the **accumulator** is: `comms::process::Receiver`
(process.rs:448; the `accumulator` field :457). That `Receiver` is the **shared
byte-framing endpoint for BOTH pipes (process) and sockets** — and **remote/TCP rides
the same `Receiver` through the narrow waist** (the unified socket transport already
carved for the unbuilt remote tier). So a budget on the `Receiver` covers process AND
remote in one stroke. Thread tier is **exempt**: crossbeam carries `Value`s, no byte
accumulation, no `take_frame`, no cap.

Three framing sites use the const today:
| site | path | today |
|---|---|---|
| `take_frame` comms/process.rs:884 | the **peer** receiver (recv'/select'/poll'; process + socket + remote) | hardcoded — THE gap |
| `channel/transfer.rs:267` | the channel-from-pipe receive | hardcoded |
| `read_framed_edn` io.rs:926 | `readln` / IOReader | already per-CALL `cap` (the only tunable one) |

## The contract (pin)
**The frame budget is a per-`Receiver` property**, defaulted to `DEFAULT_MAX_FRAME_BYTES`
(512 KiB), **set when the peer/connection is born**, surfaced uniformly through the
**locus-blind** spawn/connect/listener surface, and inherited by remote automatically
(remote *is* a socket `Receiver`).

- **Where it lives:** a `max_frame_bytes: usize` field on `comms::process::Receiver`
  (next to `accumulator`). `take_buffered_frame` (process.rs:573) / `take_frame` use
  `self.max_frame_bytes` instead of the const. Default = `DEFAULT_MAX_FRAME_BYTES`.
- **Where it's set:** `Receiver` construction (`pair()` + the Clone path that re-seeds a
  fresh accumulator must carry the cap forward). The const is the default a plain
  `pair()` uses; a `pair_with_budget(n)` (or a setter) carries an override.
- **The wat surface (locus-blind):** `:max-frame-bytes` on the byte-framing loci —
  `spawn-program' (process)` (via `ProcessOpts`, spawn.wat:53), `connect'`
  (eval_connect_prime, runtime.rs:19160), `listener'` (eval_listener_prime,
  runtime.rs:19037). Each threads the budget → the constructed `Receiver`. Default 512 KiB
  when omitted. `spawn-program' (thread)` does NOT take it (exempt — no byte framing).
- **`readln` stays per-CALL** (`:max-buffer-bytes`): it is one-shot, so per-call is the
  right granularity there. The persistent peer/connection is per-receiver. One doctrine
  (reasonable default, owner-overridable), correct granularity per surface — do NOT
  collapse them.
- **`channel/transfer.rs:267`**: route through the same per-receiver budget where it has a
  `Receiver`; if that site is a bare-buffer path without a `Receiver` handle, STOP and
  report (it may need its own threading — confirm its tier first).

## Exact change sites
- `src/comms/process.rs` — `Receiver` gains `max_frame_bytes`; `pair()` (+ a budgeted
  ctor) sets it; `Clone` carries it; `take_buffered_frame`/`take_frame` use it.
- `src/kernel/spawn.rs` — `spawn_process_peer` passes the ProcessOpts budget to the
  output `Receiver`'s construction (the err/input receivers keep the default — only the
  *output* accumulator is the attack surface a peer floods; confirm).
- `src/runtime.rs` — `eval_connect_prime` / `eval_listener_prime` thread `:max-frame-bytes`
  → the socket `Receiver`.
- `wat/spawn.wat` — `ProcessOpts` gains a `max-frame-bytes` field + a `process/max-frame-bytes`
  builder (mirroring `process/env` / `process/post-spawn`); default `DEFAULT_MAX_FRAME_BYTES`.
- `wat/` connect'/listener' surface — `:max-frame-bytes` opt (or positional, per the
  existing arg shape — match it).
- `src/comms/mod.rs` — the `RecvError::FrameTooLarge` Display already names the cap; keep
  it generic (don't bake "512 KiB" into the string — it's now per-peer).

## C decision (pinned)
**Per-`Receiver`, set at construction** (NOT per-`recv'`-call). The accumulator is
persistent across reads; a per-call cap is incoherent (which call's cap governs bytes
already buffered?). The four-questions confirm: Obvious (budget = receiver property),
Simple (one cap per receiver), Honest (the peer's frame-budget contract), Good-UX (set
once at birth, every recv honors it).

## RED probe (write + verify RED before the strike)
`tests/probe_recv_budget_override.rs` (or wat-tests/spawn/): spawn a process peer with a
TINY `:max-frame-bytes` (e.g. 64); the child `(println <a ~256-byte value>)` — bigger
than 64, far smaller than the 512 KiB default. The parent `recv'`s → expect
`FrameTooLarge` ("frame exceeded cap"). RED at HEAD: the override is ignored (cap is the
hardcoded 512 KiB), so a 256-byte value is well under → `recv'` returns the VALUE, no
rejection. GREEN after: the 64-byte budget is honored → rejected. (This proves the
override is *honored*, not just that some cap exists.)

## Gate
- the budget-override probe RED→GREEN.
- the over-cap deadlock proof still green (default-budget flood still rejects, no deadlock).
- recv'/select'/poll' + comms + channel suites green; lib 953/36/1; nursery floor; wat-tests floor.
- a peer with the DEFAULT budget behaves exactly as today (512 KiB) — no silent change.

## Out of scope (affirmative cuts)
- The remote/TCP tier impl — it inherits this for free (socket `Receiver`); not built here.
- The `print-raw'` kill — rides AFTER this: the over-cap proofs become tiny-cap peer tests
  (small `:max-frame-bytes` + a small `println` flood), no `print-raw'`, no 1 MiB doubling.
- `readln`'s per-call cap — unchanged (correct granularity for one-shot).
