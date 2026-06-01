# EXPECTATIONS — Stone 243.6a — `CheckError` → Pattern A

Independent scorecard. The orchestrator scores against its OWN re-run (FM 9) — the returned SCORE is not trusted; each load-bearing row is re-verified locally.

## Runtime-band prediction

- **Phase A (reshape + carve + cascade):** 60–120 min Mode A. Mostly mechanical (452 in-file sites); the 5 multi-span judgments are the only non-mechanical work.
- **STOP time:** 240 min (2× upper-bound). Wakeup scheduled there.
- Calibration anchor: 243.3 (TypeError, 16 variants) ran multi-round. CheckError is ~2× the variants + ~2× the cascade; band widened accordingly.

## Scorecard (row · command · expected)

| # | Row | Command | Expected |
|---|---|---|---|
| 1 | Probe flips | `cargo test --release --test probe_arc243_stone6_checkerror_pattern_a` | **3 passed / 0 failed** (was 6 compile errors) |
| 2 | Lib green | `cargo test --release --lib -p wat` | all pass / 0 fail (report N; baseline ~895/0/1-banked) |
| 3 | Tests build | `cargo build --release --tests` | clean (0 errors) |
| 4 | Outer struct | `grep -n "pub struct CheckError" src/check/error.rs` | `pub struct CheckError { pub span: Span, pub kind: CheckErrorKind }` |
| 5 | Kind enum | `grep -n "pub enum CheckErrorKind" src/check/error.rs` | present, 33 variants |
| 6 | Kind span-free | inspect `CheckErrorKind` | 28 single-span variants carry NO span; 5 multi-span carry only domain-named secondaries |
| 7 | Carve + re-export | `grep -nE "pub mod error|pub use error::" src/check.rs` | both present; `wat::check::CheckError` path preserved |
| 8 | diagnostic single-path | inspect `diagnostic()` (moved/updated) | reads `self.span`; the N-arm extraction is gone |
| 9 | T3 still clean | `grep -rn "From<.*> for CheckError" src/` | still none introduced |
| 10 | 243.6b runes intact | `grep -n "deferred-stone-243.6" src/check.rs` | `collect_hints` (1844) + walker (1954) runes still present; the `conformare` (90) rune removed or updated to closed |

## Trap-door risks (each maps to a BRIEF STOP)

- **STOP-1** multi-span "most-actionable" ambiguity → sonnet surfaces variant + both candidates rather than guessing.
- **STOP-2** circular import on carve → one-way `use crate::…` edge, else surface.
- **STOP-3** a CheckError construction with NO span at the call site → a real span-discipline gap; sonnet STOPS, orchestrator decides (do NOT paper with `Span::unknown()`).
- **STOP-4** `remedies`/non-span field drop → preserve on the kind variant.

## Phase B — vigilia REMARKABLE (orchestrator-run, post-strike)

After Phase A lands green (probe 3/0 + lib green + tests build), the orchestrator casts a **live 8-spell vigilia** on `src/check/error.rs` at the REMARKABLE bar (the namespaced-home gate, `feedback_namespaced_home_vigilia_gate`): cast → read verdicts next turn → brief sonnet for divergent findings → re-cast divergent spells → loop to **L1+L2=0** → hashless `vigilatum` stamp → ONE atomic commit (reshape + carve + stamp together). Expect R1 findings (homes-walk pattern); converge over 2–3 rounds.

## Honest-delta watch

- The 452 in-file cascade may surface a variant whose `Display` message makes the most-actionable span genuinely ambiguous — that's a real finding, not a delta to wave past.
- Line-count: expect net ~+50–150 (the outer struct + split Display add a little; the carve is move-not-add).
