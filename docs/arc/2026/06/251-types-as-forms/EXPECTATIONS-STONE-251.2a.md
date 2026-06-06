# EXPECTATIONS — Stone 251.2a — value/ home + EncodingCtx + frame lift

Written BEFORE the strike, so the score can't move the goalposts. This is a PURE LIFT —
the load-bearing expectation is **baseline-identical**.

## Scorecard

| # | What | Command | Expected |
|---|---|---|---|
| 1 | The home exists with 3 files | `ls src/value/` | `mod.rs  encoding_ctx.rs  frame.rs` |
| 2 | EncodingCtx GONE from runtime.rs | `grep -c 'struct EncodingCtx\|impl EncodingCtx' src/runtime.rs` | `0` |
| 3 | frame cluster GONE from runtime.rs | `grep -c 'struct FrameInfo\|struct FrameGuard\|fn snapshot_call_stack' src/runtime.rs` | `0` |
| 4 | lib builds | `cargo build --release` | clean (warnings ok; no errors) |
| 5 | lib tests IDENTICAL | `cargo test --release --lib -p wat` | **923 passed / 0 failed / 1 ignored** |
| 6 | corpus IDENTICAL | `./scripts/integration-run.sh` | same as baseline (no new failures) |
| 7 | clippy clean in-home | `cargo clippy --release -p wat 2>&1 \| grep 'src/value/'` | no warnings citing src/value/ |
| 8 | external API intact | `grep -n 'EncodingCtx' src/lib.rs` | re-exported from `crate::value` |

## Independent prediction

- Runtime: **15–25 min** (small mechanical lift; ~6 consumer files; the compiler names every site).
- Mode A (clean one-shot) likely — this is the EASY frontier by design.

## Trap-doors (named risks)

1. **Visibility**: `FrameGuard` + `replace_top_frame` are private in runtime.rs (eval-loop-internal);
   after the move the eval loop calls them cross-module → must become `pub(crate)`. If sonnet leaves
   them private, `cargo build` fails with a privacy error (caught by row 4).
2. **The FrameInfo destructure at runtime.rs:22572** (`value_from_frame_info`, Scout 4's died-error
   territory) — it pattern-matches `FrameInfo { callee_path, call_span }`; needs `use crate::value::FrameInfo`
   and the fields stay `pub`. Easy to miss.
3. **lib.rs re-export**: `EncodingCtx` is in a multi-name `pub use runtime::{…}` list; moving only it
   (leaving EnvBuilder/Environment/etc.) requires editing that list carefully, not wholesale.
4. **CALL_STACK thread-local**: stays module-private in frame.rs — verify FrameGuard/replace_top_frame/
   snapshot_call_stack all moved with it (they're its only users).

## Scoring method

The orchestrator re-runs rows 4–7 independently (not sonnet's say-so). Row 5 is the load-bearing
verification: any count != 923/0/1 means behavior moved → the lift is wrong → reject + diagnose.
Row 6 (corpus) confirms no integration-level drift. The score records the before/after baseline +
the repointed-consumer list, then commits on green.
