# Arc 162 — Slice 1 EXPECTATIONS

**Drafted 2026-05-07.** Pre-spawn predictions for sonnet's
arc 162 slice 1 lambda internal-identifier rename.

## Independent prediction

**Mode A.** ~30-45 minutes wall-clock.

Mechanical rename across 4 files load-bearing (`runtime.rs`,
`check.rs`, `freeze.rs`, `sigma.rs`) plus ~5 secondary files
(test files, edn_shim.rs). ~30-50 distinct live-identifier
edits + ~10-20 stale-comment-text edits. Compiler-driven for
the Value variant rename.

The classification framework (Bucket A/B/C/D) is the orientation
device: every site falls into one bucket; sonnet applies the
corresponding action. The procedural order in the BRIEF (Value
variant first, helpers next, public type, strings, tests,
comments) is dictated by compile-failure cascading.

## Hard scorecard

| Row | Pass criterion |
|---|---|
| R1 | Workspace pre-fix: 2041 passed / 0 failed (baseline) |
| R2 | Workspace post-fix: 2041 passed / 0 failed (no regressions) |
| R3 | `cargo build --release` exits clean |
| R4 | Bucket A grep — live lambda identifiers (`wat__core__lambda \| WatLambda \| parse_lambda_signature \| _lambda_body_ \| rhs_spawn_lambda`) returns **0** sites |
| R5 | Bucket D grep — `BareLegacyLambda` returns **28** sites (preserved scaffolding) |
| R6 | Total grep — `lambda\|Lambda` returns ~30-50 sites (only Bucket C historical + Bucket D variants) |
| R7 | Test file `tests/wat_spawn_lambda.rs` renamed to `tests/wat_spawn_fn.rs` (verified via `git status`) |
| R8 | Public type `WatLambdaSigmaFn` renamed to `WatFnSigmaFn`; export updated in `src/lib.rs` |
| R9 | `cargo clippy --release` warning count unchanged from baseline |
| R10 | Sonnet's report includes per-bucket counts + at least 2 honest deltas |

## Path classifications

- **Mode A**: clean rename, all rows pass, audit greps confirm.
  ~30-45 min.
- **Mode B**: rename lands but scope-creep (e.g., touched a
  Bucket C historical comment by mistake). Sonnet self-corrects
  if caught; mode-B-with-self-correction is acceptable.
- **Mode C**: rename doesn't compile, OR audit grep R4 returns
  > 0 (missed a Bucket A site), OR test count regressed. Sonnet
  stops + reports; orchestrator decides next step.

## Time-box wakeup

2× upper-bound = 90 min. Wakeup at T+90 min.

## Honest deltas to flag

- **If `WatLambdaSigmaFn` has external callers we missed:** flag
  in the report. Per memory `project_lab_reconstruction.md` lab
  is in reconstruction, so cross-repo callers shouldn't exist; if
  they do, that's a substrate-API surface we didn't know about.
- **If comments have hybrid wording (live concept + historical
  context in one sentence):** note them. The discipline is to
  keep the historical context and rewrite the live-concept part;
  if a sentence mixes both, sonnet must split the discipline
  per-clause.
- **If test embedded-wat strings have literal `:wat::core::lambda`:**
  those stay — they're test fixtures verifying the retirement
  diagnostic fires. Report any encountered so orchestrator can
  verify.
- **If the Value variant rename surfaces match-arm fall-through
  bugs:** unlikely (the variant is in active use), but flag if
  sonnet finds a previously-unreached arm during cascade fixes.

## Substrate assumptions verified

- `Value::wat__core__lambda(Arc<Function>)` is the legacy variant
  name; renaming the variant is mechanical (compiler will guide
  every match arm). Confirmed at `src/runtime.rs:159`.
- `WatLambdaSigmaFn` is exported in `src/lib.rs:116` as part of
  the public API. Renaming is a public-API change. Confirmed.
- `BareLegacyLambda` variant + Display arms are in active use
  for the arc 155 retirement diagnostic. They KEEP their legacy-
  spelling name per arc 113 orphaned-scaffolding precedent.
  Confirmed via grep (28 sites).
- Test file `tests/wat_spawn_lambda.rs` exists and tests
  spawn-lambda functionality (the `wat_*` test prefix is the
  test-runner discovery pattern). Renaming via `git mv` preserves
  history.

## Cross-references

- `BRIEF-SLICE-1.md` — the brief sonnet executes
- `DESIGN.md` — context and rationale (queued 2026-05-07)
- Arc 155 INSCRIPTION — the surface retirement that left these
  internal leftovers
- Arc 113 — orphaned-scaffolding precedent (variant + Display
  preserved with legacy name)
- Memory `feedback_design_vs_memory.md` — DESIGNs are living
  docs; this BRIEF expanded scope beyond the original DESIGN
  draft when audit grep revealed 304 sites instead of the
  initially estimated ~15-20
