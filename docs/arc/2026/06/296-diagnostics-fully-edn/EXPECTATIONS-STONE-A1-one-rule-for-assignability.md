# EXPECTATIONS — STONE A-1

Written BEFORE the strike.

| # | what | command | expected |
|---|---|---|---|
| 1 | the subject subsumes | `--check …__if_branches_subtype_related.wat` | EXIT 0 (is 1 today) |
| 2 | the parameter control | `--check …__parameter_subsumes.wat` | EXIT 0 |
| 3 | ⛔ over-reach detector | `--check …__if_branches_unrelated.wat` | EXIT 1 |
| 4 | ⛔⛔ CAPABILITY GUARD | `--check …__if_still_solves_a_type_var.wat` | EXIT 0 |
| 5 | the probe's gate | `-E 'test(a1_one_rule)'` | `4 passed, 0 skipped` |
| 6–8 | P-1 / P-2prereq / P-3 unmoved | their three filters | 10 / 4 / 5 passed, 0 skipped |
| 9 | no second subtyping test | rider names the helper; `grep -c "is_subtype" src/check.rs` does not grow by a new hand-rolled arm | `assignable` reused |
| 10 | no fifth walker | `grep -c "fn walk_" src/declare/typevar.rs` | unchanged, or the rider shows the shared recursion it used |
| 11 | the two positions now agree | rows 1 + 2 together | the same pair of types accepted in BOTH positions |

★ **Row 4 is the row the ruling exists for**, and it is the one a plausible implementation destroys.
"Subsume everywhere" passes rows 1, 2, 3, 5 and every earlier stone's filter — and fails only here.
If it fails *silently* (green because nothing in the fixture forces the variable to be observed),
the row is not doing its job and that is itself a finding.

★ **Row 3 is the row a lazy fix satisfies.** `if` accepting anything passes row 1 trivially.

★ **Row 9 exists because the tempting shape is a local `is_subtype` call at each site.** That would
work, pass every behavioural row, and re-create the exact fragmentation this stone removes — four
positions with four spellings instead of three with three.
`[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`

## Independent prediction

- **Runtime:** 25–45 min. Two call sites; most of it is getting the variable test and the direction
  right.
- **Diff:** ~40–90 lines in `src/check.rs`.
- **Floor:** possibly non-zero and in the PERMISSIVE direction — sites that used to error may now
  widen. Any NEW red is a finding with a verbatim block, classified by REASON before any delta is
  reported. `[[feedback_a_falling_failure_count_is_not_convergence]]`

## Trap-doors named in advance

1. **`unify` mutates `subst`.** A failed `unify` may leave partial bindings behind. If the
   implementation tries one path and falls back to the other, ORDER matters and a failed first
   attempt can poison the second. The ruled rule avoids this by DECIDING first and calling once —
   keep it that way.
2. **"Contains a variable" is not the same as "is a variable."** `Box<T>` is concrete at the head
   and open in its argument. The rule must say which it treats as variable-bearing, and the answer
   determines the admitted cost the DESIGN already records.
3. **`if`'s direction.** Neither branch is "expected"; the result is what both can be seen as. A
   naive fix that keeps treating the THEN branch as authoritative will pass the subject fixture and
   stay asymmetric — swap the branches in a scratch file and check.
4. **`send`'s contravariance.** The payload goes INTO a channel. Widening the value is fine;
   widening the channel is not. If the implementation ends up relating the two channels rather than
   the two values, that is STOP-5.

## What I re-run myself

Rows 1–11 verbatim, then `scripts/floor.sh` unpiped with `$?` read directly, then
`cargo clippy --release --all-targets -- -D warnings`, expecting the same 5 pre-existing
`dead_code` items and no more.
