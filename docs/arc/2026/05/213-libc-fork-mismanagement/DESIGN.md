# Arc 213 — libc::fork mismanagement under workspace pressure

**Status:** OPEN 2026-05-18 — opened to address `probe_lifeline_pipe_proof`'s pressure-flake whose intermittent nature was documented by arc 211c's audit. Arc 211 closure depends on this arc's resolution per the **tooling-proven-by-use** discipline (see INTERSTITIAL § 2026-05-18 (post-arc-211e)).

**Priority:** BLOCKING arc 211 INSCRIPTION (along with arc 212).

## Origin

`probe_lifeline_pipe_proof` (created in arc 170 Slice D / FD-multiplex Phase work; commit `198c30b`) demonstrates deterministic parent-death detection via lifeline pipe. The test's doc-comment claims:

> *"100/100 trials produce zero orphans regardless of supervisor exit timing."*

In isolation: yes, the test passes 100/100. Verified multiple times this session.

**Under workspace parallel test pressure:** the test flakes. Failure SET rotates between `probe_lifeline_pipe_proof` and `test` umbrella's deftest-hermetic subtests (per arc 211c audit + post-211e workspace runs). Never both simultaneously fail. Never reliably reproduces — pure pressure-flake.

The mechanism uses raw `libc::fork` directly with manual pipe management — NO substrate spawn-process / spawn-thread involvement. The flake suggests OS-level mismanagement: fd-table pressure, scheduling timing, pipe inheritance race under heavy parallel load.

## Scope

**In scope:**
- Investigate the flake's root cause under workspace pressure
- Determine: is it a fixable bug, or a fundamental OS-pressure characteristic of the test mechanism?
- Either:
  - **Fix:** identify + repair the libc::fork-management issue; test passes 100% under workspace pressure
  - **Document:** SCORE inscribes honest assessment ("OS-pressure characteristic; mechanism correct in isolation; expected-intermittent under parallel pressure"); test marked `#[ignore]` or test runner config excludes it from workspace failure counts
- Use arc 211's panic-as-EDN tooling to capture the failure structurally if it surfaces a panic
- If the failure is pure-hang (no panic emitted), document that arc 211a/b's tooling didn't directly help — and inscribe what tooling WOULD have helped (informs future arc work)

**Out of scope:**
- Broader libc::fork patterns elsewhere in substrate (unless investigation reveals shared root cause)
- The lifeline-pipe mechanism itself (proven correct in isolation; not redesigning)
- FD-multiplex Phase 6 paperwork (separate task #305; may interact but not part of this arc)

## Closure conditions

1. Investigation produces honest diagnosis (rooted in actual run-data, not speculation per `feedback_no_speculation`)
2. EITHER:
   - Fix ships AND probe_lifeline_pipe_proof passes under workspace pressure (≥100 trials clean)
   - OR honest "expected-intermittent" assessment ships AND test excluded from workspace failure count
3. SCORE doc inscribes findings (including: did arc 211's tooling assist? what gaps remain?)
4. Arc 211 closure becomes unblocked (other pre-condition: arc 212)

## Cross-references

- Arc 170 FD-multiplex Phase 1A-3 (the lifeline mechanism work that produced this test)
- Arc 170 Phase 1D SCORE (substrate-mechanism probe + leak-zero gate — this test IS the gate)
- Arc 211 SCORE-211C-AUDIT (confirmed pressure-flake nature)
- Arc 211 DESIGN § "Tooling-proven-by-use closure condition" (the blocking relationship)
- Arc 211 INSCRIPTION (pending; awaits this arc)
- INTERSTITIAL § 2026-05-17 "Orphan-process leak investigation" (broader FD-management concerns; shared diagnostic territory)
- INTERSTITIAL § 2026-05-18 (post-arc-211e) "Tooling proven by use — closure-discipline extension"
- `tests/probe_lifeline_pipe_proof.rs` (the test)

## Tooling-proven-by-use principle

This arc serves dual purpose:
1. **Resolve probe_lifeline_pipe_proof's disposition** (substrate correctness OR honest documentation)
2. **PROVE arc 211's tooling enabled this resolution** (substrate-tooling-validation)

Two possible validation paths:
- **If failure surfaces a panic** — arc 211b's structured EDN provides readable diagnostic; arc 211a's ctor ensures the hook is installed. Tooling proves itself directly.
- **If failure is a pure-hang (no panic)** — arc 211's tooling didn't directly help. SCORE inscribes the gap. That inscription IS load-bearing for future tooling arcs (we'd know what arc 211 didn't cover and could open follow-up tooling work).

Either outcome validates the principle: tooling-proven-by-use, not tooling-assumed-working. When arc 213 closes, arc 211 closes (along with arc 212).
