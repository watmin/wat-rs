# EXPECTATIONS — STONE A-2

| # | what | command | expected |
|---|---|---|---|
| 1 | the builder's function | `--check …__process_full_box.wat` | EXIT 0 (is 1 today) |
| 2 | ⛔⛔ the builder's posterity example | `--check …__ctor_carries_the_variant.wat` | EXIT 0 (is 1 today) |
| 3 | ⛔ the widest control | `--check …__variant_widens_to_enum.wat` | EXIT 0 |
| 4 | regression control | `--check …__match_still_works.wat` | EXIT 0 |
| 5 | over-reach detector | `--check …__nonexistent_variant.wat` | EXIT 1 |
| 6 | ⛔ direction, by MESSAGE | `--check …__enum_does_not_narrow.wat` | EXIT 1, message does **not** contain `UnknownNamedType` |
| 7 | the probe's gate | `-E 'test(a2_a_variant)'` | `6 passed, 0 skipped` |
| 8–12 | P-1 / P-1b / P-2prereq / P-3 / A-1 | their five filters | 10 / 4 / 4 / 5 / 4, all 0 skipped |
| 13 | the runtime value is untouched | `git diff --stat src/record/construct.rs` | empty, or a named STOP-4 |
| 14 | the variant is not an Aggregate | rider states which `TypeDef` it registered and why | a variant stays a variant |

★ **Row 2 is the row P-2a lacked, and it is the reason that stone was reverted.** It CONSTRUCTS a
value and passes it into a variant-typed parameter. Rows 1, 3, 4, 5, 6 can all pass while the
parameter is uninhabitable — P-2a proved that empirically, with five green rows in the state its own
design called worse than doing nothing. **Only row 2 separates "the type exists" from "the type is
inhabited."** `[[feedback_a_green_test_can_prove_nothing]]`

★ **Row 6 is the row a defect satisfies, and it satisfies it TODAY.** Exit 1 now (unknown type),
exit 1 after (direction violation). Same code, opposite mechanism, so the bar is the message.

★ **Row 3 is the widest thing this stone can break.** The ctor's type changes out from under every
existing enum construction in the corpus.

★ **Row 14 is a SHAPE claim, not a behavioural one.** Registering a variant as an Aggregate would
make `{:keys}` work and pass every other row — and it is the builder's explicitly refused design.
The rider must NAME the choice, not leave it inferable from a green.

## Independent prediction

- **Runtime:** 60–120 min. Three coupled pieces; the corpus is the unknown.
- **Diff:** ~80–180 lines in `src/`, plus whatever the floor demands.
- **Floor:** I expect a NON-ZERO first count and the builder has ruled that acceptable. Classify by
  REASON before any delta. `[[feedback_a_falling_failure_count_is_not_convergence]]`

## Trap-doors named in advance

1. **The ctor's type flows into type-variable inference.** P-2a's second cluster was
   `RecvOutcome<Variant>` vs `RecvOutcome<Enum>` — a narrow type leaking into a container's ARG,
   where the two containers then differ. A-1 subsumes concrete pairs; a pair whose args differ is
   NOT subsumed, by design. If this cluster returns, it is a real finding and STOP-5 applies.
2. **`register_enum_methods` skipping its own new types.** P-2a needed `is_monomorphic_variant_type`
   guards so it did not mint `:Enum::Variant::Variant`. The same hazard exists here.
3. **Unit variants.** `Box::Empty` has no fields. It must still be a type, still widen, and still not
   acquire a `{:keys}` binding surface it has no fields for.
4. **`type-of` on a variant.** It must ANSWER rather than raise, and what it answers is a design
   choice the rider should state rather than let fall out.

## What I re-run myself

Rows 1–14 verbatim, then `scripts/floor.sh` unpiped with `$?` read directly, then
`cargo clippy --release --all-targets -- -D warnings`, expecting the same 5 pre-existing items.
