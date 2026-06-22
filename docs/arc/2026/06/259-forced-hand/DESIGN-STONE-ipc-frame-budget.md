# DESIGN — Stone: the IPC frame-budget is a per-Receiver, owner-set contract

**Status: DESIGN (pin the surface before code).** Drawn 2026-06-21. This is an
IPC-contract decision, not a thread-plumbing task — the frame cap governs how *any*
byte-framing transport bounds its receive buffer, and remote rides the same path.

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
