# Arc 214 — Slice 2 forward-correction — EXPECTATIONS

## Independent prediction

- **Runtime band:** 45-70 min Mode A. Code is tiny (~50 lines net delta across 2 files); paperwork (DESIGN + INTERSTITIAL) is the bulk of the work. Smaller than E-2 because zero downstream callers means zero borrow-scoping or refactor cascade.
- **LOC changed:** ~50 net delta in code (drop 8-line bounded fn, flip 2 lines in pair body, rename 1 test fn, drop 1 test fn ~11 lines, update doc strings); ~150-200 lines added/changed in paperwork (DESIGN section rewrite + new subsection + INTERSTITIAL entry).
- **New files:** 1 (SCORE doc only).
- **Surprises expected:** LOW. The grep evidence pre-spawn shows zero downstream callers of `comms::thread::bounded`. The 9 remaining probe tests audit-verified compatible with bounded(1). The only non-trivial risk is DESIGN.md section editing (multiple cross-references need to stay consistent).

## Honest-delta watch

### Risk 1 — Downstream caller missed in pre-spawn grep

**What:** Pre-spawn grep for `comms::thread::bounded` returned zero callers. If the grep missed a transitive use (e.g., a re-export, a macro expansion, an EDN form, a future-but-staged file), cargo build fails after `bounded` deletion.

**Mitigation:** STOP trigger fires immediately; sonnet reports the file:line; orchestrator investigates whether the caller is honest (then BRIEF widens) or accidentally-staged (then dirty tree gets cleaned). The likelihood is low because Slice 2 was the introducer + only Slice 4 was supposed to consume.

### Risk 2 — Other probe test fails unexpectedly

**What:** Pre-spawn line-by-line audit of the other 9 probe tests verified none does multi-send-without-recv. If the audit missed a subtle interaction (e.g., a test spawns a thread that holds a Sender clone in a way that defeats the rendezvous), capacity-1 deadlocks the test.

**Mitigation:** STOP trigger fires; sonnet reports failing test name + lines; orchestrator runs the four-questions inline against the failing pattern. Most likely fix: the test honestly needs the cap-1 semantic and the failure proves the test was assuming unbounded; either rewrite the test to honor mini-TCP or, if the test is testing a legitimate use case Slice 4 needs, the entire stone gets re-scoped (very unlikely given pre-audit).

### Risk 3 — DESIGN.md edit produces inconsistencies

**What:** DESIGN.md has multiple references to bounded (line 96-97 Rust-side types listing, line 125 asymmetry discussion, line 484 Slice 2 description). All three sites need updating in the same stone or DESIGN reads inconsistently (some say "bounded exists" others say "bounded retired").

**Mitigation:** BRIEF spells out all three update sites explicitly. SCORE row verifies all three updated. ward:struere flags structural inconsistency if any remain.

### Risk 4 — INTERSTITIAL voice drift

**What:** INTERSTITIAL has a developed voice (the "substrate dreams" pattern; the songs/rhythms thread; the convergence-counting structure). A new entry that violates the voice patterns reads as a discontinuity.

**Mitigation:** BRIEF specifies closing line ("the substrate dreams the depth. The substrate dreams 1. So do we.") as the voice anchor. ward:gaze flags voice-drift if entry doesn't fit the surrounding pattern.

### Risk 5 — pair() doc comment over-engineered

**What:** The mini-TCP discipline IS load-bearing; tempting to write a 20-line essay on it in pair()'s doc comment. The discipline lives in the MODULE-LEVEL doc; pair()'s comment should be terse + point at the module-level discipline.

**Mitigation:** BRIEF gives explicit doc-comment text for both module-level AND pair(). Sonnet uses verbatim. ward:gaze flags doc-comment length-creep.

### Risk 6 — bounded(4) test fixtures in unrelated test files get touched

**What:** `tests/wat_arc170_typed_channel_pipes.rs` calls `crossbeam_channel::bounded(4)` directly (NOT `comms::thread::bounded`). Sonnet might pattern-match "bounded(4) is the thing being retired" and try to update those.

**Mitigation:** BRIEF "Out of scope" section explicitly excludes those calls. STOP trigger fires if sonnet attempts.

### Risk 7 — Cargo test invocation

**What:** Per `feedback_brief_cargo_test_invocation`, multi-crate workspace lib unit tests need `-p wat`. The probe at `tests/comms/thread.rs` is a `wat` crate integration test; correct invocation is `cargo test --release --test thread -p wat`.

**Mitigation:** BRIEF Verification section spells the invocation explicitly. If sonnet uses bare `cargo test`, it may run other crates' tests and the output gets noisy; sonnet should re-run with the precise invocation in the SCORE.

### Risk 8 — INTERSTITIAL placement

**What:** Append AFTER the existing compaction breadcrumb (which is the last entry in the file as of compaction). If sonnet inserts in the middle (sorted by date chronologically as if the file ALWAYS sorts), the standing convention's "chronology IS information" + append-only nature gets violated.

**Mitigation:** BRIEF explicit "append AFTER the existing compaction breadcrumb (after line 6417)" — single trailing position; cannot be misplaced.

### Risk 9 — Module-level doc placement in src/comms/thread.rs

**What:** Module-level doc currently has a cascade-contract paragraph + audience section. The new "Mini-TCP at depth 1" section should go AFTER cascade contract, BEFORE audience. If sonnet inserts after audience or at the very top, structure breaks.

**Mitigation:** BRIEF specifies "after the cascade-contract paragraph, before the factories." Sonnet uses Read first to see exact structure; ward:struere flags ordering.

### Risk 10 — DESIGN.md § "Universe-residency + bounded() asymmetry" wholesale rewrite

