# Arc 155 — Substrate EXPECTATIONS (slice 1a)

**Drafted 2026-05-06 evening.**

**Brief:** `BRIEF-SUBSTRATE.md`
**Output:** EDITS to 3 src files + NEW test file. NO commits.
**Model:** `model: "sonnet"` explicit per FM 12.

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File scope | EXACTLY 4 files (3 modified + 1 new) |
| 2 | `BareLegacyLambda` variant minted | New CheckError variant + Display referencing arc 155 + canonical `:wat::core::fn` |
| 3 | `BareLegacyLowercaseFn` variant minted | New CheckError variant + Display referencing arc 155 + canonical `:wat::core::Fn` |
| 4 | Operator-position walker fires | `walk_for_legacy_lambda` detects `:wat::core::lambda` Keyword; emits per site |
| 5 | Type-position walker fires | `walk_for_legacy_lowercase_fn` detects bare `:fn` parametric head; emits per site |
| 6 | `:wat::core::fn` operator works | `infer_fn` + `eval_fn` route from `:wat::core::fn` keyword (renamed from `infer_lambda` / `eval_lambda`) |
| 7 | `:wat::core::Fn(...)` type works | Type registry resolves `:wat::core::Fn(:T)->:U` to function-type representation |
| 8 | New tests run | `cargo test --release --test wat_arc155_fn_rename` shows the negative-shape tests pass; positive tests may be blocked pre-sweep (matches arc 154 pattern) |
| 9 | Workspace failure shape | `cargo test --release --workspace` fires many `BareLegacyLambda` + `BareLegacyLowercaseFn` errors; NO panics outside intentional thread-panic tests; NO unrelated TypeMismatch |
| 10 | No commit | HEAD unchanged from spawn-time |

**Hard verdict:** all 10 must hold. Rows 4, 5, 6, 7, 9 are
load-bearing.

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC delta | 150-220 LOC (1.5x arc 154's slice 1a) |
| 12 | Pattern fidelity | Mirrors arc 154's let* recipe + arc 109 slice 1e's parametric-head FQDN recipe |
| 13 | clippy clean | No new clippy warnings |
| 14 | No grinding | No backwards-compat shims |

## Independent prediction

- **Most likely (~70%) — Mode A clean.** Both recipes
  (operator-position keyword rename + type-position parametric
  head FQDN'ing) shipped successfully today / earlier (arc 154
  + arc 109 slice 1e). Bundled implementation is mechanical
  composition. ~40-60 min wall-clock under Sonnet.
- **Mode B-substrate-bug (~10%):** type-position walker
  interaction with parser layer surprises (the `:fn` keyword
  may need explicit registration in multiple places).
- **Mode C-bundled-rename-edge (~10%):** the bundle creates
  unforeseen interactions (e.g., reflection trio sees both old
  and new spellings in unexpected ways).
- **Mode B-time-violation (~10%):** Sonnet runtime exceeds 75
  min cap (Sonnet pace differential vs Opus calibration).

## Time-box

75 minutes wall-clock. ScheduleWakeup at T+75 min.

## What success unlocks

**Mode A clean:** sweep 1b can spawn — sonnet reads both walker
diagnostic streams + applies appropriate 1:1 transform per site
(~476 sites total).

## After sonnet completes

- Read this file FIRST
- Score each row
- Verify load-bearing rows by re-running `cargo test --release
  --test wat_arc155_fn_rename` locally
- Sample 2-3 workspace failures to confirm both walker
  variants firing correctly
- DO NOT COMMIT — atomic commit per recovery doc § 7 with sweep 1b

## Why this matters

User direction 2026-05-06 evening: *"we're moving closer to
clojure"* + *"everything needs a namespace."* Arc 155 lands the
fourth foundation mark of the day (after nil + do + let
sequential). The Lisp on Rust gains its function vocabulary in
the conventional Cap-type / lowercase-verb shape Clojure /
Rust / Java / Swift / Kotlin all use.
