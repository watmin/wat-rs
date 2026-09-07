# DESIGN — STONE M: an enum variant is constructed by a MAP, and only that way

## WHY

A variant is named in three places. **Two of them already agree, and construction is the holdout:**

```
WIRE       #probe/Box.Full {:payload 7}       map, named    296 H
PATTERN    [:probe::Box::Full {:payload p}]   map, named    the arm migration
CONSTRUCT  (:probe::Box::Full 7)              POSITIONAL    <- the last one in the language
```

`defrecord` already made this move — measured, four cells:

```
                (Pt :x 1 :y 2)   (Pt {:x 1 :y 2})   (Pt 1 2)
defrecord           check=0          check=1         check=1   <- positional REFUSED
```

So **enum variant construction is the last positional constructor in wat.** This stone is the
same position→name migration the wire made, then the patterns made, arriving one layer down —
the third instance of that one shape in a single arc.

## WHAT IT DELIVERS

```
(:probe::Box::Full  {:payload 7})     a payload variant, field NAMED
(:probe::Box::Empty {})               a unit variant, empty map
(:wat::core::Option::Some {:value 1}) Option/Result are ORDINARY enums — no special case
(:probe::Box::Full 7)                 REFUSED
```

Field names come from the declaration, in declaration order — `:wat::runtime::type-of` already
answers exactly that (296 L). **This is L's third payoff**, after the match-arm codemod and the
rete IR.

## ⛔ THE ONE CONTRACT DECISION, PINNED

**The map NAMES FIELDS; it is never the payload.** `(Option::Some {:value 1})` types as
`Option<i64>`, NOT `Option<HashMap<keyword,i64>>`.

That distinction is the whole stone and it is invisible in an untyped slot. Today, inside
`(:wat::core::show …)` — which accepts anything — the map form ALREADY checks clean, because the
map is swallowed as the payload:

```
(:user::pick (:wat::core::Option::Some {:value 1}))     where pick takes Option<i64>
  :user::pick: parameter #1
    expects (:wat::core::Option :- [:wat::core::i64])
    got     (:wat::core::Option :- [(:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])])
```

★ The orchestrator wrote the untyped probe first, read the green, and told the builder "no new
machinery is needed — this is a pure refusal stone." **That was false**, and only a TYPED slot
could show it. Every fixture in this stone's probe puts the construction in a typed position for
that reason. `[[feedback_a_green_from_a_mis_aimed_probe_is_indistinguishable_from_a_working_gate]]`

## ROOMS

```
src/declare/register.rs:1296   register_enum_methods — mints per-variant constructors TODAY,
                               positionally. `EnumVariant::Unit` -> sym.register_unit_variant;
                               payload variants -> a function whose return type is
                               parametric_decl_type(enum name, params)  (:833) — i.e. the ctor
                               returns the ENUM type, never a variant type. Read :1270's header.
src/check.rs:13951             infer_kwargs_construct_check — arc 294 item (C), the LIVE
                               kwargs-construction inference for defrecord/defstruct. THE
                               PRECEDENT: same mechanism, different arg shape (a map, not flat
                               kwargs). Read :13726 and :13833 first — :13833 states exactly
                               where it deliberately does NOT mirror the sibling path.
src/check.rs:4690              the `:wat::core::kwargs-construct` dispatch arm
src/match_arm.rs:159           builtin_variant — the FOUR hardcoded Option/Result exceptions.
                               NOT this stone's to delete (that is the bare-name retirement),
                               but read it: it is why Option behaves unlike a user enum today.
```

## OUT OF SCOPE — AFFIRMATIVELY CUT, NOT DEFERRED

- **The corpus migration.** 2,209 Option/Result sites + user-enum ctors move in a LATER stone by
  wat-fix. This stone builds the form and refuses the old one; a corpus that cannot be migrated
  onto a form that does not exist is the failure this ordering avoids.
- **The bare-name retirement** (`:wat::core::None` -> `:wat::core::Option::None`) and the four
  `builtin_variant` special-cases. Its own stone, behind this one.
- **`{:keys …}` on `defrecord`.** A measured live asymmetry — `defstruct` gets `:keys`, `defrecord`
  does not, and nothing justifies the split (arc 257.2's probe only ever exercised defstruct).
  Ruled by the builder as ITS OWN STONE, not folded here.
- **Variant-as-a-type.** `(:usr::Shape::Rectangle 1 2)` has type `:usr::Shape`; the checker says
  so in as many words — *"struct-destructure (x y) expects a struct type; got :usr::Shape"*. Making
  a variant a type would let `defclause` (72 sites, already the open-surface router) dispatch per
  variant. That is a DISPATCH answer via an existing entity kind, not a type-system reach — a real
  stone, and NOT a prerequisite for this one: construction NAMES its variant explicitly, so it is
  sound whichever way that lands.

## THE FOUR QUESTIONS

- **Obvious?** YES. One shape in all three places, per kind.
- **Simple?** YES. One rule: a variant's arguments are its declared fields, named.
- **Honest?** YES. It deletes the exception rather than adding a case — `Option` stops being
  special, and the positional form stops existing rather than being discouraged.
- **Good UX?** YES. The ctor now reads exactly like the pattern that destructures it and exactly
  like the wire that carries it.
