# SCORE — Stone 251.2a — value/ home + EncodingCtx + frame lift

**Verdict: PASS (green lift), with one tracked nit folded into 251.2b.** Scored against an
INDEPENDENT orchestrator re-run, not sonnet's say-so.

## Scorecard (orchestrator re-run)

| # | What | Result |
|---|---|---|
| 1 | `ls src/value/` | ✓ `encoding_ctx.rs frame.rs mod.rs` |
| 2 | EncodingCtx def gone from runtime.rs | ✓ grep = 0 |
| 3 | frame cluster gone from runtime.rs | ✓ (FrameInfo/FrameGuard/CALL_STACK/replace_top_frame/snapshot_call_stack all absent) |
| 4 | present in new home | ✓ encoding_ctx.rs:1, frame.rs:3 |
| 5 | **lib tests IDENTICAL** | ✓ **923 passed / 0 failed / 1 ignored** (== baseline) |
| 6 | clippy in-home | ✓ nothing citing src/value/ |
| 7 | external API intact | ✓ `pub use value::EncodingCtx;` in lib.rs |

Files: `src/value/{mod,encoding_ctx,frame}.rs` created; `runtime.rs` + `lib.rs` + `freeze.rs` +
`panic_hook.rs` + `assertion.rs` modified (import repoints). vm_registry/runtime_error_edn/thread_io
needed NO change (doc/variant refs only, not type imports — verified).

## Finding (orchestrator caught; sonnet's green report glossed it)

`runtime.rs` carries TWO **dead `pub use` re-export shims**: `pub use crate::value::EncodingCtx;`
(1583) and `pub use crate::value::{FrameInfo, snapshot_call_stack};` (20033). runtime.rs needs these
types internally (EncodingCtx ×21, FrameInfo ×4, snapshot_call_stack ×7) so a `use` is required —
but the `pub` re-exports them on the public `crate::runtime::*` path (`pub mod runtime`), and
**NO consumer reaches them via that old path** (verified across src/ + tests/ + examples/ + benches/).
Dead public surface; the `value/` home is the sole owner. → drop `pub` (plain `use`).

Not fixed in 251.2a (green lift banked; a 2-char polish isn't worth a fresh agent spawn, and
SendMessage to the warm agent is unavailable). **Folded into 251.2b's brief.**

## ★ Migration principle (inscribe for every lift stone)

**Re-export from the monolith IFF consumers still reach the moved name via the old `crate::runtime::`
path (zero-churn for real consumers); otherwise plain `use` (the home owns it, nothing re-exports).**
- 251.2a: EncodingCtx/FrameInfo/snapshot_call_stack = ZERO old-path consumers → plain `use`.
- 251.2b WILL differ: `RuntimeError` has ~567 consumers — repointing all at once is infeasible, so
  a `pub use crate::value::RuntimeError;` re-export in runtime.rs IS legitimately load-bearing
  (zero-churn) during the transition, pruned when the consumers migrate / the eval loop lifts.
The discriminant is consumer-count-via-old-path, decided per type, not a blanket rule.

## Next: 251.2b

signal.rs (EvalSignal/EvalBreak/RuntimeError/RuntimeErrorKind) + observe.rs (TrackedValue/
ValueSnapshot/Provenance + render_value, runtime.rs:18628). Brief MUST: (a) drop the 2 dead pubs
from 251.2a; (b) apply the re-export-iff-consumers principle (RuntimeError keeps its re-export —
567 consumers; resolve the render_value entanglement). Baseline to hold: 923/0/1.
