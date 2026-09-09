# NOTE — P-2 is blocked on VARIANCE, and my fence was drawn on the wrong axis

**Measured 2026-09-08**, arc 296 stone P-2a. The registration half was built and is preserved at
`7292a2c2a`; it was reverted because the DESIGN's contract is *both halves or neither* and only one
half is reachable today. This NOTE records why, so the next attempt does not re-derive it.

## What the ctor half costs today: 299 type-check errors, in two clusters

**Cluster 1 — `send` unifies, it does not subsume.**

```
:wat::kernel::send: parameter payload expects :wat::query::Store::Op;
                    got :wat::query::Store::Op::ScanIndex
  wat/query/sqlite-store.wat:243
```

`infer_send_prime` (`check.rs:11619`) **unifies** the payload against `I`. It never calls
`assignable`, so a subtype payload is refused. Ordinary function parameters DO consult
`assignable` → Path-Path `is_subtype` (`check.rs:16992`) — the same door arc 209's marker bound
uses. **Two argument positions, two different rules**, and only one of them subsumes.

**Cluster 2 — parametric type arguments are INVARIANT.**

```
if-branch:  RecvOutcome :- [ScanResponse::RequestTooLarge]
else:       RecvOutcome :- [ScanResponse]
```

`Variant <: Enum` does not lift through a container: `RecvOutcome<Variant>` is not a subtype of
`RecvOutcome<Enum>`. **That is variance**, and nothing in the type system currently declares it.

## ★★★ MY FENCE WAS ON THE WRONG AXIS

I fenced generics out by fencing **generic ENUMS** out, reasoning that a monomorphic enum has no
type arguments and therefore no argument-correspondence problem. Cluster 2 is a **monomorphic**
variant — `ScanResponse::RequestTooLarge` — and it hit the correspondence class anyway, because it
was *wrapped* in a generic container.

> **I measured the genericity of the DECLARATION and never asked about the genericity of the
> POSITIONS its values flow through.** A containment argument must name its consumers.
> `[[feedback_a_containment_argument_must_name_its_consumers]]`

The fence was real and it held for what it described — `generic_variant_stays_refused` never moved.
It simply described the wrong boundary.

## ⛔ AND MY PROBE PASSED IN THE FORBIDDEN STATE

All five rows stayed green with the registration half landed and the ctor half reverted — the exact
configuration the DESIGN called *"strictly worse than today's refusal."* The rider found this, not
me:

> *"The probe's five bars still pass because they never construct a value into a variant-typed
> parameter."*

The missing row is one line, and it is the only row that could have caught it:

```wat
(:user::takes-red (:usr::Colour::Red {:shade 7}))
  ->  ":user::takes-red: parameter #1 expects :usr::Colour::Red; got :usr::Colour"
```

You write `Colour::Red`, construct `Colour::Red`, and are told you supplied a `Colour`. **A probe
that proves a type is USABLE must construct a value INTO it**, not merely annotate with it.
`[[feedback_an_acceptance_row_a_defect_can_satisfy_is_not_a_row]]` ·
`[[feedback_a_green_test_can_prove_nothing]]`

## What P-2 actually needs, in order

```
1  ONE rule for argument positions      send-unify vs parameter-assignable is a real split. Any
                                        subtype work is unreliable while two positions disagree,
                                        and this is worth settling on its own merits.
2  A VARIANCE story                     Variant <: Enum must lift through containers, or every
                                        generic wrapper re-erases it. Declared variance, or a
                                        subsumption rule for type args — a type-system decision,
                                        the builder's to rule.
3  THEN the ctor stops erasing          and P-2 becomes the small stone the old seam thought it was.
```

⚠ **This supersedes the seam's "4/4 on the four questions, unblocked."** P-2 is not unblocked; it
is blocked on a type-system decision nobody has made. The four questions were run on a shape whose
consumers had not been measured — which is the same defect as the fence.

## What stands

The loud refusal is restored: `[c <- :usr::Colour::Red]` → `UnknownNamedType`, naming the variant.
That is P-1 working, and it remains strictly better than an annotation nothing can satisfy.
