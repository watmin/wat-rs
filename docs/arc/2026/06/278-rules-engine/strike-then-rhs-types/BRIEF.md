# BRIEF — type the `:then` RHS field values

A rule's `:then` writes values into a record's declared fields and never checks their types, so a
`String` lands in a field declared `i64` and enters the fact set. Everything needed to fix it is
already in the file and in scope.

## Read in order

1. `wat-scripts/scratch-pad/d10-then-rhs-is-not-type-checked.wat` — **run it first**:
   `./target/release/wat <path>` prints `CONTROL Good count: 1`, `SUBJECT Bad count: 1` and the
   derived `#tr/Bad {:n "not-an-i64"}`. The control is what makes it evidence; keep it in whatever
   fixture you build.
2. `src/rete/validate/mod.rs:966` — `validate_then_form`, and its loop at ~`:1022` building
   `kv_pairs: Vec<(String, WatAST)>`. **The field name and its value AST are already paired**, and
   `types: &TypeEnv` is already a parameter. `check_field_kw` beside it validates the NAME.
3. `src/rete/validate/typing.rs:405` — `resolve_operand_type(operand, field_names, field_types,
   binds, types)`, **already standalone**, four sources: `:field` → declared type; `?var` → the
   field its bind names, then that field's type; literal → its own type; nested call → its row's
   `ret`.
4. `src/rete/validate/typing.rs:370-390` — that resolver's doc, **including the record of source 4
   having been missing while the doc claimed exhaustiveness.** Read the arms as code.
5. `src/rete/validate/error.rs:89-95` — `RhsArityMismatch`'s shape, for a new kind. There is **no**
   RHS type kind today (`grep -c` → 0).
6. `src/rete/validate/typing.rs` — `OperandType`'s variants, especially the one carrying
   *not-derivable-here*. **This is the distinction the cure turns on.**

## ★ The invariant

> A `:then` field value whose type is **knowable** and does not match the destination field's
> declared type is refused at rule-compile time.

## ⛔ The trap, and it is the whole difficulty

**Not-knowable is not wrong.** `OperandType` already separates *knowable-and-wrong* from
*not-knowable*, and the `:when` side carries a dedicated variant so they cannot be confused. Refuse
only the first. A `?var` bound from a derived fact, a computed operand whose head is
`Form`/`Redispatch`, a type variable — all must still compile.

## Blast radius

`validate/mod.rs`, `validate/error.rs`, a gate with adjacent `.wat` / `.edn` / `.wat.bad` fixtures.
`typing.rs` only if the resolver needs a wrapper. **No engine, no fire path.**

## STOP triggers

1. **⛔ Before writing the cure, measure the corpus.** Count how many existing `:then` forms across
   `wat/`, `wat-scripts/`, `wat-tests/` and `tests/` would newly be refused. **If any legal program
   in the tree stops compiling, STOP and report it** — that is either a real latent bug you have
   found or a false positive in the cure, and both outrank shipping.
2. **If you find yourself refusing a not-knowable operand**, stop. That is the named failure.
3. **If the fix needs `resolve_operand_type` changed** rather than merely called, stop and report
   why — its four sources are shared with the `:when` path and a change there moves both.
4. **If a new error kind needs a field the existing kinds do not carry**, say so — diagnostic
   completeness is a standard here, not a nicety.

## Mutation proofs — run all three, report all three

1. **The repro's SUBJECT rule must be REFUSED** after the cure, with the declared and actual types
   both named. Its CONTROL rule must still compile and derive.
2. **A not-knowable operand must still compile** — construct one (a `?var` bound from a derived
   fact, or a computed operand whose type the resolver reports as not-derivable-here) and show it
   passes. **Without this row, a cure that refuses everything scores full marks.**
3. **Revert the cure** → the repro's SUBJECT compiles again and the wrong-typed fact reappears in
   the fact set.

## What to report

- The corpus count from STOP-1, and every site that newly fails.
- The repro before and after, both arms.
- All three mutation results.
- The new error kind and what it carries.
- Scoped nextest Summary lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong.** Eleven riders have run on this arc; every one found a
  real defect in the brief, twice an instrument I named that was structurally blind to what I
  pointed it at. Be blunt.

Do not commit.
