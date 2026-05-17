# EXPECTATIONS — Arc 206 Slice 3

## Mode prediction

**Mode A — verification + paperwork ships clean (most likely, ~70%).** Orchestrator's edits verified independently; existing EDN roundtrip coverage is sufficient; INSCRIPTION-SLICE-3 + DESIGN update + USER-GUIDE update + 058 row ship; both commits pushed. ~45 min wall-clock.

**Mode B — orchestrator edit needs correction (~20%).** Verification gate surfaces one or more wrong orchestrator edits (e.g., I removed `pub mod shim` but left a stray `shim::*` reference somewhere; or the `uuid = "1"` Cargo dep needs a specific feature flag I missed). Sonnet corrects + re-verifies. Adds ~15-20 min.

**Mode C — EDN roundtrip gap exists (~10%).** Audit shows existing coverage doesn't prove "wat-level mint via `:wat::core::uuid::v4` + EDN roundtrip end-to-end." Sonnet writes one small wat-level test (likely in `tests/wat_arc206_uuid_edn_roundtrip.rs`). Adds ~15 min.

**Mode D-time-violation — anything past 75 min.** Surface as Mode B-time-violation in SCORE; orchestrator decides whether to kill + re-brief or let finish.

## Workspace baseline expected

Pre-existing 4 failures (must persist; slice 3 introduces none):

1. `lifeline_pipe_zero_orphans_across_100_trials` (FD-multiplex; flaky per arc 170 inscription history; may toggle)
2. `deftest_wat_tests_tmp_totally_bogus` (intentional should-panic canary)
3. `t6_spawn_process_factory_with_capture_round_trips` (arc 170 Stone D2 honest delta)
4. `startup_error_bubbles_up_as_exit_3` (wat-cli pre-existing)

Acceptable post-slice-3: any subset of these 4 (lifeline may flake green). Unacceptable: any failure NOT in this list.

## EDN roundtrip prediction

Highly likely (~80%) the existing wat-edn-side coverage is sufficient and SCORE row D cites the existing tests:

- `crates/wat-edn/tests/spec_conformance.rs::uuid_canonicalized` proves `#uuid "..."` → `Value::Uuid` → canonical String
- `crates/wat-edn/src/json.rs::inst_and_uuid` proves JSON envelope roundtrip
- `crates/wat-edn/tests/uuid_v4_mint.rs` proves mint via `new_uuid_v4()` + roundtrip

If sonnet's audit finds a gap — specifically that no test drives the FULL chain "wat-level mint via `:wat::core::uuid::v4` (NEW substrate path) → wat-edn write → wat-edn read → assert equality" — adding one small wat-level test is the affirmative fix. Sonnet's call based on what the audit shows.

## Verification gate likely outcomes

- Check 1 (git status): 5 files modified/deleted as listed in BRIEF → PASS
- Check 2 (workspace baseline stash + re-test): 4 failures stashed-baseline; ≤4 failures with-edits → PASS  
- Check 3 (telemetry crate green): 36/36 → PASS
- Check 4 (no `:rust::telemetry::uuid::v4` inside telemetry crate): zero hits → PASS
- Check 5 (no `wat_edn::new_uuid_v4` inside telemetry crate): zero hits → PASS

If any check fails, that's data — surface it.

## Out-of-scope findings (surface, don't act)

These are likely to surface during sonnet's verification grep but are out of slice 3's scope; surface as honest deltas:

- Any `wat_edn::new_uuid_v4` caller outside `crates/wat-telemetry/` (e.g., lab-side, holon-rs-side) — telemetry-only slice
- Any UUID-related test in another crate that could benefit from migration — out of slice 3
- Any other `:rust::telemetry::*` shim retirement opportunity — separate concern

Per `feedback_no_known_defect_left_unfixed`: if sonnet finds a same-class defect in scope sonnet CAN fix in <5 min without bloating the slice, fix it; otherwise surface for a follow-up arc.

## Failure-mode catches

- FM 9 (pre-flight baseline): sonnet's verification gate IS the pre-flight check; runs at slice start
- FM 11 (no deferral language): SCORE row H is the explicit grep gate
- FM 16 (no tool preamble): this BRIEF doesn't mention Bash availability anywhere — trust sonnet to use tools
- `feedback_brief_constraint_contradictions`: HARD constraints don't contradict deliverables; verified inline

## Atomic commit shape

Two commits, both via HEREDOC commit messages, Co-Authored-By trailer:

1. **wat-rs commit** — slice 3 code edits + INSCRIPTION-SLICE-3.md + DESIGN.md update + USER-GUIDE.md update + SCORE-SLICE-3.md + (optional) new EDN roundtrip test
2. **lab commit** — 058 changelog row append

Both pushed after commit. Branch state at push: `arc-170-gap-j-v5-deadlock-state` (wat-rs) + `main` (lab).

## What "done" means for this slice

All 12 SCORE rows YES. Both commits pushed. INSCRIPTION-SLICE-3 reads cleanly as the honest record of "slice 2 closed the arc prematurely; here's what was missing; here's how it got closed properly; here's where the EDN serialization invariant is proven." Arc 206 reaches honest closure for real this time.

## Calibration record

Orchestrator predicted slice 2 was "ready to go" 2 hours before this BRIEF; was wrong; the duplicate-impl crack surfaced when user reviewed the inscription. Slice 3 is the forward correction. The calibration lesson: the architectural lesson INSCRIPTION-SLICE-2 inscribed ("separate-impl wins over alias-chain") was inverted by the user immediately. Future closure paperwork should run the four questions on any "architectural lesson inscribed" claim before shipping it as a discipline — what the orchestrator framed as "cleaner" was actually "duplicates the dep for no gain."

Sonnet: trust the gate; trust the disk; ship clean.
