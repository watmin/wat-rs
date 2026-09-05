# DESIGN — STONE: the special-form signature sketch is SYNTAX. It stops being a hypervector.

> **Builder, 2026-09-04:** *"we are not debating how to use holon correctly - we are ripping out
> the misuse of holon for non-vsa things - the vsa properties are assumed to be correct as we have
> used it extensively to solve hard problems.. it was misused as an edn holder until wat-ast
> matured.... my ask is simple - annihilate holon use in places that are not doing vsa/hdc things"*

This stone is the whole of that ask, measured. A full census of `HolonAST` across `src/` found
**one** misuse population. This is it.

## THE CENSUS — the predicate first, because a fuzzy predicate is how a census goes wrong

> **MISUSE** — `HolonAST` is built from, or converted to, something that is NOT a hypervector: it
> is carrying EDN/AST data on a path with no bundling, no binding, and no similarity in it.
>
> **LEGITIMATE** — `HolonAST` appears because it IS a first-class value of the language: its
> `Value` variant, its type rows, its renderer, the algebra in `holon/`, the verbs in
> `intrinsic/holon/`.

Every file holding a `HolonAST` code site, classified (comment-only lines excluded):

```
✅ LEGITIMATE — the algebra, its verbs, and the plumbing that carries a holon AS A VALUE
   src/holon/ast.rs 139 · src/intrinsic/holon/atom.rs 127 · src/holon/hologram.rs 30
   src/intrinsic/holon/hologram.rs 14 · src/intrinsic/holon/reckoner.rs 3   the algebra + verbs
   src/lower.rs 21          (:wat::holon::Bundle/Atom/Bind/Permute/Thermometer/Blend) → algebra
   src/record/update.rs 12  hologram field update by Bind/Bundle rewriting — role-filler binding
   src/edn/render.rs 22     rendering a holon VALUE to the wire; its own words:
                            "the algebra never crosses the wire"
   src/check.rs 35          TypeExpr::Path rows that type-check the holon VERBS
   src/types.rs 15          type-system plumbing + tests
   src/runtime.rs 38        VSA verb eval, tests, error-string prose  (ONE open question, below)
   src/collection/eval.rs 6 error-string prose naming HolonAST as a hashable type
   src/value/value.rs 11    the Value variant itself
   singles: freeze.rs · closure_extract.rs · function/subsume.rs · function/parse.rs ·
            value/observe.rs · types/error.rs · resolve/mod.rs · rete/vocabulary.rs ·
            intrinsic/special/holon_literal.rs · macros/tests.rs

⛔ MISUSE — ONE population: the special-form signature sketch
   src/special_forms.rs:46       pub signature: HolonAST              in SpecialFormDef
   src/special_forms.rs:67-73    fn sketch(head, slots) -> HolonAST
   src/special_forms.rs:152      match &def.signature { HolonAST::Bundle(children) => ... }  (test)
   src/reflect/lookup.rs:31,121  use holon::HolonAST; · signature: HolonAST   the mirrored field
   src/reflect/lookup.rs:267     signature: def.signature.clone()
   src/reflect/verbs.rs:234-243  the SAME shape, hand-built, then immediately un-built
```

★ **The substrate already ships this ruling to users.** `src/types/error.rs:332` and
`src/function/parse.rs:1452` both say, verbatim: *"use `:wat::WatAST` for any wat form,
`:wat::holon::HolonAST` **ONLY** for a VSA/HDC algebra value."* `special_forms.rs` is what
violates the message the substrate prints.

## ⛔ TWO FALSE ALARMS — recorded so the next census does not re-raise them

1. **`runtime.rs:7399`, `require_bundle`.** Its error string reads
   `"Bundle (signature head HolonAST)"`, which makes it read as the sketch destructurer. It is
   not. Its **only two callers are `src/intrinsic/holon/atom.rs:1463,1514`** — the VSA verbs. The
   function is legitimate; the string is stale prose from when the sketch shared it. Fixing the
   string is IN scope (it is the thing that would mislead the next reader), the function is not.
2. **`edn/render.rs`, `lower.rs`, `record/update.rs`** each looked suspicious by name and are each
   pure VSA on inspection. A file name is not a classification.

## THE SHAPE — and it is PROVABLY byte-identical

`reflect/verbs.rs` builds the holon **only to convert it straight back**:

```rust
let sketch = holon::HolonAST::bundle(children);
Ok(... Value::wat__WatAST(Arc::new(holon_to_watast(&sketch))))
```

Its own comment names the motive — *"Built through the SAME HolonAST helpers `special_forms.rs`'s
`sketch()` used to — one shape, not a second hand-rolled one."* **The crutch propagating itself for
consistency's sake.** The output was always a `WatAST`; the hypervector was a detour.

The three arms compose to identity, which is why the replacement cannot change a byte:

```
sketch(head, slots)                                   holon_to_watast (src/holon/ast.rs)
  bundle(children)        ──►  HolonAST::Bundle  ──►  WatAST::List(children, span)
  keyword(head)  STRIPS ':'──►  HolonAST::Keyword ──►  WatAST::Keyword(format!(":{}", s), span)
                                                       ("restore leading colon for round-trip")
  symbol("<name>")        ──►  HolonAST::Symbol  ──►  WatAST::Symbol(Identifier, span)
```

