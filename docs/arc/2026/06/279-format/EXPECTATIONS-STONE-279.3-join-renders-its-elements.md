# EXPECTATIONS — 279.3 · `join` renders its elements

**Written before the strike, 2026-08-17, against `9fb309e1`.** The scorecard is fixed here so the
result cannot move the goalposts.

## The scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | `join` accepts non-String elements | the kept test's `(join "," [1 2 3])` row | `"1,2,3"` |
| 2 | ★ strings render BARE | the kept test's `(join "-" ["a" "b"])` row | `"a-b"` — **not** `"\"a\"-\"b\""` |
| 3 | 26 pre-existing live sites unbroken | `scripts/floor.sh` Summary + the wat-scripts loader gate | 0 FAIL |
| 4 | one door, two callers | `grep -c 'render_str_total' src/string_ops.rs src/runtime.rs` | ≥1 in runtime.rs, ≥2 in string_ops.rs (def + use) |
| 5 | scheme is generic | `sed -n '16598,16615p' src/check.rs` | `type_params: vec!["T".into()]`, `Path("T")`, no `string_ty()` in the Vector arg |
| 6 | comment no longer lies | `sed -n '505,515p' src/string_ops.rs` | `render_unquoted` not described as `str` |
| 7 | the test is KEPT | `git status --short tests/kernel/` | both `.wat` and `.rs` modified, nothing deleted |
| 8 | floor | `scripts/floor.sh` → Summary line | **4694+ passed, 0 failed, 0 timed out** |
| 9 | clippy | `cargo clippy --release --all-targets` | **0 warnings** |
| 10 | ignores held | `grep -rnE '^[[:space:]]*#\[ignore' tests/ src/ crates/ benches/ --include=*.rs \| wc -l` | **13** |

Row 8's floor count: `9fb309e1` ran **4694/4694**. Two new assertions land, so **4696** is the
expected shape — a *lower* count is a finding, not a rounding.

## Independent prediction

**Runtime: 25–40 minutes**, dominated by two release builds (the change touches `check.rs`, which is
large, and `runtime.rs`). The edit itself is well under 10 minutes of work: one new 6-line function,
one call-site swap, one loop body replaced, one scheme rewritten, two test rows.

**Time-box: 80 minutes** (2× upper bound).

## Trap doors — named before the strike

1. **The String arm gets lost in the refactor.** The single most likely failure, and the most
   expensive, because a *narrow* gate can still pass while 26 sites corrupt. Row 2 exists to catch
   exactly this and is why it is load-bearing rather than decorative.
2. **`sym.types()` is dropped at one of the two call sites.** Silent: records degrade to
   `{:field-0 1}` and no existing test may notice. Row 4 forces both callers through the door so the
   argument is passed once, in the open, per `edn_shim.rs:3490`'s own stated discipline.
3. **`TypeExpr::Var` copied from the superseded draft** → does not compile. Cheap, loud, self-solving.
4. **A `.wat` site that binds join's result into a `Vector<String>`-typed slot.** `Vector<String>`
   unifies at `T = String`, so it should hold — but *should* is why STOP-1 exists rather than a
   prediction.
5. **The `expected: "Vec<String>"` string in the non-Vec error arm.** Easy to leave behind; it would
   then be a diagnostic that names a constraint the code no longer has. Row 5's spirit, checked by
   reading the diff.

## What would make me call this Mode B

- Row 2 green but achieved by special-casing `Value::String` **inside `eval_string_join`** rather
  than in the door — that ships the correct output through the shape this stone exists to delete.
- Floor green with the kept test absent, or with it living in `wat-scripts/scratch-pad/`.
- Any `#[ignore]` added, for any reason, with any justification.

## What I will re-run myself before committing

Rows 1, 2, 4, 8, 9, 10 — independently, on my own invocation, per FM 9. The rider's numbers are a
hypothesis until my own run agrees.
