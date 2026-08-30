# DESIGN — STONE layer-3: `from-holon`'s 3-arg hint is DEAD. Cut it.

> **Builder, 2026-08-30:** *"this registry work is forcing the removal of bad practices... you've
> found one... so we continue our clean up as we build the registry."*

## The finding

`:wat::holon::from-holon` accepts a 3-arg form, `(from-holon h -> (:wat::core::HashMap :- [K V]))`.
**Its parsed result is discarded.**

```
atom.rs:158   let _hint_is_hashmap = if args.len() == 3 { … 40 lines of validation … }
              ^ assigned once. NEVER READ. The underscore keeps the compiler quiet.
```

Those 40 lines are not inert — they validate: a non-`->` second argument raises, a wrong type
keyword raises. So the form is **parsed, policed, and then thrown away**.

## Four witnesses that it is dead, not merely unused

**1. The doc block promises a behaviour the code does not implement.**
> *"`(from-holon h -> (:wat::core::HashMap :- [K V]))` disambiguates an empty `Map`-classified
> Bundle, which is otherwise shape-indistinguishable from an empty `Set`/`Vector`/`List`/`Tuple`."*

**2. The function contradicts itself, 29 lines apart.**
```
:127   "The `-> (HashMap :- [K V])` consumer-hint form is preserved for empty-Map classifier."
:156   "Empty Map always returns empty HashMap regardless of hint."
```
The second is correct. **Arc 228 Stone 228.1's classifier-dispatch replaced arc 216's heuristic
Bundle dispatch** — a `Map`-classified Bundle now carries its own classifier, so an empty one is
identified structurally and the premise of the hint ("shape-indistinguishable") is FALSE.

**3. The checker calls it decoration in its own words.** `check.rs:3706`:
> *"The `->` and type keyword are **syntactic decoration**; return type is still T (Infer)."*

**4. Zero callers.** 11 `from-holon` call sites across `wat/`, `wat-tests/`, `wat-scripts/`.
**None uses the 3-arg form.**

## What this is an instance of

A feature whose *premise* was removed by a later arc, while its *surface* survived because nothing
forced the question. Arc 228 correctly replaced the mechanism; nobody asked what the old mechanism's
entry point was still doing. The registry work forced the question by making someone read the body.
`[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`

## The cut

```
atom.rs        the ~40-line `_hint_is_hashmap` block DELETED; arity becomes exactly 1
atom.rs        the doc block's disambiguation paragraph DELETED (it describes a dead feature)
atom.rs:127    the stale "consumer-hint form is preserved" comment DELETED
check.rs       the `args.len() != 1 && args.len() != 3` fork becomes a plain arity-1 check,
               and its 6-line comment block describing the 3-arg form goes with it
```

★ **This stone touches `src/check.rs`, and every prior wave forbade that.** Deliberate: the checker
carries a special case for the form being retired, and leaving it would mean the checker still
admits a shape the runtime rejects — a worse state than either endpoint.

## The one contract decision, pinned

**The retired form's diagnostic is the ordinary arity error, not a bespoke one.** After the cut,
`(from-holon h -> T)` reports `expected 1, got 3` from the standard machinery. No special-cased
"this form was retired" message, because no caller exists to receive it and a bespoke diagnostic
for a form nobody uses is the vapor this house cuts.

⚠ **Whether `RETIREMENT_TABLE` (`src/remedy/retirement.rs`) needs a row is an OPEN QUESTION the
rider must answer, not assume.** Its doc says *"HARD CUT stones append entries at ship time"*, but
its rows map a retired **name** to a replacement **name** (`:wat::core::struct` →
`:wat::core::defstruct`). This retires an **arity variant** of a verb that survives. If the shape
does not fit, the answer is "no row, and here is why" — not a row forced into a schema it does not
belong in.

## Out of scope = REJECTED

- **The 250-line split of `from-holon`'s decode.** That is the NEXT stone, and it is cleaner once
  this dead limb is gone. Cutting first means that stone does not carefully relocate code we have
  proven nothing calls.
- **No other verb in `atom.rs`.**
- **No change to the classifier dispatch** — arc 228's mechanism is correct and is what made the
  hint redundant.

## Calibration

Predicted 25–40 min. Small, but it crosses into `check.rs` for the first time this campaign.
