# RULING — holon is for VSA/HDC. Period. And a WALL will say so, because the sentence already failed.

> **Builder, 2026-09-04:** *"we are waging an unrelenting assault on misuse of holon before we
> resume our registry onslaught... enemies in our ranks must be dealt with before we resume the
> frontal attacks... **holon may only be used for vsa/hdc things - period**"*

> Earlier the same day: *"holon-ast is hypervector of data … edn is a wire format of data … the
> data both have can be represented in either."* **Neither is a syntax tree.** `HolonAST` was a
> crutch taken while `WatAST` was immature.

## ⛔ THE SENTENCE ALREADY EXISTED, AND IT DID NOT HOLD

This is not a new rule. The substrate has been **printing it to users** from two sites:

```
src/types/error.rs:332          "use :wat::WatAST for any wat form, :wat::holon::HolonAST ONLY
src/function/parse.rs:1452       for a VSA/HDC algebra value, a named enum for closed
                                 heterogeneous sets, or parametric T/K/V for generics"
```

And `src/special_forms.rs` stored a special form's signature — a `(:head <slot> <slot>)` form — as
a `HolonAST::Bundle` anyway, for months, while that message shipped. `src/reflect/verbs.rs` then
copied the shape *deliberately*, its own comment saying: *"Built through the SAME HolonAST helpers
`special_forms.rs`'s `sketch()` used to — one shape, not a second hand-rolled one."*

★ **That is the whole lesson.** A rule stated in prose is the CONVENTION rung of the ladder, and a
convention does not merely fail to stop a violation — **it gets cited as the reason to propagate
one.** The crutch spread by consistency with itself.

## THE LADDER — where each rung stands today

| rung | form | status |
|---|---|---|
| 1 · convention | the sentence in two error messages | **SHIPPED, AND IT ROTTED.** Violated in prod code while printing. |
| 2 · a check at construction | `tests/lint/holon_is_vsa_only.rs` | **THIS RULING ARMS IT.** Measured: it can arm at ZERO. |
| 3 · no form | `HolonAST` unnameable outside the algebra | **NAMED, NOT REACHED.** See *The one door*, below. |

`crates/wat-reader` already stands on rung 3 by accident of the crate graph: it does not depend on
`holon-rs`, so `HolonAST` is unnameable there. Every `holon` string in the reader is a keyword
*literal* in a test. That is what rung 3 looks like, and it is why the reader has never drifted.

## THE WALL'S RULE — stated so it can be argued with

> Outside the **VSA homes** and the **one carrier**, no module may **CONSTRUCT** a `HolonAST`
> (`HolonAST::<ctor>(…)`) or **DECLARE** one in a field or return type (`x: HolonAST`,
> `-> HolonAST`, `Arc<HolonAST>`, `Vec<HolonAST>`).

**Pattern-matching an existing holon is ALLOWED**, deliberately: you can only match one that
somebody legitimately made, so a match arm is downstream of a construction the wall already
governs. **Naming it in prose is ALLOWED** — an error string listing `HolonAST` among hashable
types is documentation, not use. Both exclusions keep the wall aimed at the ACT rather than the
WORD, which is what lets it arm at zero instead of drowning in 26 lines of true negatives.

```
VSA HOMES        src/holon/**            the algebra
                 src/intrinsic/holon/**  its verbs
                 src/lower.rs            (:wat::holon::Bundle/Bind/Permute/…) → the algebra
                 src/record/update.rs    hologram field update by Bind/Bundle rewriting
                 src/edn/render.rs       the wire boundary — "the algebra never crosses the wire"

THE ONE CARRIER  src/value/value.rs      Value::holon__HolonAST(Arc<HolonAST>) and the two
                                         Hologram carriers. This is the single door by which a
                                         holon becomes a wat value. It is named EXPLICITLY, not
                                         lumped in with the homes, because it is rung 3's target:
                                         if the algebra is ever to be unnameable outside its
                                         homes, THIS is the declaration that must change.
```

## ⛔ THE EXEMPTION — a rune, and what its reason must earn

