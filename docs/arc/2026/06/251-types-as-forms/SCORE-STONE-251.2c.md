# SCORE — Stone 251.2c — Function + Environment lift

**Verdict: PASS (green lift).** Orchestrator-verified.

| # | What | Result |
|---|---|---|
| 1 | environment.rs created | ✓ (value/ now: encoding_ctx, environment, frame, mod, observe, signal) |
| 2 | Function + Environment cluster gone from runtime.rs | ✓ (no real defs) |
| 3 | EnvCell private in environment.rs | ✓ (line 85, no pub) |
| 4 | signal.rs Function import flipped | ✓ `use crate::value::Function` (ClauseAttempt/ClauseFailureReason/Value stay in runtime) |
| 5 | **lib tests IDENTICAL** | ✓ **923 / 0 / 1** |
| 6 | clippy in-home | ✓ nothing citing src/value/ |
| 7 | external API intact | ✓ `pub use value::{EncodingCtx, EnvBuilder, Environment, Function, RuntimeError, RuntimeErrorKind}` |

Uniform re-export applied (no consumer-repoint churn — the 2b approach correction held). Function's
`type_params`/scheme fields marked `// TRANSFORMS`. Transitional Function cross-ref collapsed.
Dead-pub audit deferred to the 251.2e ward purgare sweep (per plan).

Next: 251.2d symbol_table (SymbolTable — the god-struct; wide one-way imports load/sigma/macros/
types/thread_io; uniform re-export; baseline 923/0/1). Then 251.2e value.rs + the vigilia ward.