**What:** The existing section has the universe-residency principle anchored in it. The rewrite renames the section to "Universe-residency + Mini-TCP at depth 1" and removes the asymmetry framing. If sonnet rewrites OVER the universe-residency principle (deleting it), a foundational concept gets lost.

**Mitigation:** BRIEF says "keep the user-direction quote about universe-residency (lines 108-110)" + "keep the two-layer honesty table (lines 112-117)" + "REMOVE #3 (bounded() asymmetry) entirely" — specific. Universe-residency stays; only the bounded-asymmetry framing retires.

### Risk 11 — Per-stone-trust-gate verification path

**What:** SCORE must verify every BRIEF deliverable independently. If sonnet writes SCORE based on its own claims rather than re-running the verification commands, ward:perspicere will flag.

**Mitigation:** SCORE row #N must include the actual verification command output (or a clear statement that sonnet ran it). BRIEF Verification section names exact commands.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `src/comms/thread.rs` module-level doc gains "Mini-TCP at depth 1" section | PASS |
| 2 | `src/comms/thread.rs` `pair<T>` body uses `crossbeam_channel::bounded(1)` | PASS |
| 3 | `src/comms/thread.rs` `pair<T>` doc-comment updated to name mini-TCP + capacity-1 | PASS |
| 4 | `src/comms/thread.rs` `bounded<T>` fn deleted entirely | PASS |
| 5 | `tests/comms/thread.rs` module-header doc updated ("Nine tests"; drops "unbounded + bounded") | PASS |
| 6 | `tests/comms/thread.rs` import line drops `bounded` | PASS |
| 7 | `tests/comms/thread.rs` `probe_slice2_unbounded_round_trip` renamed to `probe_slice2_pair_round_trip` | PASS |
| 8 | `tests/comms/thread.rs` `probe_slice2_bounded_round_trip` deleted entirely | PASS |
| 9 | DESIGN.md Rust-side types listing drops `bounded` line; pair() comment updated | PASS |
| 10 | DESIGN.md § "Universe-residency + bounded() asymmetry" renamed + asymmetry #3 removed + symmetry inscribed | PASS |
| 11 | DESIGN.md Slice 2 description gets forward-correction note (does NOT edit original body) | PASS |
| 12 | DESIGN.md gets new "Slice 2 forward-correction (2026-05-19) — Mini-TCP at depth 1" subsection at end | PASS |
| 13 | INTERSTITIAL gets new entry after line 6417 with mini-TCP origin + four-questions verdict + cross-refs | PASS |
| 14 | INTERSTITIAL entry closes with the voice-anchor line ("the substrate dreams the depth...") | PASS |
| 15 | `cargo build --release` clean | PASS |
| 16 | `cargo test --release --test thread -p wat` shows exactly 9 tests; all pass | PASS |
| 17 | `cargo test --release --workspace --no-fail-fast` shows workspace clean (no regressions outside the retired test) | PASS |
| 18 | `grep -rn "comms::thread::bounded" --include="*.rs"` returns zero matches | PASS |
| 19 | SCORE doc inscribed with verification command output + honest-delta surfaces (if any) | PASS |

**Total rows: 19.** Modes:

- **Mode A:** 19/19 PASS within time budget. Expected outcome.
- **Mode B-spec-gap:** 1-3 rows show honest delta with clear reasoning (e.g., an audit-missed subtle interaction in the other 9 probe tests). Orchestrator fix-passes or redirects sonnet.
- **Mode B-time-violation:** Wakeup fires at 90 min and sonnet hasn't completed. TaskStop; investigate.
- **Mode C:** Stops at a STOP trigger (downstream caller exists; other probe test fails; DESIGN inconsistency surfaces).

## Time-box

- BRIEF + EXPECTATIONS committed: now (this commit)
- Sonnet spawn: orchestrator drafts agent call; predicted runtime 45-70 min Mode A
- ScheduleWakeup at 90 min (2× upper-bound) as failure-to-communicate detector
- Per-stone trust gate: orchestrator verifies SCORE independently + runs 9-ward parallel pass BEFORE commit

## What this stone enables

After this stone closes + ward-passes:

- **Slice 4 (kernel layer) can start** on the shockingly-stable foundation: thread tier + process tier symmetric; one `pair()` per tier; mini-TCP at depth 1 universal; substrate-author cannot pick wrong depth.
- **Universe-residency principle operational at the comms factory layer:** programs across tiers see identical semantics; substrate enforces the discipline structurally.
- **Three "shockingly stable" foundation pivots tallied in arc 214:** (1) Stone E tunable rejection, (2) bounded() process-tier rejection, (3) THIS — bounded(N) thread-tier rejection. The substrate's discipline of refusing-options-tangle holds.
- **The dragon of "wrong default + asymmetric surface" dies.** Same shape as the dragon of misconfigurations from Slice 3.

## Cross-references

- BRIEF-214-SLICE-2-FORWARD-CORRECTION-MINI-TCP.md — the work itself
- DESIGN.md § "Stone E forward-correction (2026-05-19)" — parallel structure (this stone follows the same pattern at a different layer)
- INTERSTITIAL § "2026-05-19 — Convergence #13" — sibling discipline at the resource-management layer
- INTERSTITIAL § "2026-05-19 — Universe-residency principle" — the principle this discipline operationalizes
- INTERSTITIAL § "2026-05-19 — Kernel impeccability via ward pass (NEW PROTOCOL)" — the per-stone trust gate this stone operates under
- `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels" (line 252-415) — the substrate-wide articulation that predates this stone
- arc 119 — the in-substrate naming of mini-TCP
- Trading-lab pre-wat-rs origin — the empirical convergence on depth-1

*The substrate dreams the depth. The substrate dreams 1. So do we.*
