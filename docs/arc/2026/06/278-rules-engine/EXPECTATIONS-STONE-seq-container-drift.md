# EXPECTATIONS — seq/collection container parity (scorecard, written before the strike)

## Scorecard
| what | command | expected |
|---|---|---|
| the parity probe greens | `cargo test --release -p wat --test probe_seq_container_parity` | 7 passed / 0 failed |
| lib floor unchanged | `cargo test --release -p wat --lib -- --test-threads=1 \| grep "test result"` | 941 passed / 36 failed |
| rete accumulate differential intact | `cargo test --release -p wat --test probe_arc278_8b_accumulate_native_differential \| grep "test result"` | 5 passed / 0 failed |
| where-fence intact | `cargo test --release -p wat --test probe_arc278_6b_ii_a_where_oracle \| grep "test result"` | its existing count, all pass |
| release builds clean | `cargo build --release` | no errors |

## Runtime prediction
~4 min (the flat native-build time; R11). The change is ~4 match arms + 3 error-message strings.

## Trap-door risks named
1. **WatAST element type.** Must be `TypeExpr::Path(":wat::WatAST")` (grounded: check.rs:4313 etc.). If the
   sonnet writes a different type, the WatAST tests compile-fail or mis-type → STOP-2 fires.
2. **`rest` is inline in the big infer match, not a helper** (check.rs:5301) — the two new arms go there, not in
   `infer_positional_accessor`. Different site from first/second/third.
3. **`conj` lives in `collection/infer.rs`, not `check.rs`** — the List arm goes there (infer.rs:129).
4. **Over-reach.** A correct fix touches ONLY checker arms + error strings. Any runtime edit = mis-diagnosis
   (STOP-1). Any third file = STOP-3.
5. **Error-message drift.** The `_ =>` messages must be updated to name the now-accepted containers, or they
   lie about a wider set (an intueri/honesty miss) — part of "done", weighed in the score.

## Weigh (orchestrator, after my own re-run)
Re-run the probe + the floors MYSELF; read the diff (4 arms + 3 strings, no runtime touch, no extra file);
confirm the error messages now name the full container set. Commit on green.
