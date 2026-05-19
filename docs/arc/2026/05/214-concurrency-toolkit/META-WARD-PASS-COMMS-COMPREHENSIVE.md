# Arc 214 — Comprehensive comms sweep (10 distinct measurements)

After Stone D2 shipped, the user invoked a comprehensive measurement sweep
on src/comms/ + tests/comms/. The per-stone trust gate (5 wards: gaze +
forge + reap + sever + temper) had already cleared each stone's
increments; this doc records the CUMULATIVE sweep using the OTHER 5
spells discovered in `wat-rs/.claude/skills/` + `holon-lab-trading/
.claude/skills/`.

## Spells applied (10 total across the arc)

Per-stone (already applied per each of A, B, C, D1, D2):
- **gaze** — naming + WHY-comments
- **forge** — Hickey (values vs places) + Beckman (types enforce contracts)
- **reap** — dead thoughts
- **sever** — concern separation
- **temper** — efficiency-debt

Cumulative sweep (this doc):
- **complectens** (wat-rs) — test composition discipline
- **perspicere** (wat-rs) — deeply-nested types missing nouns
- **vocare** (wat-rs) — tests verify from caller's vantage
- **cleave** (lab) — parallel boundaries disjoint
- **ignorant** (lab) — fresh-reader walk-from-nothing

## Round 1 — cumulative sweep verdicts

| Spell | Verdict | Findings |
|---|---|---|
| PERSPICERE | CLEAN | No deeply-nested types hiding nouns; all type expressions pronounceable in one breath |
| COMPLECTENS | CLEAN with 3 L2 | catalogue-style error-types test; recurring `thread::sleep` pattern; 22-line competes_for_frames body |
| VOCARE | CLEAN | All 31 tests verify from caller's vantage; no implementation-internals reach |
| CLEAVE | CLEAN | thread/process tiers structurally disjoint; mod.rs holds only shared trait surface |
| IGNORANT | CLEAN with 3 rough-paths | "arc 214" lacks one-line summary; SHUTDOWN_RX type opaque; edn_shim shim has no public doc |

**Zero L1 lies. Zero cleave violations. Zero vocare issues. Zero deeply-nested-type issues.** Six soft observations, mostly "would benefit from one more sentence."

## The user audit — sleep doctrine surfaced

During triage of the complectens "recurring `thread::sleep`" finding, the
orchestrator initially proposed extracting a `wait_for_kernel_flush()`
helper. The user pushed back:

> *why do we have any sleeps at all?... what in our system is not lock step?*

This invoked `feedback_lock_step_via_pipe`: **"Sleep is a guess; guesses
race."** The orchestrator's helper extraction was naming the violation,
not fixing it. None of the 5 cumulative-sweep spells had surfaced this
as a substrate-doctrine violation — complectens flagged it as test-style
mumble; the deeper doctrine layer caught it only through the user audit.

Re-analysis showed:
- **libc::write(2) is synchronous** → bytes in pipe buffer when `tx.send()` returns
- **libc::close(2) is synchronous** → pipe state-changes at close-time when `drop(tx)` returns
- **libc::poll(timeout=0) is synchronous** → sees current kernel state immediately
- **io_uring submit_and_wait BLOCKS** on kernel events; POLL_ADD wakes immediately when POLLIN/POLLHUP fires

The 5 sleeps in tests/comms/process.rs were **all unjustified guesses** — they covered no actual race. Removed all 5:

| Test | Old shape | New shape |
|---|---|---|
| `try_recv_disconnected_after_sender_drop` | drop + sleep(20ms) + try_recv | drop + try_recv (close-then-poll lock-step) |
| `try_recv_succeeds_when_data_ready` | send + sleep(20ms) + try_recv | send + try_recv (write-then-poll lock-step) |
| `receiver_clone_competes_for_frames` | send + sleep(20ms) + clone.recv | send + clone.recv (write-then-recv lock-step) |
| `sender_drop_wakes_recv_with_err` | spawn(sleep(50ms) + drop) + recv (race) | drop + recv (close-then-recv lock-step); renamed to `recv_returns_err_after_sender_drop` (honest contract) |
| `select_picks_fired_receiver` | send + sleep(20ms) + select | send + select (submit_and_wait blocks on kernel event) |

All 35 tests pass with zero sleeps. The substrate is **fully lock-step** end-to-end; the previous tests were lying via guesses.

This is failure-engineering at a higher level than the per-stone gate:
**the wards measured what they were calibrated for; the user audit caught
what the wards weren't yet calibrated to find.** The lesson inscribed for
future spell maintenance: each new doctrine (like `feedback_lock_step_via_pipe`)
warrants a corresponding spell or amendment to an existing one.

## Polish applied from the cumulative sweep

In addition to the sleep removal, the soft observations were addressed:

**IGNORANT 1 (arc 214 one-liner):** Added cross-references to
`docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md` in module-level docs
of `mod.rs`, `thread.rs`, `process.rs`. Fresh readers now have a single
sentence pointing at the rationale.

**IGNORANT 2 (SHUTDOWN_RX / SHUTDOWN_BROADCAST_READ_FD type opacity):**
Expanded the cascade contract doc in `mod.rs` to name the actual types
(`OnceLock<crossbeam_channel::Receiver<()>>` for thread tier;
`AtomicI32` for process tier) and the init site (`freeze.rs:233`).

