# Arc 214 Slice 2 — WARD PASS

Per the kernel-impeccability protocol (INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass"): per-slice trust gate = BRIEF scorecard verification + ward pass before commit. This doc captures the ward round-trip for Slice 2 (thread tier).

Slice 1 ran 4 wards (gaze + forge + reap + sever). Slice 2 adds **temper** because runtime logic lands here (Select fan-in is a hot path; efficiency-debt becomes measurable).

## Round 1 — initial ward pass (after sonnet SCORE Mode A)

Targets:
- `src/comms/thread.rs` (~280 LOC; Sender/Receiver/Select/factories)
- `tests/probe_comms_thread.rs` (~160 LOC; 10 smoke tests)

5 wards spawned in parallel per `/wards` skill convention (independent agents, single message).

### gaze — 4 L2 findings (naming consistency + test mumble)

| Line | Level | Observation |
|---|---|---|
| thread.rs:249 | L2 | `let oper = self.inner.select();` — `oper` mumbles; `selected_op` says what it is (a `crossbeam_channel::SelectedOperation`) |
| thread.rs (multi) | L2 | `cb_idx` vs `fired_crossbeam_idx` inconsistent naming for the same concept (crossbeam-internal arm index) |
| thread.rs:select() | L2 | `&(_, rx)` anonymous destructure in `iter().find()` — the discarded crossbeam-arm-idx is meaningful, naming it would speak |
| probe:121 | L2 | `let _idx_b = sel.recv(&rx_b);` — discarded index needs a comment explaining why the registration is intentional but the index is unused |

Spark check: positive on module-level cascade doc + the BRIEF's pre-emption discipline.

### forge — 1 L1 + cascade observations

| Line | Level | Observation |
|---|---|---|
| thread.rs (3 sites) | L1 | `SHUTDOWN_RX.get()` at three sites (Receiver::recv, Select::new, Select::select shutdown branch) is an algebraic escape (substrate state reach-out) and must carry `rune:forge(escape)` annotation per `/forge` rune protocol |

Well-forged sections noted: Sender/Receiver newtype shape; CommSender/CommReceiver trait impls; SelectOutcome construction; the `oper.index()` → user-index mapping.

### reap — 1 finding

| Line | Observation |
|---|---|
| thread.rs:139-141 | `Receiver::is_empty()` added beyond BRIEF scope per sonnet's honest-delta declaration; no consumer, not in CommReceiver trait — dead thought |

All other items (Sender::send/close, Receiver::recv/try_recv/len/close, Select::recv/select, pair/bounded factories) confirmed alive with downstream consumers per DESIGN.md.

### sever — CLEAN

No braided concerns. Sender/Receiver/Select properly delineated; cascade contract correctly scoped at module level with method-level references; no inline domain logic in probe; shutdown-arm handling vs user-arm dispatch cleanly partitioned in `Select::select()`.

### temper — 1 hot-path finding + 2 rare-path noted

| Line | Category | Observation |
|---|---|---|
| thread.rs:Select::select() | hot-path | `self.user_arms.iter().enumerate().find(|(_, (cb, _))| *cb == arm_idx)` — O(N) linear scan EVERY select call; for fan-in of N receivers, every fire pays N comparisons. Use `Vec<Option<usize>>` indexed by crossbeam arm idx for O(1) lookup |
| thread.rs:shutdown branch | rare-path | Two `SHUTDOWN_RX.get()` calls in shutdown branch (one for compare, one to consume) — fires once at shutdown only; not hot |
| thread.rs:Select::new() | rare-path | `Vec::new()` with no capacity hint — registration is one-time setup; resize cost amortized; acceptable |

## Orchestrator design decisions (judgment calls)

**Decision 1: temper supersedes gaze variable-name findings** — The O(1) refactor introduces `crossbeam_to_user: Vec<Option<usize>>` indexed by crossbeam arm idx, plus simplified `user_arms: Vec<&'a Receiver<T>>`. The old `(cb_idx, &Receiver<T>)` tuple field is gone. Gaze's three naming findings (oper, cb_idx/fired_crossbeam_idx, `&(_, rx)`) are resolved BY the refactor:
- `oper` → `selected_op` (gaze fix)
- `cb_idx`/`fired_crossbeam_idx` → unified `arm_idx` (single name in single place; temper refactor removed the duplication site)
- `&(_, rx)` destructure → direct `self.user_arms[user_pos]` indexing (no destructure needed)

**Decision 2: reap's `is_empty` finding** — REMOVE. Sonnet's honest-delta declaration was correct self-flagging; no consumer, no trait obligation, no test coverage. Removed.

**Decision 3: forge `rune:forge(escape)` annotations** — ADD to all three SHUTDOWN_RX.get() sites with the cascade contract citation. The escape is intentional and algebraic-leak-free at the substrate level; the rune marks it for future auditors.

**Decision 4: gaze probe:121 `_idx_b` comment** — ADD one-line comment explaining the second-arm registration is intentional but the returned index is unused (the test asserts on `idx_a` firing, not on the unused index).

## Fix pass — orchestrator-direct (4 surgical edits)

Per the new protocol's "orchestrator addresses OR redirects sonnet": for these small mechanical edits + design decisions, orchestrator applied directly (no sonnet round-trip).

| # | Fix | Files |
|---|---|---|
| 1 | Remove `Receiver::is_empty()` (3 lines) per reap | thread.rs:139-141 |
| 2 | Add `rune:forge(escape) — SHUTDOWN_RX is the substrate cascade signal` annotation at 3 SHUTDOWN_RX.get() sites per forge | thread.rs:107, 207, 255 |
| 3 | Refactor Select for O(1) lookup: `crossbeam_to_user: Vec<Option<usize>>` field + `user_arms: Vec<&Receiver<T>>` simplified; rewrite `Select::new()` / `Select::recv()` / `Select::select()` accordingly. Rename `oper` → `selected_op`; unify `cb_idx`/`fired_crossbeam_idx` → `arm_idx` per temper + gaze | thread.rs:Select-* |
| 4 | Add `_idx_b` clarifying comment per gaze | probe:121-122 |

Mechanical verification post-fix:
- `cargo build --release` clean (5 pre-existing dead_code warnings; zero new)
- `cargo test --release --test probe_comms_thread` 10/10 PASS
- `cargo test --release --test probe_comms_foundation` 3/3 PASS unchanged
- `cargo test --release --test probe_channel_primitive` 3/3 PASS unchanged
- `cargo test --release --test probe_pidfd_primitive` 2/2 PASS unchanged

## Round 2 — ward re-pass (5 wards)

5 wards spawned in parallel against the fixed files.

### gaze re-pass — 1 L2 remained (probe comment not on disk)

> "Three of four Round 1 findings are confirmed addressed: `oper` → `selected_op` CLEAN, `cb_idx`/`fired_crossbeam_idx` unified to `arm_idx` CLEAN, anonymous destructure → named `fired_rx` binding CLEAN. One finding remains: probe:121 `_idx_b` has no comment on disk despite orchestrator intent."

The Edit for fix #4 had been recorded in the fix log but did not land on disk. Round 2 caught the gap honestly.

### forge re-pass — CLEAN

> "All three runes present at correct sites with correct wording. The Select refactor holds under both Hickey and Beckman lenses. The `crossbeam_to_user[arm_idx]` lookup is one source of truth; `user_arms[user_pos]` is the receiver lookup; two tables, each with one job. The `.expect()` panics on substrate invariants (not user error). FORGE CLEAN."

### reap re-pass — CLEAN

> "is_empty is gone (zero occurrences in mod.rs/thread.rs/probe/DESIGN.md). Both new Select fields (crossbeam_to_user + user_arms) fully alive — written in `new()` + `recv()`, read in `recv()` + `select()`. No new dead thoughts. The `_idx_b` is a conscious side-effectful registration (populates the lookup table as a second arm) with the return intentionally unused — correct Rust idiom, not dead code."

### sever re-pass — CLEAN

> "Three Select fields each represent one concern: `shutdown_arm` (shutdown identity), `crossbeam_to_user` (index translation), `user_arms` (user-receiver storage). `recv()` grows both atomically (two columns of one record, not two concerns). `select()` partitions cleanly: shutdown branch returns immediately, user-arm dispatch is unreachable when shutdown fired. No braid."

### temper re-pass — CLEAN

> "O(N) scan gone. `self.crossbeam_to_user[arm_idx]` is genuinely O(1) — single Vec index, no hidden iteration. `resize` in `recv()` is registration-path (not select-path) waste; acceptable. The hot path (`select()` body) has zero allocations, zero resizes, one inner.select(), one shutdown compare, one array lookup, one user_arms deref. Rare-path categorizations re-confirmed correct."

## Fix pass 2 — orchestrator-direct (1 surgical edit)

| # | Fix | Files |
|---|---|---|
| 5 | Add `_idx_b` clarifying comment on disk (the missed fix from pass 1) | probe:121-122 |

Mechanical verification post-fix:
- `cargo test --release --test probe_comms_thread` 10/10 PASS (comment-only change is no-op for test logic)

## Round 3 — gaze re-pass (only the one open finding)

### gaze re-pass — CLEAN

> "The comment at line 121 is present and well-worded: 'registered to give Select a second arm; returned index intentionally unused.' This directly answers the Round 2 finding — it explains WHY the registration is made and WHY the index is discarded. The mumble is closed. No new gaze findings. GAZE CLEAN — no Level 1 or Level 2 findings."

## Verdict

**SLICE 2 IMPECCABLE — all 5 wards clean on re-pass.**

- gaze: code speaks; names unified; the only test-side mumble carries WHY
- forge: types enforce contracts; runes annotate intentional escapes at correct sites; refactor holds under both Hickey and Beckman lenses
- reap: zero dead thoughts; honest-delta is_empty removed; new lookup table + arms both alive
- sever: zero braided concerns; shutdown branch and user-arm dispatch hard-partitioned in `Select::select()`
- temper: O(1) Select fan-in dispatch; hot path has zero allocations; rare-path costs correctly categorized

The kernel-impeccability protocol's per-slice trust gate fires GREEN: BRIEF scorecard Mode A (34/34 satisfied per sonnet's SCORE; one beyond-scope is_empty addition self-flagged and then removed) + ward pass CLEAN (all 5 wards green on re-pass).

Slice 2 ready to commit. Slice 3 (process tier — io_uring + cascade-aware multi-arm) is the next stone; mirrors Slice 2's structure with io_uring underneath instead of crossbeam_channel.

## Cross-references

- BRIEF-214-SLICE-2-THREAD-TIER.md — work order
- EXPECTATIONS-214-SLICE-2-THREAD-TIER.md — 34-row scorecard + 6 risk pre-emption
- SCORE-214-SLICE-2-THREAD-TIER.md — sonnet's Mode A report
- WARD-PASS-1-FOUNDATION-PRIMITIVES.md — Slice 1 round-trip; per-slice precedent
- INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass" — protocol doctrine
- `feedback_assertion_demands_evidence` — every ward finding is evidence; act on it
- `feedback_any_defect_catastrophic` — kernel defects intolerable; ward findings = defect candidates
- `feedback_inscription_immutable` — Round 2's `_idx_b` miss stays in this record as honest evidence of fix-pass discipline gap; not edited out
