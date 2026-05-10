# Arc 170 slice 1d — EXPECTATIONS

## Independent prediction

**Predicted runtime: 90-180 minutes opus.**

Slice 1d extends `walk_free_symbols` in `src/closure_extract.rs`
to handle binder forms slice 1b didn't enumerate. Comparable to
slice 1c (90 min) since:
- Slice 1b's walker scaffold exists; agent extends it
- Match-arm + struct-destructure + tuple-destructure are settled
  AST shapes (arcs 098 + 169 + 168 respectively)
- 162 deftest failures provide the immediate test stream
- Verification is mechanical: re-run failing tests; expect them
  to pass

**Hard cap: 360 minutes.**

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — Investigation | sample 5-10 failing deftests; identify the binder category each one's free-symbol error reveals; categorize | ✓ |
| B — Walker extends to match-arm bindings | `walk_free_symbols` (or appropriate helper) tracks names introduced by `(:wat::core::Some name)` / similar patterns inside `(:wat::core::match scrut (pattern body) ...)` arms | ✓ |
| C — Walker extends to other surfaced binder categories | each additional binder category surfaced by investigation gets walker support; surface list of categories handled | ✓ |
| D — T16+ tests added | `tests/wat_arc170_closure_extraction.rs` gets new test(s) per category fixed; each test extracts a fn body using the binder + verifies the binder name doesn't surface as free symbol | ✓ |
| E — Workspace 0-failed | `cargo test --release --workspace` shows passed: ~2128 failed: 0 (or near-zero with surfaced residual). The 162 deftest failures should drop dramatically | ✓ |
| F — Phase A + B work untouched | git diff shows slice 1d only added to closure_extract.rs + tests; phase A + B file changes unchanged | ✓ |
| G — Slice 1b API unchanged | `extract_closure` signature + `ClosurePackage { prologue, entry_form }` shape unchanged at the API level; only walker internals extended | ✓ |
| H — No commits | dirty tree includes phase A + B + 1d; orchestrator commits atomically | ✓ |
| I — Slice 1, 1B, 1C, 2 SCOREs untouched | immutable per `feedback_inscription_immutable.md` | ✓ |
| J — Zero Mutex usage | no Mutex/RwLock/CondVar introduced (zero-mutex doctrine) | ✓ |
| K — Honest deltas surfaced | per FM 5; no TODOs | ✓ |

## Honest delta categories

- **Diversity of missed binders beyond match-arm** — surface
  each category found
- **Walker design refactor** — if scope-tracking needs structural
  rework, surface before implementing
- **Sub-cases not walker bugs** — surface failures whose root
  cause isn't walker-related; orchestrator decides
- **FM 5 trap** — TODOs verboten

## Calibration row

Actual runtime: ___ minutes (Mode A clean / B partial / C failed).

Binder categories investigated: ___
Binder categories fixed: ___
Tests added: ___ count
Workspace state post-1d: ___ passed ___ failed
Residual failures (non-walker): ___ count + brief

## What's next (orchestrator-side, post-slice-1d)

When slice 1d ships:
1. Verify workspace = 0-failed locally (FM 9)
2. Atomically commit phase A + phase B + slice 1d as ONE commit
   per recovery doc § 7
3. Author SCORE-SLICE-3.md documenting all three phases
4. Slice 4 BRIEF + EXPECTATIONS authored — bandaid retirement
   pair (Process legacy 3 fields + walker bodies + legacy
   dispatch arms; opus + sonnet atomic-commit per slice 4
   discipline)

## SCORE artifact

Slice 1d's work is part of slice 3's atomic-commit bundle. The
SCORE document covering this work is SCORE-SLICE-3.md (which
documents phase A + B + 1d together) — slice 1d does NOT get
its own SCORE doc. The slice numbering preserves chronological
clarity (1d is substrate work that surfaced from slice 3
testing); the SCORE pattern reflects the atomic-commit bundle.
