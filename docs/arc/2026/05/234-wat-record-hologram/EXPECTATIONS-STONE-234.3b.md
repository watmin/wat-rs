# EXPECTATIONS — Arc 234 Stone 234.3b

Mode A target: 11/11 PASS.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **234.3b probe 6/6 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 3 | 234.3a regression | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 4 | 234.2c regression | `cargo test --release --test probe_arc234_stone2c_accessor_class_safety 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 5 | 234.2b regression | `cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | 234.5 regression | `cargo test --release --test probe_arc234_stone5_holon_auto_dispatch 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | 234.2a regression | `cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 8 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 11 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty |

## Prediction

**Target:** 45–75 min Mode A. **Upper:** 90 min (STOP-3).

Surface: ~80-120 lines runtime (field lookup loop + type check + struct_form rebuild + holon_form rebuild) + ~15 lines check.rs.

Risks:
- **HolonAST Bundle child replacement** — building a new Bundle with one child swapped; verify the Bundle constructor takes Vec<HolonAST>
- **UnknownField error variant** — may exist (arc 169 struct-destructure may have minted it) OR need new variant; if new, the variant addition is in-scope as small extension
- **Keyword key leading-colon strip** — per Stone 234.2a SCORE D5; mechanical

## STOP triggers

1-10 all REJECTION criteria per BRIEF. STOP-6 specifically guards polymorphic alias upgrade, variadic, check-time narrowing.

## SCORE

`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.3b.md` (NEW). 11-row verbatim outputs + implementation surface + cascade depth + time + calibration + trap-door audit + honest deltas.
