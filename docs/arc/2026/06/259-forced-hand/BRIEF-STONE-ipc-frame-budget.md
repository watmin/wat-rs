# BRIEF — Stone: IPC read-budget (recv' tunable, per-Receiver, semantics B)

Read the DESIGN first: `DESIGN-STONE-ipc-frame-budget.md` (esp. the GROUNDING UPDATE).

## The model (grounded with the builder)
Pipes are bidirectional; **each side defends its own READ pipe with its own limit —
independent, no handshake, not uniform.** A read-budget is the cap you apply when
pulling a frame off a wire you read from; if the peer sends bigger, YOU tear down
(DoS defense). Between a parent and a process child there are two read pipes:
- child→parent (`output_rx`), read by the **parent** via `recv'` — **the gap this stone fills**.
- parent→child (child stdin), read by the **child** via `readln` — **already defended**
  by `readln`'s `:max-buffer-bytes` per-call cap. NOT touched here.

So this stone adds the parent's `recv'` read-budget to match the `readln` side.

## The work (one paragraph)
Give `comms::process::Receiver` a per-receiver `max_frame_bytes` (default
`DEFAULT_MAX_FRAME_BYTES`); make `next_complete_frame` size-cap COMPLETE frames too
(semantics B), not just un-terminated accumulation; let the parent set the budget on
the spawn locus (`ProcessOpts`) so the peer's `output_rx` honors it; generalize the
`FrameTooLarge` Display. GREEN gate: the on-disk RED probe
`wat-tests/spawn/recv-budget-override.wat`.

## Rooms (read in order; file:line + why)
1. `src/edn_shim.rs:1064` — `next_complete_frame`. **PART A, the heart.** Line 1071 is
   the existing no-newline `TooLarge` check (KEEP). The `Complete` (1092) and
   `Malformed(_)` (1099) arms return `Frame(end)` with NO size check — the gap. Add a
   size check on `end` before returning the frame. Update the doc comment (1043-1056),
   which describes accumulation-only semantics today.
2. `src/comms/process.rs:448` — `Receiver<T>` struct (`accumulator` :457). **PART B.**
   Add `max_frame_bytes: usize`.
   - `:572` `take_buffered_frame` → thread `self.max_frame_bytes` into `take_frame`.
   - `:883` `take_frame(acc)` calls `next_complete_frame(acc, DEFAULT_MAX_FRAME_BYTES)`
     at :884 → change signature to `take_frame(acc, max_frame_bytes)`, pass through.
   - `:703` `Clone` (fresh accumulator + ring per :439) → carry `max_frame_bytes`.
   - `:1527` `pair()` → set `max_frame_bytes = DEFAULT_MAX_FRAME_BYTES`. Add
     `pair_with_budget(max)`; refactor `pair() = pair_with_budget(DEFAULT_MAX_FRAME_BYTES)`.
     Sender needs NO field.
3. `src/kernel/spawn.rs:706` — `spawn_process_peer(forms, post_spawn_fn, env_fn, sym, list_span)`.
   **PART C.** Add a `max_frame_bytes: usize` param. The OUTPUT pair (`:728`,
   `output_tx/output_rx`) is the parent's `recv'` surface — build it with
   `pair_with_budget(max_frame_bytes)`. Input pair (`:718`, child stdin) and err pair
   (`:744`, crash channel) KEEP `pair()` default — the budget governs the parent's
   recv' of user messages on output_rx.
4. `src/kernel/spawn.rs:~460-495` — `eval_kernel_spawn_process_prime`. **PART C.** Reads
   args[0]=forms, [1]=post_spawn_fn (fn), [2]=env_fn (String). Add args[3] = budget
   (i64; copy the int-extraction idiom from how the value/i64 is read elsewhere —
   TypeMismatch on non-int). Pass to `spawn_process_peer`. This is the SAME
   field-flows-as-extracted-arg pattern post_spawn_fn/env_fn already use — NOT a new
   user-facing arg.
