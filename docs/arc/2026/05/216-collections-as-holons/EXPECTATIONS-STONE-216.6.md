# EXPECTATIONS — Arc 216 Stone 216.6 — Process-tier cascade validation

Mode A target: 11/11 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | Probe file created | `tests/probe_arc216_stone6_process_collection_roundtrip.rs` mirrors existing `tests/comms/process.rs` pattern; uses `wat::comms::process::pair` + `CommSender`/`CommReceiver` |
| 2 | Probe 1 — HashMap<String, String> | `pair::<HashMap<String, String>>()` round-trip preserves entries |
| 3 | Probe 2 — HashSet<String> | `pair::<HashSet<String>>()` round-trip preserves elements |
| 4 | Probe 3 — Vec<String> | `pair::<Vec<String>>()` round-trip preserves order |
| 5 | Probe 4 — HashMap<String, Vec<String>> | Nested cascade compiles + round-trips |
| 6 | Probe 5 — Vec<HashSet<String>> | Nested cascade compiles + round-trips |
| 7 | Probe 6 — HashMap<String, Vec<HashSet<String>>> | Triple-nested cascade compiles + round-trips |
| 8 | Probe 7 — Empty HashMap | Empty round-trips as empty (length 0 preserved) |
| 9 | Probe 8 — FIFO with collection payloads | Three sends; three recvs; ordering preserved |
| 10 | Probe 9 — Compile-time HolonRepresentable check | `assert_holon_representable::<T>()` compiles for each collection type variant; test body documents the proof |
| 11 | SCORE doc inscribed | `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.6.md` — scorecard + verification summary + elapsed time + any tests flipped/surfaced |

## Independent prediction (calibration record)

**Target runtime:** 45-75 min Mode A
**Upper bound:** 90 min
**Confidence:** high

**Rationale:**
- Verification stone; substrate is settled (216.1/216.2/216.3 + 216.5a-d all shipped)
- Pattern template is on disk: `tests/comms/process.rs` Stone C String round-trip
- Mechanical translation — substitute T = HashMap/Vec/HashSet, mirror the shape, run the test
- Risk: cascade fails at runtime (STOP-1) — would surface a substrate gap not yet known; low probability given all prior 216 stones green
- Risk: existing test asserts the OLD "this fails" behavior (STOP-3) — pre-arc-216 a `Sender<HashMap<...>>` wouldn't compile, so such tests would be compile errors not failing assertions; probably none

**Per `feedback_stone_briefs_cite_prior_score`:** BRIEF cites all three HolonRepresentable stones + the antidote-complete 216.5d for substrate state.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- INSCRIPTION + closure — Stone 216.7
- Any substrate refactor
- Any test in arcs 213 / 214 / 215
- check.rs `validate_sandbox_scope_leak` (arc 140; different concern — scope leakage, not HolonRepresentable capture)

## Honesty deltas accepted

- Probe count adjustment if a probe overlaps with existing Stone C coverage — sonnet picks; documents
- Test surface in `tests/comms/process.rs` vs new file — sonnet picks (new file is cleaner; existing file would conflate Stone C + Stone 216.6)

## Honesty deltas NOT accepted

- **Probe substitution — STOP-2 trigger.**
- **Silent test flip — STOP-3 trigger.**
- **"This fails at runtime; let me skip" — STOP-1 trigger.**
