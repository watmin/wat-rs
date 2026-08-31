# BRIEF — rete's answer must agree with core's

Make a bare keyword constant type as `enum` only when it names a **unit variant that exists**, so a
misspelled or tagged variant is refused the way core already refuses it. Read `DESIGN.md` first —
its **arm table** is the work, and the obvious one-line fix closes only one of the two arms.

## Read in order, and why

1. **`src/rete/validate/typing.rs`, `keyword_constant_segment`** — twelve lines. The whole defect:
   `rsplit_once("::")` + `TypeDef::Enum`, and no check that the variant exists.
2. **`src/rete/matcher.rs:130`, `enum_variant_ctor`** — **the cure, already written**, with the doc
   that names this exact class: *"ONE COPY … hand-written at THREE independent sites."* Note it
   returns `Some((enum, variant, ARITY))` for Unit **and** Tagged — the arity is what closes arm 2.
3. **`src/rete/expr_ir/mod.rs`, `keyword_value`** — the runtime's answer, `sym.unit_variant`, which
   is **unit-only**. This is the function `typing.rs` must stop disagreeing with.
4. **`strike-enum-variant-typo/probe.{rs,wat,-bad.wat}.txt`** → copy to `tests/rete/` as
   `probe_arc278_enum_variant_typo.{rs,wat}` and `..._bad.wat`. Driven RED at `c75b0152c`.

## The dispositions

| arm | fixture | today | wanted |
|---|---|---|---|
| control | `probe…typo.wat` | prints `1` | unchanged — **must stay 1** |
| 1 misspelled | `probe…typo_bad.wat` | prints `0`, exit 0 | **refused**, located |
| 2 bare tagged | **you must build it** — DESIGN gives the shape | prints `0`, exit 0 | **refused**, located |

Arm 2's fixture is a two-line variant of the control: `(:wat::core::defenum :tg::P :wat::enum::Pure
:Hi [n <- :wat::core::i64])`, a field typed `:tg::P`, seeded `(:tg::P::Hi 7)`, constrained on the
bare `:tg::P::Hi`. It was driven; it prints `0`.

## The order

1. Copy the probe pair. Run both: **control PASS, arm-1 FAIL.** Quote the FAIL verbatim.
2. Add arm 2's fixture and test. **Confirm it is also RED** before fixing anything.
3. Fix: route through `enum_variant_ctor` **and require arity 0**.
4. Both arms GREEN, control still `1`.
5. Mutation-prove **each arm separately**: drop the arity-0 requirement → **arm 2 alone** reddens;
   restore the hand-rolled prefix resolution → **arm 1 alone** reddens.

## STOP triggers

1. **If routing through `enum_variant_ctor` alone makes both arms green, STOP and re-check arm 2's
   fixture** — the helper accepts tagged variants, so a green arm 2 means the fixture is not
   reaching the tagged path.
2. **If the refusal's reason or phase differs from core's, STOP and surface it.** ★ decision.
3. **If `keyword_constant_segment` has a caller beyond the two in `typing.rs`, STOP.**
4. **If the control stops printing `1`, STOP** — you have broken legitimate enum constraints, which
   is a worse outcome than the defect.

## What the report must show

For **each** arm, side by side: **what core does** with the equivalent expression, and **what rete
now does**. That comparison is the deliverable — "it refuses now" is not. DESIGN carries core's
verdicts for both arms, driven; yours must match them in kind.
