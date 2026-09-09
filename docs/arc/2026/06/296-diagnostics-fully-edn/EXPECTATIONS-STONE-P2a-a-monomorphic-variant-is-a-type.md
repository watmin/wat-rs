# EXPECTATIONS — STONE P-2a

Written BEFORE the strike.

| # | what | command | expected |
|---|---|---|---|
| 1 | the annotation is accepted | `--check …__variant_annotation.wat` | EXIT 0 |
| 2 | ⛔ subsumption still carries | `--check …__variant_flows_to_enum_param.wat` | EXIT 0 |
| 3 | ⛔ the direction holds, by MESSAGE | `--check …__enum_does_not_flow_to_variant_param.wat` | EXIT 1, message names `Colour::Red`, does **not** contain `UnknownNamedType` |
| 4 | the verb agrees | run `…__is_type_on_a_variant.wat` | stdout `true` |
| 5 | ⛔ the fence, by MESSAGE | `--check …__generic_variant_stays_refused.wat` | EXIT 1, message **is** `UnknownNamedType` |
| 6 | the probe's gate | `-E 'test(p2a_a_monomorphic)'` | `5 passed, 0 skipped` |
| 7 | P-1 holds | `-E 'test(p1_annotation)'` | `10 passed, 0 skipped` |
| 8 | P-2 prereq holds | `-E 'test(p2prereq)'` | `4 passed, 0 skipped` |
| 9 | P-3 holds | `-E 'test(p3_one_question)'` | `5 passed, 0 skipped` |
| 10 | the runtime value is unchanged | the rider states whether `src/record/construct.rs` was touched and why | checker-side only, or a named STOP-4 |
| 11 | the corpus | floor, by the orchestrator | any remainder classified BY REASON, one verbatim block per cluster |

★ **Row 3 is the row a defect satisfies, and it satisfies it TODAY.** The fixture exits 1 right now
because the annotation names an unknown type. After the stone it must exit 1 because a `Colour` is
not known to be a `Colour::Red`. **Identical exit code, opposite mechanism** — so the bar is the
message, in both directions: it must gain `Colour::Red` and lose `UnknownNamedType`.
`[[feedback_an_acceptance_row_a_defect_can_satisfy_is_not_a_row]]`

★ **Row 5 is the fence, and it is also message-bound.** A stone that half-supported generics could
leave the generic fixture exiting 1 for a new and different reason while claiming the fence held.

★ **Row 2 is the widest control.** Step 3 of the design changes the ctor's type out from under every
existing enum construction in the corpus. If subsumption does not carry it, this row goes red first
and everything after it is noise.

★ **Row 10 exists because the tempting shortcut is to change the VALUE.** The wire form already
carries the variant (`#wat.core/Option.Some {…}`, 296 H). A runtime-shape change would ripple into
EDN, comms and goldens — a different stone, and STOP-4 rejects it here.

## Independent prediction

- **Runtime:** 40–70 min. Registration and the edge are small; the ctor's result type is where the
  corpus answers back.
- **Diff:** ~60–120 lines in `src/`, plus whatever the floor demands.
- **Floor:** I expect a NON-ZERO first count. 104 monomorphic enums now construct at a narrower
  type. That is substrate-as-teacher, not a crisis — but the remainder gets classified by REASON
  before any delta is reported. `[[feedback_a_falling_failure_count_is_not_convergence]]`

## Trap-doors named in advance

1. **Subsumption at the argument position may not be the same path arc 209 proved.** Arc 209 is a
   marker BOUND. An ordinary parameter may take a different arm. Confirm, do not assume — STOP-1.
2. **A variant's own structure.** A variant has fields; registering it as a membership-only leaf
   would make `type-of` answer `None` for something that plainly has structure. Decide deliberately
   whether it is a leaf or carries a `TypeDef`, and say which and why.
3. **Name collision.** `:usr::Colour::Red` as a type name and `(:usr::Colour::Red {…})` as a call
   head are the same string in two positions. P-1's wall, `resolve`, and the ctor intercept all read
   that string. A change to how it registers may reach all three.
4. **`match` arms.** `[:usr::Colour::Red {:shade s} …]` already parses the variant in PATTERN
   position. If registration changes what that string resolves to, patterns are the second consumer
   and the probe does not cover them. If they move, that is a finding.

## What I re-run myself

Rows 1–10 verbatim, then `scripts/floor.sh` unpiped with `$?` read directly, then
`cargo clippy --release --all-targets -- -D warnings`, expecting the same 5 pre-existing
`dead_code` items and no more.
