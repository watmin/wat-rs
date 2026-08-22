# DESIGN — the type registry holds the BUILTIN types

**Status: written 2026-08-22 against `8cc8c9a30`. Builder ruled A: *"we build it and fix the
checker."* The representation question below is the one thing still open.**

Arc 255's thesis is ONE resolution path. This stone is the half 255's own DESIGN does not cover.

## ⛔ First — the correction that scopes this stone

255's DESIGN is about **callable** builtins: the 454-arm dispatch `match`, `:wat::core::i64::+`,
`length`, `send`. Slice **255.1** registers those into `sym.functions` and deletes the reserved-prefix
blanket-accept for CALL heads. Every example in that DESIGN is a verb.

**Builtin TYPE names are a different population and 255 does not scope them.** Measured:

```
:wat::core::i64      registered as a TypeDef .......... 0 occurrences
:wat::core::String   .................................. 0
:wat::core::Vector   .................................. 0
Sender · Receiver · Peer · Address · Instant ......... 0 each
register_builtin(…) calls in register_builtin_types ... 36, ALL aggregate error/outcome records
```

`255/NOTE-a-capability-declaration-cannot-be-verified-to-name-anything.md` lists three consumers
waiting on "the same membership half", one of them `109/NOTE-type-annotation-names-unchecked.md` —
the type-annotation gap. **That note's consumer is not actually served by 255.1**, because 255.1
populates `sym`, and a type name is not a callable. This stone is that missing half, and it is
smaller than 255.1.

★ **The reframe: the door is not missing, it is UNDER-POPULATED.** `TypeEnv::contains` is the right
door and `register_builtin_types` is the right home — it has simply never held primitives,
containers, or opaque capability types. My earlier DESIGN
(`109/DESIGN-STONE-a-type-reference-must-resolve.md`) asserted `contains` "already covers builtins"
from the function's NAME. It does not, and that error is what produced a hand-written closed-world
list in a rider's diff — the exact "second resolution path" 255's note warns against.

## The population — small, enumerable, and already half-written down

```
crate::check::BARE_PRIMITIVES ......... 5    the checker's own source of truth
crate::check::BARE_CONTAINER_HEADS .... 7    ditto
opaque capability / handle types ..... ~15   Sender, Receiver, Peer, Address, Instant, …
scalar/AST leaves ..................... ~7   bigint, rational, keyword, HolonAST, WatAST, Value, Never
```

Roughly thirty names. Two of the four groups are already consts the checker consults, so this is
mostly a matter of registering what is already enumerated, not inventing an enumeration.

## THE DECISION — how does the registry represent a builtin type?

`TypeDef` has six variants (Aggregate · Enum · Newtype · Alias · Union · Surface) and `Nature` has
four (Struct · Record · HolonRecord · Peer). **A primitive is none of them.** Grounded counts:
`TypeDef::` appears at **311** sites; `TypeEnv::contains` has **10** callers; and **zero** sites do
contains-then-unwrap-`get`.

| # | option | Obvious | Simple | Honest | Good UX | verdict |
|---|---|:---:|:---:|:---:|:---:|---|
| A | a new `TypeDef::Builtin` variant | YES | **NO** | YES | **NO** | reject — Simple·UX |
| B | a new `Nature::Primitive` + zero-field `Aggregate` | **NO** | YES | **NO** | **NO** | reject — Obvious·Honest·UX |
| C | `TypeEnv` grows `builtin_type_names: HashSet<String>`; `contains` consults both | YES | YES | YES | YES | runner-up |
| **D** | **one door — `classify(name) -> {Defined(&TypeDef), Builtin, Unknown}`; `contains`/`get` become wrappers** | **YES** | **YES** | **YES** | **YES** | **proposed** |

**A.** *Obvious* YES — a builtin type is a type, so it lives with the types. *Simple* **NO** — 311
`TypeDef::` sites, and every exhaustive match grows an arm that has nothing meaningful to do with a
primitive; it braids "is this a name?" with "what is its structure?". *Honest* YES — one registry, and
the compiler forces every consumer to confront the new kind. *Good UX* **NO** — three hundred sites
write an arm they then ignore.

**B.** *Obvious* **NO** — an `AggregateDef` with zero fields and `Nature::Primitive` states that `i64`
is an aggregate, which it is not; a reader meets a falsehood in the first line. *Simple* YES — no new
variant, existing plumbing, and this is the option's whole attraction. *Honest* **NO** — it reuses a
shape whose meaning it contradicts, and `Nature` is consulted for nature-root logic that would now
see a fourth kind that is not one. *Good UX* **NO** — every `Nature` match gains a case to
special-case away.

**C.** *Obvious* YES — "these names are types and have no user-visible definition" is exactly true of
a primitive. *Simple* YES — one field, one `||` in `contains`, no variant, no match churn. *Honest*
YES — `contains` answers membership, `get` answers structure, and a primitive genuinely has the first
without the second; measured, nothing today assumes otherwise. *Good UX* YES — the 10 `contains`
callers get the right answer unchanged and the 311 `TypeDef::` sites are untouched.

**D — C, plus the rung that makes the wrong assumption unrepresentable.** C leaves a live hazard: a
future caller may reasonably assume `contains ⇒ get().is_some()`, and nothing stops them. D hands the
caller a discriminated answer instead of two independent predicates, so the assumption has no form to
be written in. *Obvious* YES — one question, three honest outcomes. *Simple* YES — one function; the
two existing doors become thin wrappers, so no call site changes. *Honest* YES — "exists but has no
`TypeDef`" becomes a NAMED state rather than an asymmetry a caller has to already know about.
*Good UX* YES — the door cannot be misread.

C and D both pass four-for-four; **D is proposed because it forecloses a hazard C merely avoids
today**, at the cost of one function. If the builder prefers the smaller change, C is honest and this
DESIGN does not argue otherwise.

## The gate — or this becomes another list that rots

Registering thirty names by hand fixes today and rots tomorrow. The stone is not done until adding a
builtin type without registering it is **caught**, not merely discouraged:

> every type name the checker can produce or consult must be a name the registry knows.

`BARE_PRIMITIVES` and `BARE_CONTAINER_HEADS` are already consts, so those two groups can be
registered *from* the const rather than copied beside it — no drift possible. The opaques are the
residue that needs a wall. ⚠ **Do not size that residue with grep** — impose the registration and read
what the corpus rejects. `[[feedback_impose_the_check_and_read_the_screams]]`

## What this stone does NOT do

- It does not touch 255.1 (callable builtins into `sym`) or the reserved-prefix blanket-accept for
  CALL heads. Different population, different slice, still 255's.
- It does not build the type-reference wall. That wall is parked on branch
  `arc109-type-refs-parked` and consumes this door once it exists.
- It does not fix the `defsurface` per-method-generics bug. That is a real independent defect
  (`register_types_impl` destructures `SurfaceMember::Method { .., .. }` discarding `type_params`,
  then mints the alias with `surf.type_params`), and it has no observable symptom until the wall
  lands — which is why the wall is its instrument and the two land together.

## Acceptance — the shape

1. `TypeEnv` answers membership for `:wat::core::i64`, `:wat::core::Vector`, `:wat::kernel::Peer` and
   every other name in the population.
2. Nothing that consumes `TypeDef` structure changes behaviour — a builtin has membership, not
   structure, and no existing match learns a new arm.
3. The floor is unchanged. **This stone is invisible from wat**: nothing reads the new answer yet.
   That is the point — it is the door, not the wall.
4. ⛔ A negative control that FAILS: a name that is not a builtin type must still be Unknown. Without
   it, a registry that answers "yes" to everything passes rows 1-3.
