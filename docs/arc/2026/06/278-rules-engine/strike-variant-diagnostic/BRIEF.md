# BRIEF — name the mistake: a variant that does not exist, not a field that never could

D1 made a misspelled enum variant in a rete constraint REFUSE. It refuses by falling through to the
`UnknownField` machinery, so the message reads *"`:evt::Req` has no field `:evt::G::Hii`; available
fields: [k, grade]"* — pointing the author at fields when they mistyped a **variant**. Split the
`_ => "keyword"` arm so a `::`-qualified name whose prefix is a known enum gets its own refusal,
naming the enum and the variants that exist. Read `DESIGN.md` beside this file first — it opens with
a correction to my own breadcrumb (the kind I named does not exist, and "agree with core" is the
wrong target because core does not name it either), and its "out of scope" cuts two shapes.

## Read in order

1. `src/rete/validate/typing.rs:225-236` — `keyword_constant_segment` and the doc paragraph above
   it. The `_ => "keyword"` arm is the site; the paragraph is true about the LOCATION and silent
   about the message.
2. `src/rete/validate/typing.rs:465-477` — the caller. Note the recorded history in its comment:
   this used to return `UnboundInThisRule`, which is why the constant "could only ever be reported
   as a missing field". You are finishing that same repair.
3. `src/rete/matcher.rs` — `enum_variant_ctor`, the ONE resolver D1 routed this through. Your split
   asks it a second question, not a different question.
4. `src/rete/validate/error.rs` — `UnknownField`: how a located rete refusal carries its remedy
   list (`available-fields`). Yours carries `available-variants`. Same file, same shape.
5. `tests/rete/probe_arc278_enum_variant_typo.rs` + its three `.wat` fixtures — D1's drive, already
   in the tree. These are your fixtures; the `_bad` one is the misspelling and the `_tagged` one is
   the payload-variant case.

## Sketch

```rust
// typing.rs — the third state, which today is spelled "keyword"
match enum_variant_ctor(types, k) {
    Some((_, _, 0)) => "enum",
    _ if prefix_of(k).is_some_and(|p| matches!(types.get(p), Some(TypeDef::Enum(_)))) => {
        // the diagnosable mistake — the prefix names an enum, the variant does not exist
    }
    _ => "keyword",
}
```

The caller returns `OperandType::Resolved(..)`, so decide there how the refusal is raised — read the
caller before choosing whether the split returns a richer type or the caller consults the same
predicate.

## Blast radius

`src/rete/validate/typing.rs`, `src/rete/validate/error.rs`, probes. Nothing else.

## Traps named in advance — each with its step

1. **A payload variant is not a unit variant.** `enum_variant_ctor` returns arity; `Some((_,_,0))`
   is the unit case D1 pinned. A correctly-spelled PAYLOAD variant must not become your new error.
   **Step:** the `_tagged` fixture exists precisely because that path resolves differently — drive it
   and say which arm it lands on.
2. **A genuine keyword constant must stay a keyword.** `:alpha` with no `::`, or a `::` name whose
   prefix is not an enum, are legitimate. **Step:** the control fixture
   (`probe_arc278_enum_variant_typo.wat`) is the guard; it must stay green, and it is the
   anti-vacuity control — without it, a refusal that fires on every keyword would look correct.
3. **Do not delete the `UnknownField` route.** Other constants legitimately reach it. **Step:** your
   arm is narrower — prefix-is-a-known-enum only — and everything else keeps its current path.
4. **The remedy list must be the variants, not the fields.** The whole finding is a confidently
   wrong remedy. **Step:** assert the message CONTAINS the real variants, exactly, and assert it does
   NOT offer field names.
5. **New test code trips `wat::lint`.** **Step:** run
   `cargo nextest run --release -E 'binary_id(wat::lint)'` before reporting; prefer exact
   `assert_eq!` over `contains` on deterministic values. This has caught a red for three consecutive
   riders.
6. **A probe whose subject fails to type-check cannot live under `wat-scripts/`.** That tree is
   parsed AND type-checked by `every_wat_scripts_file_loads`. **Step:** deliberately-failing wat goes
   beside the other probes in `tests/rete/`, the way D1's `_bad` fixture already does.

## STOP triggers

- **STOP-1** — if the caller's `OperandType::Resolved(&'static str)` shape cannot carry a refusal
  without a wider change, STOP and report what it would take. The radius says two files.
- **STOP-2** — if any currently-green test goes red, STOP and report which.
- **STOP-3** — if the tagged path does not reach your new arm, STOP and report where it lands
  instead. DESIGN cuts guessing about it.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-silent-zero/` — A2b, the same split-by-type cure on the
same class of catch-all, in the same arc.

## The one thing worth more than the fix

**Tell me where this brief was thin.** Fifteen riders before you each returned a prescription of
mine that did not survive contact. The last found that a recorded measurement in a fixture — *"only
this arm can see it"* — had been false since the day it shipped, masked by a cache that landed in the
same commit. It re-drove it rather than trusting the comment. If a step here is wrong, unnecessary,
or impossible, say it plainly.