**IGNORANT 3 (edn_shim wire-chain doc):** Added a wire-chain summary at
the top of `process.rs` module-level doc spelling out the full pipeline:
`T → HolonAST → tagged-EDN string → newline-framed bytes → libc::write
→ io_uring Read → bytes → EDN → HolonAST → T`. Names the helper functions
explicitly (`write_holon_ast_tagged` / `read_holon_ast_tagged`).

**COMPLECTENS 1 (error-types catalogue test):** Split
`probe_slice1_error_types_construct_and_distinguish` (1 test, 5 concerns)
into 5 focused tests:
- `probe_slice1_send_error_carries_unsent_value`
- `probe_slice1_recv_error_is_unit_struct`
- `probe_slice1_try_recv_error_variants_are_distinct` (distinctness assertion)
- `probe_slice1_close_error_carries_diagnostic_text`
- `probe_slice1_wire_error_carries_diagnostic_text`

Each test now narrows the failure surface to one error type. The
distinctness assertion (the "AND distinguish" part of the original
catalogue name) gets its own test that proves TryRecvError's variants
are not conflatable.

**COMPLECTENS 2 (recurring sleep helper):** Initially extracted
`wait_for_kernel_flush()` helper; reverted entirely after the user audit
revealed the sleeps themselves were unjustified. Net result: 5 sleeps
removed; no helper needed.

**COMPLECTENS 3 (22-line competes_for_frames body):** Attestation, not
deferral. This test was redesigned in Stone D1 specifically to
deterministically prove the clone property (fresh accumulator + shared
pipe fd via fd-dup) after sonnet observed a 60-second hang in the
original design. The 22-line body is the result of careful failure
engineering; further extraction would HIDE the timing-assumption
explanation currently visible inline. Bounded; documented; leave as-is.

## Test count delta

| Before sweep | After sweep |
|---|---|
| 31 tests (3 foundation + 10 thread + 18 process) | 35 tests (7 foundation + 10 thread + 18 process) |

The 4 new tests are from the foundation split (one error-types catalogue
became 5 focused tests; net +4).

## Cumulative verification

```
cargo test --release --test comms              # 35/35 PASS — zero sleeps
cargo test --release --test probe_channel_primitive  # 3/3 PASS unchanged
cargo test --release --test probe_pidfd_primitive    # 2/2 PASS unchanged
```

## Doctrine cross-references

- `feedback_lock_step_via_pipe` — "Sleep is a guess; guesses race." Lock-step via the wire.
- `feedback_never_deadlock` — cascade-aware recv is load-bearing for "deadlocks are illegal"
- `feedback_no_known_defect_left_unfixed` — when we know how to surface or fix a defect right now, do it now
- `feedback_assertion_demands_evidence` — every assertion attempt is the trigger for "I know I don't know"
- `feedback_failure_engineering` — failure as data; artifacts propagate discipline to fresh agents

## What this sweep proves

src/comms/ + tests/comms/ are remarkable across 10 distinct measurements:

1. **gaze** (per-stone) — code speaks; names match content
2. **forge** (per-stone) — types enforce contracts; honest composition
3. **reap** (per-stone) — zero dead code; every item alive
4. **sever** (per-stone) — concerns properly separated; no braiding
5. **temper** (per-stone) — efficiency-debt acknowledged at appropriate scales
6. **complectens** (this sweep) — tests compose honestly; no monolithic one-shots
7. **perspicere** (this sweep) — type expressions pronounceable in one breath
8. **vocare** (this sweep) — all tests verify from caller's vantage
9. **cleave** (this sweep) — thread/process tiers structurally disjoint
10. **ignorant** (this sweep) — fresh reader can navigate without external help

Plus an 11th measurement surfaced by the user audit:
**lock-step doctrine** — substrate communicates via synchronous primitives
end-to-end; no guesses; no sleeps; tests use the wire.

After this sweep, the comms surface is the most carefully-measured
substrate module in wat-rs. The 10-spell template applies to every future
src/<module>/ + tests/<module>/ pair as they ship.

## Inscription

What this sweep proves about the protocol: the per-stone trust gate
catches what the per-stone wards can measure. The CUMULATIVE sweep catches
inter-stone consistency. The USER AUDIT catches doctrine violations
neither layer is yet calibrated to find. All three layers are necessary;
none is sufficient alone.

The orchestrator's reflex to extract `wait_for_kernel_flush` instead of
removing the sleep is the recorded failure mode. The fix: when proposing
a "name the violation" remediation, ask first whether the violation
should EXIST. Sleep-as-helper is naming a defect, not fixing it.

This is inscribed because the next orchestrator (sonnet, future-me, a
fresh contributor) will hit a similar reflex. The artifact propagates
the discipline.

## Cross-references

- WARD-PASS-1 through 3D2 — per-stone trust gate records
- DESIGN.md — full arc 214 design
- `feedback_lock_step_via_pipe` — the doctrine the user audit surfaced
- `feedback_failure_engineering` — artifacts as teaching
- `feedback_no_known_defect_left_unfixed` — "deferral is a divide by zero"
