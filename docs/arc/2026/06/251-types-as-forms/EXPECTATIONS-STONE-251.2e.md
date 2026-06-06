# EXPECTATIONS — Stone 251.2e — Value cluster lift (foundational, last)

Pure lift; uniform re-export. Load-bearing = baseline-identical. HARDEST lift (Value is foundational,
~1000 lines, widest variant-import list).

| # | What | Command | Expected |
|---|---|---|---|
| 1 | new file | `ls src/value/` | + `value.rs` |
| 2 | Value cluster gone from runtime.rs | `grep -nE '^pub enum Value\|^impl Value\|^impl PartialEq for Value\|^pub struct StructValue' src/runtime.rs` | none (defs) |
| 3 | submodule transitional imports flipped | `grep -rn 'use crate::runtime' src/value/` | NONE remain (all crate::value now) |
| 4 | lib builds | `cargo build --release` | clean |
| 5 | **lib tests IDENTICAL** | `cargo test --release --lib -p wat` | **923 / 0 / 1** |
| 6 | corpus IDENTICAL | `./scripts/integration-run.sh` | no new failures |
| 7 | clippy clean in-home | `cargo clippy --release -p wat 2>&1 \| grep 'src/value/'` | nothing |
| 8 | external API intact | `grep -n 'Value' src/lib.rs` | re-exported from `crate::value` |

Runtime prediction: **30–50 min** (the hardest — ~1000 lines, the widest import list of any lift,
the impl Value block with TRANSFORMS markers + possible Display/HolonRep impls to catch).

Trap-doors: (1) the WIDE variant import list (holon/typed_channel/fork/rust_deps/hologram/io/
crossbeam/chrono/uuid) — add what the compiler names. (2) catch ALL Value impls (grep `for Value`),
not just the 4 named. (3) sequence_eq/hash_sequence stay private. (4) the 3 submodule import flips —
value/ becomes self-contained for these types (no more crate::runtime in value/). (5) any impl Value
method with eval coupling → STOP-2 (might belong in the eval home).

Scoring: orchestrator re-runs 2–7. Row 5 (923/0/1) load-bearing. Row 3 (no crate::runtime in value/)
confirms the home is self-contained for its own types. Commit on green + PUSH. THEN the vigilia ward
(separate phase: drive value/ to L1+L2=0, purgare the dead re-export surface, vigilatum stamp).
