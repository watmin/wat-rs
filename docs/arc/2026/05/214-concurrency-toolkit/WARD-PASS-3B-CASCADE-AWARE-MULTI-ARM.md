# Arc 214 Slice 3 Stone B — WARD PASS

Per the kernel-impeccability protocol (INTERSTITIAL § 2026-05-19): per-stone trust gate = BRIEF scorecard verification + ward pass before commit. This doc captures the ward round-trip for Stone B (cascade-aware multi-arm POLL_ADD).

5 wards (gaze + forge + reap + sever + temper) — same set established in Slice 2 + Stone A.

## Round 1 — initial ward pass (after sonnet SCORE Mode A)

Target: `src/comms/process.rs` — module-level doc + new `PollOutcome` enum + new `wait_for_data_or_cascade` private fn + refactored `Receiver::recv` body. Sender / take_frame / pair UNCHANGED from Stone A (out of Round 1 scope).

### gaze — 3 findings (2 L1 stale-doc + 1 L2 mumble)

| Site | Level | Observation |
|---|---|---|
| process.rs:8 | L1 | Module doc section header `## Stone A scope (this commit)` is stale — Stone B is the current commit; the parenthetical lies |
| process.rs:124 | L1 | Receiver struct doc says `NOT cascade-aware (Stone B)` — cascade IS now wired in Stone B; the doc contradicts the implementation |
| process.rs:254 | L2 | `BROAD_TOKEN` mumbles; `BROADCAST_TOKEN` speaks (multi-line scope spanning push + drain + tiebreak) |

Spark observations: positive on `PollOutcome` enum variant doc comments + `wait_for_data_or_cascade` doc explaining broadcast-wins-ties + Stone E debt callout + module-level doc's new "Cascade contract (Stone B)" section.

### forge — CLEAN

> "PollOutcome enum: clean ADT; two variants; private; correctly scoped. wait_for_data_or_cascade: pure-ish helper; values in/out; side effects honest. DATA_TOKEN / BROADCAST_TOKEN constants scoped tightly inside fn. The `as u32` casts on libc::POLLIN | libc::POLLHUP are zero-extending; not lossy. SAFETY comment present + names fd-ownership-elsewhere + lifetime invariants. Defensive empty-CQE-drain return is honest. Receiver::recv composition: three concerns in clean sequence."

Both Hickey and Beckman lenses pass. The new unsafe block carries an honest SAFETY comment naming the fd-ownership-elsewhere + lifetime invariants (Stone A round-1 forge lesson applied at construction).

### reap — CLEAN

> "PollOutcome::DataReady constructed at wait_for_data_or_cascade + matched in Receiver::recv. PollOutcome::Shutdown constructed at wait_for_data_or_cascade + matched in Receiver::recv. wait_for_data_or_cascade called at Receiver::recv:169. DATA_TOKEN + BROADCAST_TOKEN both constructed AND consumed in the same fn scope. No leftover Stone A unreachable code. No TODO/FIXME/unimplemented! markers. Stone A items (Sender + take_frame + pair) still alive. Honest-delta: NONE. REAP CLEAN."

Sonnet's "NONE" honest-delta declaration verified — nothing else slipped in beyond the BRIEF.

### sever — CLEAN

> "PollOutcome: one concern (encoding the outcome). wait_for_data_or_cascade: one concern (block via POLL_ADD until either arm fires; broadcast wins ties); does NOT touch self.accumulator; does NOT call opcode::Read. Receiver::recv: one concern (orchestration); three structural steps cleanly separated and non-overlapping (fast-path accumulator check, cascade-poll step, Read step). Bootstrap fallback (broadcast_fd < 0) is a top-of-loop clean branch; no mid-loop conditional, no flag toggling."

The cascade-poll step and the Read step are structurally distinct in `Receiver::recv`; `wait_for_data_or_cascade` does not braid into the Read concern.

### temper — CLEAN

> "SHUTDOWN_BROADCAST_READ_FD.load(SeqCst) done ONCE at top of recv() (outside loop). read_fd cached ONCE before loop. CQE drain loop bounded by ring capacity (4 entries; only 2 SQEs pushed). Vec::extend_from_slice + accumulator.borrow_mut() patterns unchanged from Stone A. The only new work per loop iteration is `broadcast_fd >= 0` (single i32 comparison; zero cost). Known-deferred (Stone E): per-call IoUring::new(4) + per-call IoUring::new(2). The double IoUring::new() overhead per recv() call is documented in the module + recv doc comment. No flag."

Stone B's added overhead is one i32 compare per iter + one extra IoUring::new(4) per recv() call. The latter is the known Stone E deferral; the former is essentially free.

## Orchestrator design decisions (judgment calls)

