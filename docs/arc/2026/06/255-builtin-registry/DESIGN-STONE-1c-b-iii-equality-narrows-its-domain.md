# DESIGN — STONE 1c-b-iii: `=` narrows its domain, the way `<` already did

> **Builder, 2026-09-03:** *"long term... we will use the Partial labels... to hunt down everything
> who is not total... wat will be made total... we will consult this registry for heretics who
> demonstrate lack of totality."*
>
> `wat/runtime-meta.wat` already says the same of its own `:Partial` variant: *"★ THIS VARIANT IS
> THE WORK LIST: the totality endgame's census is `all_entries().filter(|e| e.totality == Partial)`."*

## ⛔ THREE PROPOSALS DIED ON ONE READ OF THE AXIS'S OWN PROSE

This stone was very nearly a sixth property. It is not. `wat/runtime-meta.wat:217`:

> *"Totality — is the verb **DEFINED ON EVERY INPUT in its DECLARED DOMAIN**?"*
> *":Partial — Undefined somewhere in its **declared domain**."*

**The axis was already domain-relative.** It never asked "is this verb total in the abstract."
So `=` is not Partial because the axis is too coarse — **`=` is Partial because its declared
domain is too wide.** It claims `∀T` and is defined only on the comparable subset.

What died, and why, so none of it is re-proposed:

- **a `:TypeIndexed` pole** (proposed by the `intueri` cast) — unnecessary; the axis already
  expresses domain-restriction, and the ward's own caveat said the label without a predicate
  behind it repeats `:Unreviewed`'s broken promise.
- **`properties_of(name, arg_types)`** — unnecessary; `<` already gets the right answer with no
  such interface, because its *domain* is right.
- **"purity varies by operand type too"** — **false, and mine.** `constructor_meta(head, sym)`
  takes the HEAD; the head IS the type. A record's wire-representability is a fixed property of a
  fixed type, not a per-call variance. Retracted.

## The measured ground — one gate is the whole difference

```rust
infer_ordering   unify(a,b) STRICTLY  →  is_type_orderable(t)     TypeExpr::Fn { .. } => false
infer_equality   unify-or-subtype-or-numeric  →  (NO domain gate)
```

`is_type_orderable` (`check.rs:12868`) narrows `<`'s declared domain to exactly what
`values_compare` handles. `--check` rejects `(< <fn> <fn>)`. **No undefined point remains in the
declared domain, so `@Totality Total` is honest** — and that is precisely the grade Stone 1c-b-ii
landed for all four orderings.

`infer_equality` (`check.rs:12817`) tests only whether the two types RELATE (unify / subtype /
both-record / both-numeric). It never asks whether the type is equatable **at all**. So `∀T` stays
the declared domain, `Fn` is inside it, and `values_equal` (`runtime.rs:5302`) returns `None`
there — which `eval_eq` raises. Proven by a committed counterexample,
`wat-scripts/scratch-pad/probe-core-eq-is-partial.wat`.

**Same family, same engine shape, opposite totality — entirely because one narrowed its domain and
the other did not.**

## The stone

Build **`is_type_equatable`**, the sibling `is_type_orderable` already proves the shape of, mirror
its allow-list against what `values_equal` actually handles, and gate `infer_equality` on it. Then
grade `=`/`not=` with whatever that measurement supports, retire `intrinsic_meta`'s by-name
placeholder, and land the two rows held by
`[[NOTE-equality-is-argued-proven-partial-and-held]]`.

## ★★★ THE RISK, AND IT IS THE STONE'S REAL CONTENT

`wat/test.wat:61`:

```
(:wat::core::defn :wat::test::assert-eq :- [T] [actual <- :T expected <- :T] -> :wat::core::nil
```

**The test framework's own assertion primitive compares two bare type variables with
`:wat::core::=`.** A domain gate that rejects an unresolved type var does not fail one file — it
fails `assert-eq`, and every test that uses it.

`is_type_orderable` already ruled this case: `TypeExpr::Var(_) => true, // unresolved — defer to
runtime`. An equality sibling must rule it too, and the ruling decides what `=` can honestly be
graded:

```
if a type VAR is admitted   → every CONCRETE call site is gated and total;
                              a GENERIC body still defers to the runtime backstop
                            → is `=` then `Total`, or `Total`-at-concrete-sites-only?
                              ⬜ THIS MUST BE MEASURED, NOT ASSUMED.
```

⚠ This is the one place the builder's two cures for partiality genuinely diverge. *Narrowing the
domain* cannot narrow past a type variable. *Forcing the failure into a matchable shape* can — and
that is arc 233's thesis, out of scope here but the honest alternative if the measurement says the
hole survives.

## Acceptance — DERIVED, and deliberately CONDITIONAL

```
is_type_equatable exists, mirroring is_type_orderable's shape and values_equal's real arm set
infer_equality gates on it
(:wat::core::= <fn> <fn>)   --check REJECTS   ← today it exits 0; the probe flips to a compile error
wat/test.wat's assert-eq    still compiles    ⬅ THE LOAD-BEARING ROW
the four rete/sift fixtures  pass UNEDITED    ⬅ if they need editing, the gate answered wrong
=/not= registered            grade per the measurement, NOT per this design's expectation
floor                        5128/5128
```

★ **`=`'s grade is deliberately not predicted here.** Two acceptance tables in this campaign were
wrong because they stated an expectation instead of deriving a bar; this one names the measurement
and refuses to pre-empt it. `Total` if the domain gate closes the hole; `Partial` if the type-var
door leaves it open — and either is a shippable, honest result.

## Out of scope — CUT

- The eight rete rows `=`/`not=` unblock. Their own stone, immediately after.
- Arc 233's "make the failure matchable" cure. Named as the alternative if the hole survives.
- Any new axis, pole, or registry interface. All three proposals are dead above.