5. `wat/spawn.wat:53` — `ProcessOpts` record (`post-spawn-fn`, `env-fn`). **PART C.**
   - Add `max-message-bytes <- :wat::core::i64` as a 3rd field.
   - Update the 3 existing helpers (`process` :78, `process/post-spawn` :83,
     `process/env` :86) to pass a default for the new field via a new const
     `(:wat::core::def :wat::spawn::DEFAULT-MAX-MESSAGE-BYTES 524288)` (comment: mirrors
     `DEFAULT_MAX_FRAME_BYTES`, src/edn_shim.rs:1008 — don't scatter the literal).
   - Add a `process/max-message-bytes` helper (sibling to `process/env`): sets the
     budget, defaults the other two. (The full `ProcessOpts` ctor stays available for
     any multi-field combination — no new form needed beyond this convenience.)
   - The `spawn-program'` process clause (the `[ProcessOpts, Vector<WatAST>]` arm, the
     2-arg sig UNMOVED) currently passes `(ProcessOpts/post-spawn-fn locus)` +
     `(ProcessOpts/env-fn locus)` to `spawn-process'`. Add
     `(:wat::spawn::ProcessOpts/max-message-bytes locus)` as the 4th.
6. `src/comms/mod.rs:945` — `FrameTooLarge` Display. **PART D.** Drop the const ref AND
   the "un-terminated" qualifier (B rejects complete frames too): e.g. "frame exceeded
   cap (message larger than the receiver's max-message-bytes budget)". Keep the
   substring "frame exceeded cap".

## Implementation sketch (fill it, don't reinvent)
PART A:
```rust
match edn_frame_status(prefix_str) {
    EdnFrameStatus::Complete | EdnFrameStatus::Malformed(_) => {
        if end > max_bytes { return FrameScan::TooLarge(end); }  // semantics B
        return FrameScan::Frame(end);
    }
    EdnFrameStatus::Incomplete => { search_start = end; continue; }
}
```

## Blast radius (bounded)
`src/edn_shim.rs`, `src/comms/process.rs`, `src/comms/mod.rs`, `src/kernel/spawn.rs`,
`wat/spawn.wat`. `spawn-program'` stays 2-arg. Do NOT touch connect'/listener', readln,
or channel/transfer.rs (named follow-ons — Out of scope).

## STOP triggers (halt + report; do NOT improvise)
1. **STOP-1:** if PART A turns any EXISTING green test red (a test that sends a complete
   frame larger than its budget), STOP and report the test — do not raise the default.
2. **STOP-2:** `src/channel/transfer.rs:267` uses `DEFAULT_MAX_FRAME_BYTES` on a bare
   buffer. LEAVE it on the default; only report if it obviously has a `Receiver` handle.
3. **STOP-3:** if the i64 extraction idiom for args[3] isn't obvious from existing arg
   handling, STOP and report — don't guess the Value variant.
4. **STOP-4:** if adding the 3rd `ProcessOpts` field cascades to positional struct-new
   sites beyond the 3 helpers + the one `spawn-program'` clause, STOP and list them.

## Out of scope (named follow-ons, NOT this stone)
- readln `:max-buffer-bytes` → `:max-message-bytes` rename (intueri family alignment).
- connect'/listener' read-budget locus option (socket tier — same pattern; they inherit
  PART A+B's foundation, default preserved).
- The remote/TCP tier (inherits for free).

## Do NOT commit
Leave all changes uncommitted. Report: a filled EXPECTATIONS scorecard with REAL command
outputs, the files+regions changed, any STOP that fired, and any delta from this brief.
The orchestrator weighs against its own re-run before committing.

---

## EXPECTATIONS (scorecard — fixed BEFORE the strike)
| # | what | command | expected |
|---|---|---|---|
| 1 | budget probe GREEN | `cargo test --release -p wat --test test budget` | `1 passed` (should-panic sees "frame exceeded cap") |
| 2 | over-cap flood still rejected, no deadlock | `cargo test --release -p wat --test test overcap` | green |
| 3 | flood + supervisor :Lost green | `cargo test --release flood_no_deadlock supervisor_select_lost` | green |
| 4 | IPC framing + round-trips green | `cargo test --release ipc_framing process_peer_ipc multiline` | green |
| 5 | full lib unit suite | `cargo test --release --lib` | floor 953/36/1, no regressions |
| 6 | full wat-tests suite | `cargo test --release -p wat --test test` (after `touch tests/test.rs`) | 267 passing (266 floor + new); only pre-existing `test-run-string-entry-direct` fails |
| 7 | clippy clean on touched files | `cargo clippy --release` | no new warnings |

Runtime: 15-30 min (release rebuild + full suite). Trap-doors: PART A is a shared framer
change (also governs readln/socket/channel receivers — Exp 5/6 guard it); the 3rd
ProcessOpts field is a positional-struct cascade (STOP-4).
