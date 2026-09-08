# EXPECTATIONS — STONE P-2 PREREQ

Written BEFORE the strike. Every bar derives from the rule, not from what I expect to see.

| # | what | command | expected |
|---|---|---|---|
| 1 | the subject flips | `./target/release/wat …__use_then_is_type.wat` | stdout `true` (is `false` today) |
| 2 | ⛔ the over-reach detector does NOT flip | `./target/release/wat …__no_use_is_not_a_type.wat` | stdout `false` |
| 3 | the hand-listed half still answers | `…__handlist_control.wat` | stdout `true` |
| 4 | a phantom stays a phantom | `…__phantom_rust_name.wat` | stdout `false` |
| 5 | the probe's gate | `cargo nextest run --release -E 'test(p2prereq)'` | `4 passed, 0 skipped` |
| 6 | P-1 does not regress | `cargo nextest run --release -E 'test(p1_annotation)'` | `10 passed, 0 skipped` |
| 7 | the wall still refuses a phantom annotation | `…probe_arc296_p1…__phantom_param_and_return.wat` | EXIT 1, names `:usr::TotallyMadeUp` |
| 8 | the collapse | `grep -c "use_decls" src/declare/typevar.rs` | `0` — or the rider reports STOP-3 with the red |
| 9 | `get` still returns None for a seeded name | the rider states which test or assertion proves it | membership without structure preserved |
| 10 | hand-list untouched | `git diff --stat src/types.rs` shows no deletion in Group 3 | crossbeam rows intact |

★ **Row 2 is the row a defect satisfies.** Rows 1, 3, 4, 5, 6, 7 all pass under the WRONG fix —
seeding every `RustDepsRegistry` name at startup. Only row 2 separates "this program declared it"
from "the build knows about it," and it is the same NAME as row 1, so nothing but the `use!` line
distinguishes them.

★ **Row 8 is a collapse, not a cleanup.** If the fourth store is truly subsumed, its arm is dead
code; if the arm is still needed, the two stores are not equivalent and the DESIGN is wrong. Either
answer is worth having — but "left it in, harmless" is not one of them.

★ **Row 9 exists because the tempting shortcut is a fabricated `TypeDef`.** `builtin_names`' field
doc records that options A/B (register as an `Aggregate`/`Alias`) were considered and rejected
precisely because they invent a structure that does not exist. A seeded `:rust::*` name must have
membership and no structure. `[[feedback_the_wall_was_the_fix_not_the_fold]]`

## Independent prediction

- **Runtime:** 20–35 min. The seed is a few lines; the collapse and its verification are most of it.
- **Diff:** ~25–60 lines, plus deletions if row 8 holds.

## Trap-doors named in advance

1. **Idempotence.** `register_builtin_leaf` `debug_assert!`s on a duplicate. Disjoint today; a seed
   that can run twice, or a name that later joins both populations, panics in DEBUG only — where
   the floor does not run. A release-green tree would hide it.
2. **Ordering.** The seed must precede every consumer that asks `contains`, not merely the
   annotation wall. If an earlier pass asks, seeding at line 258 is too late — that is a finding.
3. **The fork.** `TypeEnv` ships across a process boundary. A seeded name must survive the fork, or
   a spawned child disagrees with its parent about the same program — the very defect this stone
   closes, re-opened along a different axis.
4. **`use!` inside a non-residue form.** The collection loop walks `residue`. If a `use!` can appear
   somewhere the residue does not carry, that name is silently unseeded. RELAND-1 already hit this
   shape once (stdlib `use!` dropped when residue was filtered to runtime-def forms).

## What I re-run myself, not take on report

Rows 1–8 verbatim, then `scripts/floor.sh` unpiped with `$?` read directly, then
`cargo clippy --release --all-targets -- -D warnings` — expecting the same 5 pre-existing
`dead_code` items and no more.
