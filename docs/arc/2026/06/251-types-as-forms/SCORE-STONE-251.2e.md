# SCORE — Stone 251.2e — Value cluster lift (foundational, last)

**Verdict: PASS (green lift). The value/ home is fully populated.** Orchestrator-verified.

| # | What | Result |
|---|---|---|
| 1 | value.rs created | ✓ value/ now: encoding_ctx, environment, frame, mod, observe, signal, symbol_table, value |
| 2 | Value cluster gone from runtime.rs | ✓ (no defs; ~1000-line block removed) |
| 3 | value/ self-contained (own type defs) | ✓ `grep -rn 'use crate::runtime' src/value/` empty |
| 4 | **lib tests IDENTICAL** | ✓ **923 / 0 / 1** |
| 5 | clippy in-home | ✓ nothing citing src/value/ |
| 6 | external API intact | ✓ Value/StructValue re-exported from crate::value |
| 7 | runtime.rs shrinking | ✓ 31,328 → **28,785** (~2,543 lifted across 251.2a–e) |

**Off-brief change (verified legitimate):** sonnet raised `extract_classifier`→`pub(crate)` —
value.rs:1004 `declared_type_name` calls `crate::runtime::extract_classifier` (a fully-qualified
call, not a `use`). Legitimate move-consequence (the method moved + calls a runtime fn), analogous
to 251.2a's FrameGuard→pub(crate). **Transitional coupling noted:** value/value.rs → runtime::
extract_classifier (the algebra layer); resolves when `algebra/` lifts (extract_classifier → algebra/).

**TRANSFORMS markers** on type_name/declared_type_name (clojure-ination keyword type-strings).
Uniform re-export held.

## ★ 251.2 sub-stone LIFTS COMPLETE (a–e). The runtime data model lives in src/value/.

Next: **the vigilia WARD** — cast the inward guard on src/value/, drive to L1+L2=0 (incl. the
purgare sweep of the accumulated dead `pub use` re-export surface), earn the vigilatum stamp. Then
the keystone is sealed and 251.2 closes.
