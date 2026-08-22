# DESIGN — the type registry holds the BUILTIN types

**Status: RULED 2026-08-22 — E (consumption: the existing door) implemented by C (storage). Builder tiebreak: narrow waists. See the CORRECTION at the end — E alone was not implementable.**
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

---

## ⛔⛔ THE TIEBREAK KILLS BOTH C AND D — THE NARROW WAIST ALREADY EXISTS

**Builder, 2026-08-22:** *"which solution is the long term endstate… we should be installing killing
blows to bad practices and enabling strong evolutionary traits… narrow waists are the tie breaker."*

Asked on that axis, I went looking for the waist and found it **already built**, in the file I had
been reading all afternoon.

`src/value/symbol_table.rs:244`, and its own comment calls it **THE DOOR**:

```rust
/// Every facet `name` is registered under, across all five registries.
pub fn registrations(&self, name: &str) -> RegistrationSet
```

> *"One call answers 'what is registered under this name?' across every [registry] — so a
> single-registry read is a DELIBERATE, greppable choice, never the default that happens because
> four were forgotten."*

```rust
pub enum RegistryKind { Macro, Type, Function, UnitVariant, DefValue }
```

> *"⛔ EXHAUSTIVE BY LAW. The `_`-wildcard ban on enum scrutinees means adding a sixth registry turns
> every consumer's match RED until it decides what the new kind means."*

And its `Type` facet is answered by exactly the door this stone is about:

```rust
if self.types.as_ref().is_some_and(|t| t.contains(name)) { set.push(RegistryKind::Type); }
```

### What that means for C and D

**Both are rejected, and D is the worse of the two.**

**C — a side `HashSet` consulted by `contains`** — puts the Type facet's truth in two places, so the
waist answers from a widened predicate. It widens the waist by one store per kind of thing we later
remember. Anti-waist.

**D — a new `classify()` door** — is worse, and the diagnosis is embarrassing: **it mints a rival to
the function whose own comment names it THE DOOR.** Two doors answering "what is this name?" is not a
narrow waist; it is no waist. I proposed it *because* it was the shape of a waist, without checking
whether the waist existed — the same failure as the hand-list one level up, and the same failure as
re-deriving a note filed on 2026-08-05.
`[[feedback_search_for_the_mechanism_not_in_the_broken_callers_neighbourhood]]`

### E — the endstate: populate `TypeEnv`; the existing waist becomes honest for free

There is no new door, no new variant, no new predicate, and no list. `register_builtin_types` gains
the ~30 names it never held, and:

```
registrations(":wat::core::i64")     →  {Type}      through the door that already exists
registrations(":wat::kernel::Peer")  →  {Type}      "
registrations(":user::NoSuchType")   →  {}          the honest answer, for the first time
```

Every waiting consumer is served through the interface it already uses — the type-reference wall,
W1's capability keys, reflection, the undefined-func class. None of them learns a new API.

| # | option | Obvious | Simple | Honest | Good UX | narrow waist? | verdict |
|---|---|:---:|:---:|:---:|:---:|:---:|---|
| A | `TypeDef::Builtin` variant | YES | NO | YES | NO | — | reject |
| B | `Nature::Primitive` | NO | YES | NO | NO | — | reject |
| C | side set + `\|\|` in `contains` | YES | YES | YES | YES | **NO — widens** | reject |
| D | a new `classify()` door | YES | YES | YES | YES | **NO — a rival door** | reject |
| **E** | **populate `TypeEnv`; the door already exists** | **YES** | **YES** | **YES** | **YES** | **YES** | **TAKE** |

### Why E is the evolutionary shape, stated as traits rather than praise

- **The waist is one call, five kinds, closed.** Everything above it (resolve, checker, W1,
  reflection, diagnostics) and every registry below it evolves independently. That is the hourglass.
- **Adding a new KIND of nameable thing is a compile-time wall** — the `_`-wildcard ban turns every
  consumer red until it rules on the new kind. Adding a new INSTANCE is just registration. Cheap
  where it should be cheap, loud where it should be loud.
- **Two bad practices lose their form.** The *name-shape test* (255's ruled-out "B3 forgery" —
  asking whether a string LOOKS like a type) has nothing left to do, because the door says what a
  name IS. And the *private copy* (a resolver-local list of builtin names) has nowhere to live,
  because the facet it would duplicate is already the answer.
- **It is subtractive.** The stone deletes a category of future work rather than adding a mechanism:
  after it, "is this name live?" has exactly one implementation for every kind of name.

★ The whole cost of the missing feature was thirty unregistered names, and it produced a hand-list, a
false DESIGN premise, a rejected second resolution path, and a wall that could not be built. **The
waist was right; the data behind one of its facets was empty.**

---

## ⛔ CORRECTION, same session — E CONFLATED TWO AXES. C IS NOT DEAD; IT IS THE MECHANISM UNDER E.

Writing the brief exposed an error in how I presented the ruling. `TypeEnv`'s store is

```rust
types: HashMap<String, TypeDef>,
```

**A name cannot be registered without a `TypeDef` value.** So "populate `TypeEnv`" is not, by itself,
an implementable instruction — it still has to answer *what is stored for `:wat::core::i64`*, which
is the very question A/B/C were about. E did not dissolve that question; it answered a different one.

The two axes, separated:

| axis | question | status |
|---|---|---|
| **consumption** | which door do consumers ask? | **SETTLED by the waist tiebreak** — `SymbolTable::registrations` → `TypeEnv::contains`. **D is dead**: a rival `classify()` beside THE DOOR is a second waist. |
| **storage** | what does `TypeEnv` hold for a name with no structure? | **still open — A vs B vs C**, and the waist argument does not reach it. |

**What the tiebreak actually killed was D, and it killed it correctly.** C was never a rival door; C
is a storage choice *behind* the same door, and it survives:

- **A — a `TypeDef::Builtin` variant.** One store, but 311 `TypeDef::` sites, most of which gain an
  arm about primitives they do not care about. Fails Simple and UX, unchanged.
- **B — `Nature::Primitive` + zero-field Aggregate.** Still says `i64` is an aggregate. Fails
  Obvious and Honest, unchanged.
- **C — `TypeEnv` grows a second store of names-without-structure; `contains` consults both; `get`
  still returns `None`.** Four-for-four, and now also the narrow-waist answer *at its own layer*:
  `TypeEnv`'s interface (`contains` / `get`) is unchanged, so nothing above it — including THE DOOR —
  learns anything new. The storage widens; the waist does not.

**Ruling stands as E, implemented by C.** The honest statement of the stone is therefore: *the answer
comes from the door that already exists; `TypeEnv` grows the store that lets that door tell the truth
about names that have membership but no structure.*

★ The distinction is exactly the one `registrations`' own comment draws — membership ("what is
registered under this name?") versus structure ("give me its definition"). A primitive genuinely has
the first and not the second, and C is the only option that lets the registry SAY that rather than
fabricate a structure to satisfy a map's value type. Measured: **zero** call sites do
contains-then-unwrap-`get`, so the asymmetry breaks nothing today.
