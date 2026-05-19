# Arc 214 Slice 3 Stone D2 — WARD PASS

Per the kernel-impeccability protocol (INTERSTITIAL § 2026-05-19): per-stone trust gate = BRIEF scorecard verification + ward pass before commit. This doc captures the ward round-trip for Stone D2 (Select<'a, T> cascade-aware N+1-arm fan-in).

5 wards (gaze + forge + reap + sever + temper) — established protocol from Slice 2 + Stones A+B+C+D1.

## Round 1 — initial ward pass (after sonnet SCORE Mode A)

Targets:
- `src/comms/process.rs` — module-level doc updated; imports extended (ReceiverIndex + SelectOutcome); Select<'a, T> struct + new + recv + select + Default impl appended
- `tests/comms/process.rs` — module doc extended; imports add Select + ReceiverIndex + SelectOutcome; 2 new probe_slice3d2_* tests appended

Sonnet's report: Mode A 40/40 with ZERO honest-delta. All 10 EXPECTATIONS risks were CLEAN. BRIEF skeleton compiled and ran correctly first pass; no adaptation required.

### gaze — 1 L2 finding

| Site | Level | Observation |
|---|---|---|
| process.rs (5 sites) | L2 | Synthetic `SelectOutcome::Recv { index: ReceiverIndex(0), result: Err(RecvError) }` returns for io_uring substrate failures appeared at 5 sites in Select::select. ONE site had the WHY comment ("Index is arbitrary because no arm actually fired — the SUBSTRATE failed"); 4 OTHER sites had the same construction with NO comment. Reader hitting any uncommented site cannot tell if arm 0 is meaningfully chosen or if it's an honest signal of substrate failure |

Gaze's suggested fix: "Add a one-line `// ReceiverIndex(0): synthetic sentinel — no arm fired; substrate failure` at each site, OR extract a named helper that carries the explanation once."

### forge — CLEAN (with 1 out-of-scope observation)

> "FORGE CLEAN — types and composition hold; SAFETY comments honest. The two unsafe blocks carry correct SAFETY annotations: POLL_ADD names fd-ownership-elsewhere ('a-bounded Receiver + substrate worker), and Read names buf-outlives-submit_and_wait. Token arithmetic (BROADCAST_TOKEN=0, data arms 1..=N, arm_idx = token-1) is consistent throughout. CQE drain correctly exhausts all ready completions before applying broadcast-wins-ties. Bail-out pattern correctly distinguishes substrate-level failures (ReceiverIndex(0), no arm fired) from arm-specific Read failures (ReceiverIndex(arm_idx), arm is known). Default delegates to new() cleanly."

Out-of-scope observation (noted for future, NOT a Stone D2 finding): `select()` on an empty Select with no broadcast_fd registered will block forever in submit_and_wait(1). The type system permits the call. A future arc may add a `debug_assert!(!self.receivers.is_empty())` guard or constrain at construction. Stone D2 scope is bounded; the observation is for future cleanup.

### reap — CLEAN

> "Select::new + Select::recv + Select::select all have probe consumers. impl Default for Select reachable (downstream consumers per DESIGN.md — Stone E + kernel dispatcher). New imports (ReceiverIndex, SelectOutcome) consumed in both process.rs and test file. PhantomData<T> initialized in Select::new. BROADCAST_TOKEN constructed AND consumed (push + match). Stones A-D1 all still alive (31/31 prior tests pass). No TODO/FIXME/unimplemented! markers. No extra drift beyond BRIEF (no extra helpers, no extra trait derives)."

### sever — CLEAN

> "Lifetime tracking vs data buffering: Select holds only Vec<&'a Receiver<T>>; each Receiver owns its own accumulator. Registration (new/recv) vs wait (select) are entirely separate. Fast-path vs io_uring poll: lines 658-666 exit via return if buffered frame found; 671+ slow-path only when no buffered frame. Poll ring vs Read ring: two distinct IoUring instances; structurally isolated. Bail-out paths: substrate failures and arm-specific failures kept distinct via ReceiverIndex(0) vs ReceiverIndex(arm_idx). BROADCAST_TOKEN scoping: independent locals; no collision with Stone B's BROADCAST_TOKEN. Stones A-D1 ALL unchanged."

### temper — CLEAN

> "Two per-call IoUring per select() iteration (poll ring + read ring) — identical shape to Stone B's recv; both Stone E deferral. broadcast_fd atomic load hoisted ABOVE the loop (line 668-669) — one load per select() call, not per iteration. arm_count + ring_capacity recomputed each iteration but O(1) (Vec::len() + bit-manipulation). Fast-path scan O(N) necessary for any-ready check. CQE drain bounded by N+1. No allocations in hot path beyond per-call IoUring + stack buf. No unintentional waste."

## Orchestrator design decisions (judgment calls)

**Decision 1: GAZE L2 finding — extract `substrate_failure_outcome` helper vs add per-site comments** — EXTRACT HELPER. Four-questions on the choice:

- Per-site comments: Mixed — repeats same comment 4 times; duplicates explanation; reading select() body hits 4 identical comments.
- Helper extraction: YES on all four — DRY; one place for the WHY; call sites name intent via `substrate_failure_outcome()`; less code overall.

YES YES YES YES on helper. Apply.

**Decision 2: Apply helper at ALL 5 substrate-failure sites (including the originally-commented one)** — YES. The helper's doc carries the canonical WHY; leaving the first site with inline code + helper for the others would be inconsistent. Unify all 5.

**Decision 3: DO NOT apply helper at the 5 arm-specific sites (ReceiverIndex(arm_idx))** — Those are arm-specific Read-step failures where arm_idx is meaningful context. Keep inline; not the same shape.

**Decision 4: Forge's empty-Select observation** — NOTE FOR FUTURE, not a Stone D2 finding. The behavior is out of Stone D2 scope (Stone D2 is "the fan-in works"; empty Select is a degenerate caller-error case that warrants a guard in a future arc).

## Fix pass — orchestrator-direct (1 helper + 5 site rewrites)

| # | Fix | File:line |
|---|---|---|
| 1 | Add `fn substrate_failure_outcome<T>() -> SelectOutcome<T>` private helper above the `// ─── Select ───` section divider, with doc-comment naming substrate-failure semantics + ReceiverIndex(0)-is-arbitrary-sentinel + arm-specific-distinction | process.rs:~592-611 |
| 2 | Replace IoUring::new poll-ring failure inline construction with `return substrate_failure_outcome();` (also drop the now-stale 4-line WHY comment that was inline) | process.rs:~681 |
| 3 | Replace broadcast POLL_ADD push failure inline construction with helper call | process.rs:~706 |
| 4 | Replace data POLL_ADD push failure inline construction with helper call | process.rs:~723 |
| 5 | Replace submit_and_wait failure inline construction with helper call | process.rs:~737 |
| 6 | Replace CQE-result-<-0-drain inline construction with helper call | process.rs:~749 |

Mechanical verification post-fix:
- `cargo test --release --test comms` 31/31 PASS (helper extraction is semantically equivalent; no test logic impact)

## Round 2 — gaze re-pass (only the one ward had findings)

### gaze re-pass — CLEAN

> "Helper present with honest WHY doc-comment: YES. Lines 593-611: substrate_failure_outcome<T>() with full WHY (arbitrary sentinel, no user arm fired, substrate failed before any arm could complete, distinction from arm-specific failures). One place, one explanation. All substrate-failure sites use the helper. All 5 arm-specific sites (using ReceiverIndex(arm_idx)) correctly retain inline construction — those carry meaningful context. No new mumbles. GAZE CLEAN."

## Verdict

**STONE D2 IMPECCABLE — all 5 wards clean on re-pass.**

- gaze: code speaks; helper extraction eliminates 5 duplicated boilerplate sites; WHY lives once in helper doc; call sites name intent
- forge: types enforce contracts; SAFETY comments honest at both new unsafe blocks; bail-out pattern correctly distinguishes substrate-level vs arm-specific failures
- reap: zero dead thoughts; all Stone D2 additions have probe consumers; Stones A-D1 all still alive
- sever: zero braided concerns; Stone D2 is purely additive; substrate vs arm-specific failure paths cleanly separated; helper extraction is sever-positive (shorter select() body; named intent)
- temper: known deferrals acknowledged; atomic load hoisted; ring sizing O(1); no unintentional waste

The kernel-impeccability protocol's per-stone trust gate fires GREEN: BRIEF scorecard Mode A (40/40 satisfied per sonnet's SCORE; ZERO honest-delta — first-pass clean) + ward pass CLEAN (5/5 wards green on re-pass; 1 doc-cascade L2 fixed via helper extraction).

