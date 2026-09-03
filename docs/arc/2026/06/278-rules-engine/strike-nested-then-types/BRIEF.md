# BRIEF — carry D10's type check one level down

D10 types a `:then` fact form's own fields. A constructor **nested inside** one is still untyped, so
a wrong-typed value still reaches the fact set. Every piece needed already exists; this thread one
parameter through and calls them.

## Read in order

1. **Run the repro first**: `./target/release/wat` on a file with
   `:then [(:nh::Outer :i (:nh::Inner :n ?s))]` where `?s` is a `String` and `Inner.n` is `i64`.
   Driven at `e38b1f46a`: it compiles, fires, and derives
   `#nh/Outer {:i #nh/Inner {:n "nested-string"}}`. Build your fixture from that.
2. `src/rete/validate/mod.rs:774` — `walk_nested_constructors`. It takes
   `(operand, rule_name, types, errors)`. **`binds` is what is missing.**
3. `src/rete/validate/mod.rs:1088-1095` and `:1117-1124` — the two external call sites, both inside
   `validate_then_form`. **`binds` is already in scope on the line above each**, because D10 hoisted
   `collect_rule_bind_types` out of an inner block.
4. `src/rete/validate/typing.rs:177` — `lookup_field_types(types, fact_type)`, parallel to the
   `lookup_fields` the walker already calls at `~:877`.
5. `src/rete/validate/typing.rs:743` — `check_then_field_type`, D10's producer. **Call it unchanged.**
6. `src/rete/validate/mod.rs:810-830` — the `match` arm of this same walker (D5's cure): it skips an
   arm's **pattern** and recurses into its **body**. Your change must not disturb that.

Also read `docs/arc/2026/06/278-rules-engine/strike-nested-then-types/DESIGN.md` and `.../EXPECTATIONS.md`,
and `strike-then-rhs-types/SCORE.md` for how D10's check behaves.

## The shape

Add `binds` to `walk_nested_constructors`, thread it through all **7** call sites (5 recursive, 2
external — all in `validate/mod.rs`), and in the walker's aggregate branch — where it already resolves
the nested type and walks its kv pairs — look up the field types and call `check_then_field_type` per
pair, exactly as the top level does. Do the same in that branch's positional arm.

**No new error kind.** `RhsFieldTypeMismatch` already carries the six fields; a nested occurrence is
the same claim at a different position.

## ⛔ Three traps

1. **Not-knowable is not wrong** — identical to D10, now at depth. Construct one and prove it still
   compiles.
2. **D5's cure must survive.** This walker skips `match` arm **patterns** and recurses into **bodies**.
   A bare variant keyword in a pattern is not a value to type. Regressing this re-refuses legal
   `match` in `:then`.
3. **⛔ Give every new `.wat.bad` fixture a REAL `main`.** The legacy idiom ends
   `(:user::main [] -> :wat::core::nil nil)`, which is *itself* a startup failure — so `assert!(!ok)`
   on such a fixture cannot go red under the mutation it exists to detect. Your fixtures must **run
   and print** when the wall is absent.

## Blast radius

`src/rete/validate/mod.rs` and a gate with adjacent fixtures. No new error kind, no `typing.rs`
change, no engine.

## STOP triggers

1. **Measure the corpus before shipping**, as D10 did (it scanned 1664 `.wat` at HEAD and cured and
   diffed per file). **Any legal program that stops compiling is a finding** — report it.
2. **If `match`-in-`:then` starts being refused**, stop — that is D5 regressed.
3. **If you need a new error kind or a `typing.rs` change**, stop and report why.
4. **If threading `binds` requires touching anything outside `validate/mod.rs`**, stop and report the
   call chain.

## Mutation proofs — run all four, report all four

1. **The nested repro is REFUSED** after the cure, with both types named and the caret on the nested
   operand. A well-typed nested control must still compile and derive.
2. **A not-knowable operand at depth still compiles** — constructed, not asserted.
3. **Revert the cure** → the nested repro compiles again and `#nh/Inner {:n "…"}` reappears in the
   fact set.
4. **`match` in `:then` still compiles** — D5's own repro (`experiri-then-match.wat`) must still load.

Restore after each.

## What to report

- The corpus count, and every site that newly fails.
- The nested repro before and after, verbatim.
- All four mutation results.
- Scoped nextest Summary lines including `binary_id(wat::lint)`.
- Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.
- **Anywhere this brief was thin or wrong.** Twelve riders have run on this arc; every one found a
  real defect in the brief. The last found that my "everything needed is already in scope" was false
  in a way that would have shipped a green, wrong cure. Be blunt.

Do not commit.
