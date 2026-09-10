# DESIGN — ③b-i: the variant name is COMPOSED, never TYPED

**Ruled in the main chat, 2026-09-10, at `7ccce48ba`** (floor 5325/5325, clippy 0). Option **D**,
after option C — which I had proposed and which the builder had ruled — was measured unsound and
retracted.

## ⛔ WHY C DIED, AND IT WAS MY ANALYSIS THAT FAILED

C relaxed `has_dotted_name` syntactically: *a leaf may carry at most one dot.* I ran the four
questions on it and rated Honest *"mostly — it reopens one case, and the collision surfaces as a
`Duplicate`."* **I argued about what H-1 BANS and never asked what DEPENDS on the ban.**

`src/edn/render.rs:3520`, the UNTYPED decode path:

```rust
if let Some((enum_leaf, variant)) = split_variant_tag_name(name) {
    reconstruct_enum_tagged(...)      // taken on ANY name-half dot
} else {
    types.get(&path)                  // the record branch — NEVER REACHED
}
```

A name-half dot routes to the variant reconstructor **unconditionally**; there is no fallback to
the record branch. Under C a record declared `:my::app::Foo.Bar` renders `#my.app/Foo.Bar {…}` and
decodes as a variant of an enum that need not exist. **Silent misdecode, no `Duplicate` involved**,
because only one thing was ever declared. The typed path (`render.rs:3060`) errors cleanly; the
untyped one misroutes — and that asymmetry is precisely what `render.rs:3913` says the wall
prevents:

> *"a dot LEFT of the slash is namespace nesting; a dot in the NAME half means variant, and
> `resolve::registration`'s `DottedName` wall is what makes a record unable to forge one."*

I quoted that sentence into the main chat and still did not test it.
`[[feedback_a_containment_argument_must_name_its_consumers]]`

★ The instrument was the builder's question — *"if they make a defn and an enum with the same name,
one loses — this is pathological and we catch it, right?"* Chasing whether the collision was caught
is what put me in front of the wall's consumer. The collision was not the defect; asking was.

## THE RULING — two doors, one rule table

H-1 stays **absolute** for a declared name. The variant constructor path stops travelling through
that door and gets its own, which takes the two halves **separately** and composes them itself:

```
register(name, …)                    the USER door   — a name-half dot is refused, always
register_variant(parent, leaf, …)    the DOOR door   — composes via `compose_variant`,
                                                       and accepts the dot BECAUSE IT WROTE IT
```

★ **This preserves what H-1 makes true rather than weakening it.** A name-half dot still implies
"variant", because the only entry point that will accept one never receives a dotted string — it
receives `(parent, leaf)` and builds the name through the grammar's own composer. The EDN
discriminator stays sound by construction. C removed a wall; D moves the variant off it.

★★ Constraint engineering, not failure engineering: the wrong thing has **no representation**,
rather than being caught after the fact.

## Shape — `NameOrigin` is to "how was this name made" what `Privilege` is to "who made it"

`gate` stays the single rule table. It gains one explicit input beside the one it already has:

```rust
pub(crate) enum NameOrigin { Declared, ComposedVariant }

  Existing::Equivalent                                          -> NoOp
  Existing::Divergent                                           -> Duplicate
  Absent + !namespaced                                          -> Unnamespaced
  Absent + namespaced + Declared + dotted name                  -> DottedName   ← origin-gated
  Absent + namespaced + undotted + reserved + Privilege::User    -> Reserved
  Absent + namespaced + (Stdlib | !reserved)                     -> Insert
```

Only the `DottedName` arm changes, and only by acquiring `origin == Declared`. Every other wall —
Duplicate, Unnamespaced, Reserved — applies to a composed variant exactly as before. **A user still
cannot define a variant under `:wat::*`**: the enum's own name is refused at its own registration,
and `Reserved` still fires on the composed path regardless.

