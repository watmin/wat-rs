# EXPECTATIONS — STONE P-1

Written BEFORE the strike, so the result cannot move the goalposts. Every bar is derived from the
rule, not from what I expect the rider to produce.

## The scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the subject refuses — param + return | `./target/release/wat --check tests/types/probe_arc296_p1_annotation_names_a_type__phantom_param_and_return.wat; echo $?` | `1`, and the message NAMES `:usr::TotallyMadeUp` |
| 2 | the subject refuses — field position | same, `__phantom_record_field.wat` | `1`, and the message NAMES `:usr::AlsoMadeUp` |
| 3 | ⛔ the trap holds — a bound type param | same, `__generic_type_param.wat` | `0` |
| 4 | ⛔ the over-reach detector — bare Uppercase, NO binder | same, `__phantom_bare_uppercase_is_a_var.wat` | `0` |
| 5 | the sibling wall still names itself | same, `__bare_legacy_primitive.wat` | `1` **and stderr still says `BareLegacyPrimitive`**, not the new variant |
| 6 | declared types unaffected | same, `__control_declared_type.wat` | `0` |
| 7 | builtins + instantiated generic unaffected | same, `__builtin_and_generic_instantiation.wat` | `0` |
| 8 | the probe's own gate | `cargo nextest run --release -E 'test(p1_annotation)'` | `7 tests run: 7 passed`, **0 skipped** |
| 9 | no second walker | `grep -c "fn walk_" src/declare/typevar.rs` | unchanged from HEAD, or the new fn shares `walk_free_type_vars`'s recursion — the rider states which and shows the diff |
| 10 | the census exists and is deduped | the rider's report | a count of refusing files + a deduped, frequency-ordered name list |

★ Row 5 is the one a defect could otherwise satisfy. A wall that refuses `:i64` with the NEW generic
error would pass a bare `exit == 1` check while silently demolishing arc 109's named diagnostic. The
bar is the variant name in the output, not the exit code.

★ Row 4 is the widest over-reach path: `:Whatever` has no binder anywhere and is accepted purely by
the lexical rule. A wall that consulted `raw_type_params` alone — without `is_type_var_path` — would
refuse it, and rows 1/2/3/6/7 would all still pass.

★ Row 8 says **0 skipped** because the two subject tests are `#[ignore]`d at HEAD. A run reporting
`5 passed, 2 skipped` is the stone not having removed the attributes, and it looks identical to
success in a summary line.

## Independent prediction

- **Runtime:** 25–45 min for the wall + the two un-ignores. The census adds 5–15 min of wall clock
  (833 files × ~0.2s, serial).
- **Diff size:** ~60–110 lines in `src/declare/`, plus one error variant and its render arm.

## The trap-doors named in advance

1. **The census comes back large.** STOP-3 caps it at ~40 distinct names. Above that, the population
   is not "phantoms" and the design has not seen it.
2. **`defenum` variant field annotations may take a different registration path** than `defn` params.
   If the field fixture (row 2) needs a second site, that is an honest delta to report, not a scope
   expansion to absorb quietly.
3. **A type used before it is declared.** If registration order means a forward reference to a type
   declared later in the file now refuses, that is a REAL finding and a design gap — report it with
   the verbatim case rather than adding a "declared later" escape.
4. **The error variant's render arm.** Diagnostics are fully EDN in this arc; a new variant that
   does not render is a red in a place the seven probe tests do not look.

## What I will re-run myself, not take on report

Rows 1–8 verbatim, then `scripts/floor.sh` unpiped with `$?` read directly, then clippy. The floor
is mine; per FM 18/19 the rider never runs it.