After Stone D2: process tier API surface matches thread tier completely. `comms::process::*` and `comms::thread::*` are structurally indistinguishable to consumers. The load-bearing "Thread<I,O> and Process<I,O> identical surface" claim from arc 214's DESIGN is realized at the comms layer. Only Stone E (persistent IoUring + config tunable) remains in Slice 3.

## Out-of-scope observation (inscribed for future)

Forge noted: `select()` on an empty Select with no broadcast_fd registered will block forever in submit_and_wait(1). The type system permits the call. Stone D2 does not guard this case because it's a degenerate caller-error pattern and falls outside the stone's stated scope.

Future arc may add:
- `debug_assert!(!self.receivers.is_empty() || broadcast_fd >= 0);` guard in select()
- OR constrain Select at construction (require at least one receiver before select() is callable)

Inscribed here as a pending refinement; not a Stone D2 defect.

## Cross-references

- BRIEF-214-SLICE-3D2-SELECT.md — work order
- EXPECTATIONS-214-SLICE-3D2-SELECT.md — 40-row scorecard
- SCORE-214-SLICE-3D2-SELECT.md — sonnet's Mode A report (zero honest-delta)
- WARD-PASS-3A through 3D1 — prior round-trips
- INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass" — protocol
- `feedback_iterative_complexity` — Stone D split into D1 + D2 per four-questions
