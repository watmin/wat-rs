# EXPECTATIONS — Stone S-C.2c — mint base `Value::wat__Record`

Paired with `BRIEF-STONE-S-C2c.md`. Written BEFORE spawn (FM-9 baseline re-run done: **827/0**
on disk @ HEAD `05818c3f`). The orchestrator scores against an INDEPENDENT local re-run.

## Independent runtime prediction

**30–55 min Mode A.** Smaller decision surface than S-C.2ab (no parse changes, no macro changes,
no name→index re-route — that landed in 2ab). The work is: one variant + 3 base-structural arms
(Eq/Hash/assoc) + an or-pattern cascade the compiler drives + holon-op error arms + 1 co-located
unit test. Wakeup time-box: **2× upper = 110 min** (`ScheduleWakeup` @ ~6600s).

## How each scorecard row is verified (independent re-run)

| # | Row | Verify by | Mode-A pass |
|---|-----|-----------|-------------|
| 1 | variant compiles under `#[wat_value]` | `cargo build --release -p wat 2>&1 \| grep "^error"` | 0 errors |
| 2 | **lib baseline (LOAD-BEARING)** | `cargo test --release --lib -p wat 2>&1 \| grep "test result"` | `827 passed; 0 failed` (+ co-located base→Err unit test ⇒ may read 828) |
| 3 | **external probe (LOAD-BEARING)** | `cargo test --release --test probe_arc237_sC2c_base_record 2>&1 \| grep "test result"` | `6 passed; 0 failed` |
| 4 | S-C.2ab guard unchanged | `--test probe_arc237_sC2ab_field_order` | 5/5 |
| 5 | S-A1 unchanged | `--test probe_arc237_sA1_assignable` | 6/6 |
| 6 | holonic field access unchanged | `--test probe_arc234_stone3c_keyword_accessor` | 6/6 |
| 7 | holonic assoc parity unchanged | `--test probe_arc234_stone3b_record_assoc` | 6/6 |
| 8 | defrecord surface unchanged | `--test probe_arc227_stone2_defrecord` | 35/35 |
| 9 | `src/runtime.rs` + probe ONLY | `git status --short` | nothing outside `src/runtime.rs`, the probe, the SCORE doc |

**Load-bearing rows independently re-run before commit (FM-9 applied to the claim, not the
report):** rows 2 + 3. The probe must MEASURE base's structural identity — I confirm the 6
contracts assert what their names claim (Eq×3 forms + base≠holonic + Hash dedup + type identity),
not adjacent surface.

## Mode classification

- **Mode A (clean):** all 9 rows green; cascade was mechanical (every compiler error was an
  obvious Bucket A/B/C site); ≤ STOP-3.
- **Mode B (brief gap):** a non-obvious compiler error surfaced (→ Sonnet should have STOPPED and
  surfaced it verbatim, per the error-pivot law); OR a cascade site didn't fit A/B/C cleanly
  (signals a missing bucket in the sub-DESIGN — that's MY defect, re-brief). OR baseline dropped
  (additive stone must not drop it).
- **Time-violation:** wakeup fires with Sonnet still running ⇒ `TaskStop` + score Mode-B-time.

## Trap-doors (mirror the sub-DESIGN's REJECTION STOPs)

1. `holon_form: Option` / any flavor flag → REJECT (two variants).
2. on-demand holon projection for base → REJECT (base holon-ops ERROR).
3. base constructor / macro split → REJECT (that is S-C.3).
4. holonic `assoc` parity rebuild disturbed → REJECT.
5. base collapsed into `Value::Struct` → REJECT.
6. holon-rs touched → REJECT (STOP-5).

## On green

Atomic commit: `src/runtime.rs` + `tests/probe_arc237_sC2c_base_record.rs` + `SCORE-STONE-S-C2c.md`
as ONE commit (the compile-RED probe lands WITH the substrate that makes it compile — Seam-2
verdict). Then update `REMAINING-ORDER.md` + cliffnotes "Currently" (S-C.2c SHIPPED; S-C.3 NEXT).