**Decision 1: 2 L1 stale-doc findings + 1 L2 mumble** — FIX. The doc-lies finding category is real per the "what is inscribed is inscribed" doctrine (feedback_inscription_immutable applies to historical artifacts, but ACTIVE doc claims must be honest about current state). The `BROAD_TOKEN` mumble is a one-rename fix.

## Fix pass — orchestrator-direct (3 surgical edits)

| # | Fix | File:line |
|---|---|---|
| 1 | Module doc section header: `## Stone A scope (this commit)` → `## Current scope (through Stone B)`; body text updated to reflect that Stone B added cascade-aware (no longer "NO cascade-aware multi-arm") | process.rs:7-14 |
| 2 | Receiver struct doc: removed the `NOT cascade-aware (Stone B)` lie; now declares `Cascade-aware (Stone B): recv wakes on substrate shutdown via io_uring multi-arm POLL_ADD on SHUTDOWN_BROADCAST_READ_FD` | process.rs:124-128 |
| 3 | `BROAD_TOKEN` → `BROADCAST_TOKEN` (replace_all — constant declaration + push site + match arm) | process.rs (3 sites) |

Mechanical verification post-fix:
- `cargo test --release --test probe_comms_process` 6/6 PASS (doc + rename changes are no-op for runtime behavior)

## Round 2 — gaze re-pass (only the one ward had findings)

### gaze re-pass — CLEAN

> "Finding 1 — Module doc section header: Now reads `## Current scope (through Stone B)` — honest, no longer references a stale Stone A label. Resolved.
> Finding 2 — Receiver struct doc: Now reads `Cascade-aware (Stone B): recv wakes on substrate shutdown via io_uring multi-arm POLL_ADD on SHUTDOWN_BROADCAST_READ_FD.` The old `NOT cascade-aware (Stone B)` lie is gone. Resolved.
> Finding 3 — `BROAD_TOKEN` mumble: All three sites use `BROADCAST_TOKEN`; no `BROAD_TOKEN` anywhere in the file. Resolved.
> New findings scan: No new L1 or L2 issues introduced. Names honest. Structure mirrors intent. The cascade contract section of the module doc is fully truthful.
> GAZE CLEAN."

## Verdict

**STONE B IMPECCABLE — all 5 wards clean on re-pass.**

- gaze: code speaks; doc claims match implementation (cascade-WIRED honestly named); `BROADCAST_TOKEN` speaks where `BROAD_TOKEN` mumbled
- forge: types enforce contracts (private PollOutcome ADT + private helper); SAFETY comments honest at the new unsafe block; constants tightly scoped
- reap: zero dead thoughts; both enum variants constructed + matched; both user_data tokens constructed + consumed
- sever: zero braided concerns; PollOutcome + wait_for_data_or_cascade + Receiver::recv each represent one concern; bootstrap fallback is a clean top-of-loop guard
- temper: SHUTDOWN_BROADCAST_READ_FD loaded once outside loop; CQE drain bounded by ring capacity; only new per-iter cost is a free i32 compare; known Stone E deferrals acknowledged

The kernel-impeccability protocol's per-stone trust gate fires GREEN: BRIEF scorecard Mode A (37/37 satisfied per sonnet's SCORE; zero honest-delta) + ward pass CLEAN (all 5 wards green on re-pass).

After Stone B, the process tier matches Slice 2's thread-tier cascade discipline: blocked recvs cannot hang past substrate shutdown. The deadlock-class defect the cascade contract names is structurally closed.

Stone B ready to commit. Stone C (`HolonRepresentable` serialization layer — generic `Sender<T>` / `Receiver<T>` with HolonAST ↔ EDN bytes via wat-edn) is the next stepping stone in Slice 3.

## Cross-references

- BRIEF-214-SLICE-3B-CASCADE-AWARE-MULTI-ARM.md — work order
- EXPECTATIONS-214-SLICE-3B-CASCADE-AWARE-MULTI-ARM.md — 37-row scorecard + 8 risk pre-emption
- SCORE-214-SLICE-3B-CASCADE-AWARE-MULTI-ARM.md — sonnet's Mode A report
- WARD-PASS-3A-IO-URING-BYTES.md — Stone A round-trip; precedent
- WARD-PASS-2-THREAD-TIER.md — Slice 2 round-trip; 5-ward protocol established
- INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass" — protocol doctrine
- `src/typed_channel.rs:329-368` — substrate's existing libc::poll cascade discipline (Stone B mirrors event-mask + tiebreak)
- `src/runtime.rs:201` — `SHUTDOWN_BROADCAST_READ_FD` definition
- `feedback_never_deadlock` — cascade-aware recv is load-bearing for "deadlocks are illegal"
- `feedback_inscription_immutable` — doc-lies finding category respects "active claims about current state must be honest"
