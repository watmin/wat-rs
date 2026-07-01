# EXPECTATIONS — 296 S7: `EnsureFnInvalid.reason` → enum

Independent scorecard, written BEFORE the strike so the result cannot move the goalposts. The orchestrator re-runs each
row itself and weighs the emitted wire EDN, not the sonnet's report.

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the enum exists + is derived | `grep -n "enum EnsureFnInvalidReason" src/check/error.rs` ; `grep -n "wat_macros::ToEdn" src/check/error.rs` | the enum + `#[derive(…ToEdn)]` on it |
| 2 | the field is retyped | `grep -n "reason: EnsureFnInvalidReason" src/check/error.rs` | field is the enum, not `String` |
| 3 | no `format!` reason survives | `grep -n "reason: format!" src/check.rs` | 0 |
| 4 | the RED probe is un-ignored + GREEN | `cargo nextest run --release -E 'test(ensure_fn_invalid_reason_is_structural_not_prose)'` | 1 passed (no longer `#[ignore]`'d) |
| 5 | `:reason` is structural on the wire | orchestrator captures the probe's emitted `:reason` EDN | `#wat.kernel/ArgTypeMismatch {:arg-type "…" :clause-return-type "…"}` — a Tagged map, NOT a String |
| 6 | Display byte-identical | check-family Display/snapshot tests | GREEN, untouched (human sentence unchanged) |
| 7 | full gate | `cargo nextest run --release` | 0 failed (floor holds; +0 or the probe now counted GREEN) |
| 8 | clean build | `cargo build --release` | clean; warning delta ~0 vs HEAD |

## Independent prediction
- **Runtime:** 8–15 min (small: one enum + Display, 7 mechanical site edits, one probe un-ignore).
- **Trap-doors:**
  - A hidden 8th `EnsureFnInvalid` site or a golden test asserting the old `:reason` String → the sonnet must surface
    it (STOP-1), not paper over it. The orchestrator greps `EnsureFnInvalid` + `:reason` independently at weigh.
  - The `Display` sentence drifting by a character (a backtick, the em-dash) → a Display test flips. Byte-for-byte.
- **The weigh:** the orchestrator re-runs rows 3/4/5/7 itself and reads the emitted `:reason` EDN by hand
  (capture-don't-trust). A bent probe = auto-reject (PROBATIO FLEXA MENTITVR).
