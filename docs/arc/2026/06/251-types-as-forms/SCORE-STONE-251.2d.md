# SCORE — Stone 251.2d — SymbolTable lift

**Verdict: PASS (green lift).** Orchestrator-verified.

| # | What | Result |
|---|---|---|
| 1 | symbol_table.rs created | ✓ (value/ now: encoding_ctx, environment, frame, mod, observe, signal, symbol_table) |
| 2 | SymbolTable gone from runtime.rs | ✓ (no defs; 274-line block removed) |
| 3 | **lib tests IDENTICAL** | ✓ **923 / 0 / 1** |
| 4 | clippy in-home | ✓ nothing citing src/value/ |
| 5 | external API intact | ✓ `pub use value::SymbolTable` (lib.rs:160) |
| 6 | runtime.rs shrinking | ✓ 31,328 → **29,820** (~1,500 lifted across 251.2a–d) |

Uniform re-export held. keyword-keyed maps marked // TRANSFORMS. Transitional `use crate::runtime::{EnumValue, Value}` (move at 251.2e). Dead-pub audit → 251.2e ward purgare sweep.

Next: 251.2e — value.rs (Value enum + StructValue/EnumValue/SpawnOutcome/ProgramHandleInner/Clause cluster + sequence_eq/hash_sequence; HARD, foundational, LAST) + the vigilia ward (L1+L2=0, purgare re-export sweep, vigilatum stamp). The keystone seals.