A site outside the homes may carry a co-located `// rune:lint(holon-not-vsa, <category>) — <reason>`.
The reason must name **why the holon is the subject**, never that it happens to be convenient.
"It round-trips losslessly" is NOT a reason — the `:wat::core::fn` arm round-tripped losslessly and
was still wrong, because losslessness is a property of the conversion, not a licence for the
detour.

## THE RESIDUE — the exact worklist that arms the wall at zero

Measured 2026-09-04 with a pattern **validated line-by-line** before its count was trusted (the
first attempt matched the `::HolonAST` inside the string `":wat::holon::HolonAST"` and reported 33
phantom offenders in `check.rs` alone — `[[feedback_validate_a_search_pattern_before_trusting_its_count]]`):

```
1  the special-form signature sketch        10 lines / 3 files
   [[DESIGN-STONE-the-special-form-sketch-is-syntax-not-a-hypervector]]  — DESIGNED, committed
     src/special_forms.rs:46,67,69,71,73,153 · src/reflect/verbs.rs:234,241,243
     src/reflect/lookup.rs:121

2  the stepper's :wat::core::fn arm          HolonAST::Atom used as a generic box
   [[DESIGN-STONE-stepvalue-is-watast-and-the-round-trip-is-lossy]]      — ✅ LANDED in-flight

3  require_bundle goes HOME                  src/runtime.rs:7388-7405 → src/intrinsic/holon/atom.rs
     Both of its callers are already there (atom.rs:1463,1514). It is a VSA helper misfiled into
     the runtime. Arc 109's own precedent, still commented three lines below it:
     "`require_ast_children` moved to `src/reflect/verbs.rs` … Behaviour unchanged."
     ⚠ Its error string reads "Bundle (signature head HolonAST)" — stale prose from when the
     sketch shared it, and it is what made this site read as misuse on first census. Fix the
     string in the same motion or the next reader re-raises it.

4  ONE rune                                  src/runtime.rs:20130, a #[cfg(test)] fixture building
     `HolonAST::symbol(":foo")` to exercise `value_to_watast`'s coercion arm. The holon IS the
     subject under test. Rune it; do not move it.
```

After 1–4 the offender count outside homes+carrier is **ZERO**, and the wall is a wall rather than
a campaign — `tests/lint/no_rc_use.rs`'s own doctrine: *"a lint raised at zero is a wall, a lint
raised at 1306 is a campaign."*

## ✅ WHAT THIS RULING DOES NOT TOUCH

The `.wat` corpus is healthy and was measured, not assumed. Its `:wat::holon::` vocabulary is the
algebra doing its job — `to-holon` 1973 · `leaf` 257 · `Hologram` 123 · `OnlineSubspace` 109 ·
`Bind` 91 · `Reckoner` 76 · `encode` 68 · `EngramLibrary` 66 · `Thermometer` 59 · `Bundle` 58 ·
`cosine` 37 · `coincident` 24 · `simhash` 18 · `Blend` 18. **The rot was in the Rust internals,
exactly where the builder said the enemy was.** The wall's scope is `src/` and `crates/*/src/`.

★ And the VSA itself is not on trial. The builder: *"the vsa properties are assumed to be correct
as we have used it extensively to solve hard problems."* This ruling removes things that were never
VSA from a place reserved for VSA. It changes no vector, no bundle, no cosine.

## THE FOUR QUESTIONS — on the WALL, flat YES/NO

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **arm `holon_is_vsa_only` at zero** | YES | YES | YES | YES |

- **Obvious? YES** — the rule is one sentence the substrate already prints; the wall only makes it
  answerable.
- **Simple? YES** — one lint file, one regex pair, a homes list, a rune escape. The established
  shape of thirty-three siblings in `tests/lint/`.
- **Honest? YES** — and it is the point. The prose rule made a claim nothing could check, and the
  claim was false for months. A wall that can go red is the first version of this rule that can be
  wrong out loud.
- **Good UX? YES** — a violation names its own file and line at the floor, at the moment it is
  written, instead of surfacing as a corrupted rational eleven months later.