The leading colon is stripped on the way in and restored on the way out. That dance exists ONLY
because the value detoured through a representation that spells keywords differently. Remove the
detour and the dance goes with it.

```
SpecialFormDef.signature            HolonAST  ──►  WatAST
Binding::SpecialForm { signature }  HolonAST  ──►  WatAST
special_forms.rs::sketch            builds WatAST::List directly
reflect/verbs.rs                    builds WatAST::List directly; `holon_to_watast` call DELETED
```

## THE ONE CONTRACT DECISION

**The emitted `WatAST` must be byte-identical to what the holon round trip emits today.**
Concretely: the head is `WatAST::Keyword` **with** the leading colon (`WatAST::Keyword` stores it —
`crates/wat-reader/src/ast.rs:119`, and `edn/render.rs:818` pushes `k` verbatim); each slot is a
`WatAST::Symbol` carrying an `Identifier`; the whole is a bare `WatAST::List`, NOT a
`(:wat::holon::Bundle ...)` call. Spans are synthetic — use `crate::rust_caller_span!()`, which is
what `holon_to_watast` already stamps on every node it builds.

## THE FOUR QUESTIONS — flat YES/NO

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **the sketch is a `WatAST::List`** | YES | YES | YES | YES |

- **Obvious? YES** — the stored value is `(:head <slot> <slot> ...)`. That is a form. The type
  named `WatAST` is for forms; the substrate's own error message says so.
- **Simple? YES** — one field type, one enum field, two build sites, one destructure, one test.
  Two helper functions become one, and a conversion call disappears.
- **Honest? YES**, and it is the whole stone: nothing is ever bundled with this value, bound to
  it, or compared to it by cosine. Calling it a hypervector is a claim the code never cashes.
- **Good UX? YES** — a reader of `special_forms.rs` currently has to know the holon algebra to
  read a signature table.

## Scope

**In:** `SpecialFormDef.signature` · `Binding::SpecialForm.signature` · `special_forms.rs::sketch`
· `reflect/verbs.rs`'s hand-built sketch and its `holon_to_watast` call · the destructuring test
at `special_forms.rs:152` · `require_bundle`'s stale `"signature head"` error string · whatever
rustc names.

**Out, affirmatively:** every ✅ row of the census above — the algebra, its verbs, the renderer,
the type rows, the `Value` variant. This stone removes a misuse; it does not touch VSA.

**Out, because a SIBLING STONE took it — `src/runtime.rs:12525`, the stepper's `:wat::core::fn` arm:**

```rust
":wat::core::fn" => {
    let h = watast_to_holon(&form);
    Ok(StepValue::Terminal(holon_to_watast(&HolonAST::Atom(Arc::new(h)))))
}
```

I first filed this as a ruling for the builder, reading its comment — *"so cosine / hash / cache
keys see it as a single coordinate"* — as a VSA rationale. **The builder refused the framing:**
*"this is bullshit - this makes no sense... holon's atom is a lisp quote... it holds arbitrary
things..."* He is right, and the source says so plainly:

- `holon-rs/src/kernel/holon_ast.rs:10` — *"**Holder** (arc 225): `Atom(Arc<HolonAST>)`. **The
  algebra's quote** — minimal holder, repeatable holds compose."*
- `:85` — *"`Atom(Atom(x))` differs from `Atom(x)` differs from `x` — quote-wrapping is repeatable
  and meaningful (Lisp's `'(quote x)` ≠ `'x`)."*
- `src/intrinsic/holon/atom.rs:456` — `@arg h :wat::holon::HolonAST` → `@ret
  :wat::holon::HolonAST`. Atom takes a holon and returns a holon.

`Atom` is quote **inside the algebra**. The arm lowers syntax into the algebra, quotes it there,
and lowers it back out — and the CEK stepper never bundles, binds, or takes a cosine of anything.
The comment describes what `Atom` does when something LATER encodes it; nothing here encodes. It is
the algebra's quote used as a generic opaque box for arbitrary syntax, which is this stone's exact
subject. The observable consequence is the rational's defect one arm over:
`(:wat::eval-step! (quote (fn [x] x)))` returns `(:wat::holon::Atom (fn [x] x))` — a form the
stepper did not receive.

It is handled by `[[DESIGN-STONE-stepvalue-is-watast-and-the-round-trip-is-lossy]]`, whose rider
already owns `runtime.rs`; the arm becomes
`Ok(StepValue::Terminal(WatAST::List(items.to_vec(), list_span.clone())))`. Recorded here because
this document is the census, and a census that omits a site because a sibling took it is a census
with a hole.

⚠ **How I got it wrong is the lesson, not the site.** I read a comment as an authority instead of
checking the type it described — the EIGHTH comment-caused error of this campaign, committed in the
same message where I listed the first seven.
`[[feedback_an_adjacent_implementation_is_not_the_subject]]` ·
`[[feedback_a_header_is_not_the_file]]`
