# EXPECTATIONS — Arc 210 Slice 1

## Mode prediction

**Mode A — clean atomic ship (~75%).** Sonnet reads the BRIEF, applies the 4 task changes mechanically: parser at `src/check.rs:7490` (arg-count + new keyword validation), runtime at `src/runtime.rs:2287` (mirror), defmacro at `wat/core.wat:221-227` (positional binder add), tests at `tests/wat_arc198_def_restricted.rs` (~10-12 sites). cargo test green; SCORE 3/3 YES. ~45-60 min wall-clock.

**Mode B — small surprise (~20%).** Likely shapes:
- `src/runtime.rs:2287` arm is more entangled than expected (additional state/registration touchpoints)
- One or two test sites have non-obvious shape (e.g., parser-error-message assertions that need careful updating)
- The defmacro `:AST<wat::core::nil>` typed-binder for keyword-literal needs different typing (e.g., `:AST<wat::core::keyword>`)

Sonnet surfaces, orchestrator decides (most likely: "ship with the surprise; SCORE notes the delta honestly").

**Mode C — STOP-trigger fires (~5%).** Most likely:
- Pre-flight grep finds wat-side consumers DESIGN claimed don't exist (would extend ripple)
- Defmacro can't splice the keyword positional cleanly (would force `& rest` + manual destructure pattern)
- Test diagnostic-message assertions reveal MalformedForm is rendered in multiple places that need coordinated update

Sonnet stops, surfaces, orchestrator decides.

## Strategy-decision prediction

Orchestrator's guess: BRIEF strategy LOCKED (option I — parser accepts 4-arg shape with literal `:restricted-to` validation). Sonnet's audit may reveal:
- Better way to validate `:restricted-to` keyword (e.g., dedicated helper function elsewhere in check.rs)
- Optimal location for the new diagnostic error variant

Sonnet picks the cleaner shape; if structurally identical, BRIEF's framing wins.

## Pre-flight greps sonnet should re-run (verify ripple)

Before code edits, sonnet should re-verify:
1. `grep -rn "defn-restricted\|def-restricted" wat/ wat-tests/ tests/` — confirm zero hits outside the BRIEF's named files (wat/core.wat, tests/wat_arc198_def_restricted.rs)
2. `grep -rn "def-restricted" src/*.rs` — confirm head-keyword matches at the listed file:line locations don't need shape updates

If pre-flight surfaces additional consumers, STOP trigger 1 fires.

## Out-of-scope findings (surface, don't act)

These may surface during the slice; surface in SCORE honest-deltas, don't act:
- Diagnostic message improvements beyond the minimum (e.g., suggesting the new shape when old shape detected)
- Position-flexibility for `:restricted-to` (DESIGN locks "name, then `:restricted-to`, then prefix-vec, then expr"; if sonnet wants to allow `:restricted-to` to appear after expr, surface as DELTA — don't add)
- Multi-keyword-tag support for future extensions (out of scope; arc 210 is `:restricted-to` ONLY)

## Failure-mode catches

- FM 1 (grep before claiming): every assertion sonnet makes about substrate state cites file:line
- FM 9 (baseline pre-flight): sonnet runs workspace cargo test BEFORE editing to confirm baseline; reports baseline-failure count
- FM 16 (no tool preamble): BRIEF doesn't mention Bash/cargo availability
- `feedback_assertion_demands_evidence`: every substrate claim cites file:line

## What "done" means for this slice

3 SCORE rows YES; workspace cargo test green (= baseline failure count, ±0); ~10-12 test sites migrated; ~50-100 lines changed across substrate + sugar + tests.

## Atomic commit shape

NO commit by sonnet. Orchestrator commits ALL files atomically when sonnet returns + scoring done. One commit: substrate + sugar + tests.

## Calibration record

Arc 207 slice 2 (typed Uuid mint): similar shape; ~30 min sonnet. Arc 198 slice 1 (def-restricted mint, the predecessor): ~45 min substrate work. Arc 210 slice 1 estimate: 45-75 min (additive parser extension; bounded scope; verified pre-flight). Hard stop 90 min.

If sonnet returns under 30 min, calibration data for over-specification.

## Connection to arc 209 forward chain

Arc 210 slice 1 unblocks → arc 210 slice 2 closure (small) → arc 209 Stone A drafts (spawn-program defmacro + walker reshape) → arc 209 Stone B (restricted_to application uses arc 210's new shape from day 1) → arc 209 Stone C (defservice macro) → arc 209 Stone D (counter migration).

Sonnet: trust the substrate; trust the disk; trust the pre-flight; ship the atomic slice; return.
