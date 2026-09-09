# EXPECTATIONS — STONE P-3

Written BEFORE the strike. Every bar derives from the rule.

| # | what | command | expected |
|---|---|---|---|
| 1 | half 1 subject refuses | `--check …__user_annotation_without_user_use.wat` | EXIT 1, names `:rust::sqlite::Connection` |
| 2 | half 1 control accepted | `--check …__user_annotation_with_user_use.wat` | EXIT 0 |
| 3 | ⛔ the widest control | run `…__stdlib_annotation_still_loads.wat` | stdout `"loaded"` |
| 4 | half 2 subject | run `…__is_type_on_a_derive_marker.wat` | stdout `true` |
| 5 | ⛔ half 2 over-reach detector | run `…__is_type_on_a_non_marker.wat` | stdout `false` |
| 6 | the probe's gate | `-E 'test(p3_one_question)'` | `5 passed, 0 skipped` |
| 7 | P-1 does not regress | `-E 'test(p1_annotation)'` | `10 passed, 0 skipped` |
| 8 | P-2 prereq does not regress | `-E 'test(p2prereq)'` | `4 passed, 0 skipped` |
| 9 | no SKIP arm was added | `grep -n "continue" src/check.rs` in the validator | no arm that skips a declaration; the rider quotes the scope-selection line |
| 10 | corpus unmoved | the rider's sweep of `wat/ wat-scripts/ wat-tests/` | zero refusals outside the probe fixtures |
| 11 | the three agree, both ways | rows 1+2 with `resolve`'s answer for the same names | refuse together without a `use!`; accept together with one |

★ **Row 3 is the row that catches the catastrophic version.** If the stdlib arm is drawn wrong, the
stdlib's own `:rust::sqlite::*` annotations stop resolving and NOTHING loads — rows 1, 4, 5 would
all still "pass" in the sense of producing their expected exit codes for the wrong reason, because
a program that cannot load refuses everything. Row 3 is the only row that distinguishes *"the
subject was refused"* from *"the world stopped."*

★ **Row 5 is the row a defect satisfies.** Half 2 done as `|| true`, or as "any keyword with a
namespace," passes rows 4, 6 and every other row. Only row 5 separates *"is a parent in some
subtype edge"* from *"is a plausible-looking name."*

★ **Row 9 is a claim about SHAPE, not behaviour.** RELAND-1 deleted a `continue`; this stone
re-introduces the same predicate for a different purpose. If the diff contains an arm that declines
to validate a declaration, the stone has re-shipped the blanket while passing every behavioural
row. `[[feedback_a_rejected_option_returns_in_new_clothes]]`

★ **Row 10 is derived, not hoped.** The census measured every non-stdlib `.wat` that annotates a
`:rust::` type and found all of them carry their own `use!`. A non-zero answer refutes the census,
not the fixture.

## Independent prediction

- **Runtime:** 20–40 min. Half 2 is one line; half 1 is a signature change plus the per-entry
  selection at two loops.
- **Diff:** ~40–80 lines.

## Trap-doors named in advance

1. **The merged set is convenient and wrong.** Reusing `use_decls` for the reserved-prefix arm
   passes every row here, because stdlib annotations only reference stdlib `use!`s today. It would
   leave the wall permissive for any FUTURE stdlib annotation of a user-declared name. Keep the
   stdlib set stdlib-only.
2. **`symbols.functions_iter()` includes generated functions.** Enum ctors, struct methods, surface
   methods — a generated `:wat::*` name whose annotation came from user source would be judged in
   the wrong scope. If that shape exists, it is a finding.
3. **`is_subtype_parent` scans values, not keys.** It is `O(edges)` per call and `is-type?` is a
   user-callable verb; a corpus sweep asking it in a loop is a different cost profile than the
   wall's one-shot pass. Not a correctness issue — worth one sentence if it shows.
4. **Half 2 and the wall can drift again.** They now share `is_subtype_parent` but not one
   predicate. If a fifth store ever appears, two places must change. Say so if a shared predicate
   is within reach without a refactor this stone did not authorise.

## What I re-run myself

Rows 1–11 verbatim, then `scripts/floor.sh` unpiped with `$?` read directly, then
`cargo clippy --release --all-targets -- -D warnings`, expecting the same 5 pre-existing
`dead_code` items and no more.
