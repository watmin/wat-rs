# EXPECTATIONS — Stone 238.1 — complete `values_equal`

Paired with `BRIEF-STONE-238.1.md`. Orchestrator scores against an INDEPENDENT local re-run.

## Independent runtime prediction

**20-40 min Mode A.** ~5-6 additive arms in one function (`values_equal`), each mirroring an
existing arm (record ≈ Struct; map/set/Instant/Duration = `Some(a==b)` delegation). No cascade
(additive before `_ => None`). Wakeup time-box: **2× upper = 80 min**.

## Scorecard verification (independent re-run)

| # | Row | Verify | Mode-A pass |
|---|-----|--------|-------------|
| 1 | compile clean | `cargo build --release -p wat 2>&1 \| grep "^error"` | 0 |
| 2 | **probe GREEN (LOAD-BEARING)** | `cargo test --release --test probe_arc238_eq_completeness 2>&1 \| grep "test result"` | `8 passed; 0 failed` (was RED — all errored) |
| 3 | **lib baseline (LOAD-BEARING)** | `cargo test --release --lib -p wat 2>&1 \| grep "test result"` | `>= 828 passed; 0 failed` (+ Instant/Duration unit tests → ~830) |
| 4 | record-variant regression | `--test probe_arc237_sC2c_base_record` | 6/6 |
| 5 | defrecord surface regression | `--test probe_arc227_stone2_defrecord` | 35/35 |
| 6 | `values_equal` + co-located tests ONLY | `git status --short` | only `src/runtime.rs` + the probe + SCORE |

**FM-9 applied to the claim:** I independently re-run rows 2 + 3, and confirm the probe MEASURES
structural equality (equal→true, unequal→false, maps/sets order-independent), not just "returns a
bool." I also spot-read the record arm to confirm it compares `class_fqdn` + recurses (type-strict),
not `holon_form` (which base lacks).

## Mode classification

- **Mode A:** all rows green; arms mirror existing shapes; ≤ STOP-3.
- **Mode B:** an existing arm was touched (forbidden); OR an opaque-type arm was added (scope creep);
  OR a `WatAST: PartialEq` was fabricated; OR baseline dropped. Any → re-brief.
- **WatAST honest-delta:** if `WatAST` lacks `PartialEq`, Sonnet skips arm 6 + surfaces it — that's
  Mode A with a noted delta (a follow-up decides whether WatAST should be wat-comparable), NOT a fail.

## Trap-doors (mirror BRIEF STOPs)

1. Modifying an existing arm → REJECT (additive only).
2. Adding an opaque-type arm (fn/handle/channel/ML) → REJECT (out of scope; those stay erroring).
3. Fabricating `WatAST: PartialEq` → REJECT (STOP + surface if missing).
4. Touching `values_compare`/`eval_eq`/anything beyond `values_equal` + co-located tests → REJECT.
5. holon-rs touched → REJECT.

## On green

Atomic commit: `src/runtime.rs` + `tests/probe_arc238_eq_completeness.rs` + `SCORE-STONE-238.1.md`
as ONE commit. Then 238.2 (INSCRIPTION + USER-GUIDE), arc 238 closes, arc 237 resumes at S-C.2d.
