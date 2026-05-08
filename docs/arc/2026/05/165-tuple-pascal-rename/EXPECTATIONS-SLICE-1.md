# Arc 165 slice 1 — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 30-45 minutes.**

Reasoning: 13 substrate-internal site flips (12 string-literal renames
+ 1 test fixture + 1 docstring) + 4 new test cases + cargo test
iteration. Mechanical scope, no behavioral change beyond the storage
canonicalization. Comparable to arc 163 slice 3f (substrate primitive
paths to FQDN) which ran ~25 min for ~10 site flips.

**Time-box (2× upper-bound): 90 minutes.** Orchestrator schedules a
wakeup at +90 min; if sonnet is still running at that point, it gets
killed via TaskStop and scored as Mode B-time-violation.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A   | `Value::Tuple(_) => "wat::core::Tuple"` at runtime.rs:480 | line shows the FQDN PascalCase string |
| B   | Eval arm key flipped at runtime.rs:3081 | line shows `":wat::core::Tuple"` matching to `eval_tuple_ctor` |
| C   | Type-comparison literal flipped at runtime.rs:3641 | line shows `"wat::core::Tuple"` |
| D   | `eval_tuple_ctor` constructed head flipped at runtime.rs:5607 | line shows `":wat::core::Tuple".into()` |
| E   | Heterogeneous tuple type-check head flipped at check.rs:8959 | line shows `":wat::core::Tuple".into()` |
| F   | Pattern 2 poison at check.rs:3901-3914 UNCHANGED in shape | callee match key still `:wat::core::tuple`; expected/got/redirect target still `:wat::core::Tuple`; one-line comment added |
| G   | Test fixture at check.rs:14463 flipped | line shows `(:wat::core::Tuple counter driver)` |
| H   | New test file `tests/wat_arc165_tuple_pascal.rs` created | file exists with 4 test cases per BRIEF |
| I   | `cargo test --release --workspace --no-fail-fast` clean | 0 failed; total ≥ 2041 + 4 (new tests) |
| J   | Pre-existing test count unchanged | pre-arc-165 baseline 2041 still passes; the new tests are ADDITIONS |
| K   | Comment text updates per BRIEF | runtime.rs:3079 + 5593 + check.rs:1068-1086 docstring + check.rs:8944 docstring all reflect post-arc-165 PascalCase |

## Honest-delta categories (if surfaced, report; don't fix without orchestrator OK)

- **Test discovery beyond the 4 listed cases** — if cargo test reveals
  failures in OTHER test files referring to `wat::core::tuple` (e.g.,
  type_name comparisons in pre-existing tests), report the failures
  and the diagnostic-suggested fix; orchestrator decides whether to
  apply or rescope.
- **Pattern 2 poison's dispatch path** — if the existing
  `infer_tuple_constructor` call after the poison stops working
  because the storage-side rename invalidated some assumption, surface
  the diagnostic; do NOT silently bridge.
- **type_name comparison at runtime.rs:3641** — if this comparison
  was effectively unreachable pre-arc-165 (latent defect: type_name
  returned `"tuple"` while comparison expected `"wat::core::tuple"`),
  the post-arc-165 alignment may CHANGE behavior at sites that hit
  this code path. Report the test count delta if any pre-existing
  tests now exercise this path.

## Calibration row

Actual runtime: ___ minutes (Mode A clean / Mode B partial / Mode C
failed). Compare to predicted 30-45 min band; flag if outside.

## SCORE artifact

Sonnet's report writes to chat; orchestrator commits SCORE-SLICE-1.md
after scoring all rows.
