# EXPECTATIONS — Stone 251.2d — SymbolTable lift

Pure lift; uniform re-export. Load-bearing = baseline-identical.

| # | What | Command | Expected |
|---|---|---|---|
| 1 | new file | `ls src/value/` | + `symbol_table.rs` |
| 2 | SymbolTable gone from runtime.rs | `grep -nE '^pub struct SymbolTable\|^impl SymbolTable' src/runtime.rs` | none (defs) |
| 3 | lib builds | `cargo build --release` | clean |
| 4 | **lib tests IDENTICAL** | `cargo test --release --lib -p wat` | **923 / 0 / 1** |
| 5 | corpus IDENTICAL | `./scripts/integration-run.sh` | no new failures |
| 6 | clippy clean in-home | `cargo clippy --release -p wat 2>&1 \| grep 'src/value/'` | nothing |
| 7 | external API intact | `grep -n 'SymbolTable' src/lib.rs` | re-exported from `crate::value` |

Runtime prediction: **20–35 min** (god-struct with ~260 lines + wide import list; the impl methods
are the finesse — must move whole, no eval coupling).

Trap-doors: (1) the wide one-way import list — many crates; add what the compiler names. (2) the
`impl SymbolTable` closing brace — move the WHOLE impl, not a partial. (3) transitional `EnumValue`
import from runtime. (4) CheckEnv field → `crate::check::CheckEnv` (intra-crate cycle, fine).

Scoring: orchestrator re-runs 2–6 independently. Row 4 (923/0/1) load-bearing. Commit on green + PUSH.
Dead-pub audit informational only (ward sweeps at 251.2e).