⚠ `Privilege` says WHO; `NameOrigin` says HOW THE NAME WAS MADE. Two axes, one table — this is the
same shape as arc 255's WALL-vs-BLANKET: one predicate, two consumers, told apart by an explicit
parameter rather than by a call-site bypass.

## The three sites, all of which already hold both halves

```
src/declare/preregister.rs:314   compose_variant(type_name, variant_name)   the ctor path
src/declare/register.rs:1341     compose_variant(&enum_def.name, variant_name)   unit variant
src/declare/register.rs:1371     compose_variant(&enum_def.name, variant_name)   tagged variant
```

## ★★★ ③b-i IS BEHAVIOUR-PRESERVING TODAY, AND THAT IS THE WHOLE POINT

Pre-flip `compose_variant` writes `::`, so a composed variant name has **no dot**, so `has_dotted_name`
would never have fired on it. Origin-gating an arm that cannot fire changes nothing observable.

**A ruling that moves no existing expectation is a ruling that took nothing away** — the two H-1
gate rows (`registration.rs:265`, `:271`) stay exactly as written, where C would have flipped both.
Under D they are still `DottedName`, because their origin is `Declared`.

So this stone lands green and inert, and becomes load-bearing at ③b-ii — the same discipline that
put A–E before F+G and ③a-ii before ③a-iii. When the irreversible one lands, the only thing that
can be wrong is the flip.

## The collision — measured, and UNPINNED

The builder's question, run on the real substrate at `7ccce48ba`, both orders:

```
defn :my::app::Foo::Bar  then  defenum :my::app::Foo :Bar    --check exit 1 · run exit 3
defenum :my::app::Foo :Bar  then  defn :my::app::Foo::Bar    --check exit 1 · run exit 3
    duplicate define: :my::app::Foo::Bar already registered      (register.rs:1352)
```

Caught, symmetrically, at check time — first-definer-wins never silently applies.

⛔ **But NOTHING PINS IT.** No test in the tree pairs a `defn` against an enum variant's ctor path.
And the catch lives in a DIFFERENT PHASE from the gate: `preregister.rs:315` maps
`sym.has_function(ctor_path)` to `Existing::Equivalent`, which the gate turns into a **`NoOp`** —
on its own that reads "already there, fine" and the variant would lose silently. The loud
`DuplicateDefine` comes from the later `register` pass, from an explicit check no test holds.
Fold that check into the gate, or "simplify" preregister's `Equivalent` mapping, and the
pathological case goes quiet with nothing turning red. A guard that has never failed once.

③b-i pins it. `[[feedback_a_green_test_can_prove_nothing]]`

## Acceptance

```
the two existing H-1 gate rows are UNCHANGED — DottedName, both privileges
gate(":my::Shape.Circle", ComposedVariant, …) == Insert          (the new arm, non-vacuous)
gate(":my::Shape.Circle", Declared,        …) == DottedName      (the wall, still absolute)
a two-dot composed leaf is still refused                          (D is not "any dot")
the collision raises DuplicateDefine at --check, BOTH orders      (the builder's question, pinned)
a `defn` with a dotted name is still refused at --check           (H-1 absolute, end to end)
floor 5325/5325 via scripts/floor.sh · clippy -D warnings = 0 · NO expectation moves
```

## What ③b-ii inherits

The flip, one landing: the two door bodies to `.`, the 28 `display` strings with them, and the
codemod over the corpus. Re-derived on this tree — **6,108 occurrences · 353 spellings · 482 of 843
files**, not the seam's stale 9,946 / 493 — plus 18 the capitalised-only predicate missed
(`PeerKind::thread`, `PeerKind::process`).

★ And those 18 are the codemod's whole justification, measured rather than argued:

```
::PeerKind::thread     12   a VARIANT   |   ::Record::def          25   a METHOD
::PeerKind::process     6   a VARIANT   |   ::HolographicLru::put  14   a METHOD
```

Textually identical — `::Capitalized::lowercase`. Only `variant-parent-of` separates them. A regex
renames `Record::def` and nothing fails; it just means something else.
