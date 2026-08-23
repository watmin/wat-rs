# DESIGN — nothing MINTS and nothing RENDERS the angle form

> *"release the shadowdancer - defservice emits the binder"* — the builder, 2026-08-23

Position 4 works as of `69933d362`, so a macro can now emit what it always should have. This stone
spends that capability, and closes a channel the earlier census never saw.

## The rule being made true

```
absent  ≡  :- []  ≡  the empty binder        expressed or not, same thing
```

Measured at **declaration** (`defn`, `defrecord`) ✅ and at **constructor** (`(:u::Plain :- [] :n 9)` → `9`) ✅.

⚠ **And measured FALSE at type REFERENCE:**

```clojure
(:wat::core::defn :u::takes [x <- (:u::Plain :- [])] -> :i64 …)
  →  ":u::takes: parameter #1 expects :u::Plain<>; got :u::Plain"
```

`(Head :- [])` becomes `Head<>` — a distinct identity that does not match `Head`. So the builder's
rule does not yet hold everywhere, and the five mono-vs-parametric branches in `wat/service.wat`
**cannot** be made unconditional until it does.

## The fifth minting channel — the RENDERERS

The earlier census found four ways an angle name enters the world and walled two. There is a fifth,
and it never touches `keyword/from-string` or `keyword-node`, so no wall would have caught it:

```
src/check.rs:16278      format!("{}<{}>", head, inner.join(","))
src/runtime.rs:13401    format!("{}<{}>", n, f.type_params.join(","))
src/runtime.rs:13480    format!("{}<{}>", name, scheme.type_params.join(","))
src/runtime.rs:13647    format!("{}<{}>", base, type_params.join(","))
```

Four sites re-serialize a type back INTO the retired spelling. With an empty param list they emit
`Head<>` — which is exactly where the reference-position failure above comes from.

★ **And they are user-facing.** Measured on the current build:

```
:u::want: parameter #1 expects :wat::core::Vector<wat::core::i64>; got :wat::core::String
```

**A user who copies that type into their source gets a LEX ERROR.** The diagnostic teaches a spelling
the reader refuses — substrate-as-teacher running backwards. This is the sharpest reason the stone
exists: a wall whose own error messages contradict it is not a wall, it is a trap.

## What ships

**1. The four renderers emit the surviving form.**

```
type_params EMPTY     →  Head                      (never `Head<>`)
otherwise             →  (Head :- [A B])
```

One shared helper — four `format!`s with one shape is the disease this arc has spent the day on. Its
home is beside the existing name/param-spec doors, and it is subject to the `one_param_spec` rune.

**2. `(Head :- [])` ≡ `Head` at REFERENCE position**, which falls out of (1) once the renderer stops
minting `Head<>`. This is what makes the builder's rule true in all four positions.

**3. `wat/service.wat` stops minting, and loses its branches.**

```
942-943  proto-op-ty-kw / proto-reply-ty-kw   DEAD BINDINGS — defined, never used.
                                              Verified: one occurrence each in the file.
                                              They are the FIRST wall scream, and they are dead code.
2374-85  launch-head-kw                       `wat::spawn::Locus/launch<A,B,C,D,E>` → the bare
                                              keyword, with `:- [...]` emitted as SIBLINGS at the
                                              call site (position 4, live since 69933d362).
500      proto-tp                             the `<…>` suffix string — dies with its last consumer.
1021·1024·1360·2014·2025                      five `(if (empty? proto-args) …)` branches become
                                              UNCONDITIONAL once (2) lands. This is the builder's
                                              rule cashed out: the macro always emits `:- [args]`.
```

★ **The exemplar is already in the file.** `proto-op-ty-ann` (line 1021) mints the reference FORM
structurally off `proto-args` — ③ wrote it correctly. Every remaining site copies that shape; the
only change is dropping its `if`.

**4. `wat/core.wat`'s `:wat::core::keyword/of`** — a stdlib macro whose entire purpose is building
`Head<a,b>`. One caller (`tests/macros/probe_arc249_4_rehome_in_wat_kw_of_tmpl.wat`). It emits the
form or it retires.

## What this does NOT do

Affirmatively cut:

- **The minting wall itself.** Parked as a patch, cascade measured at 3034/4893. It goes up in the
  NEXT stone, once nothing mints — putting it up first would only re-red the tree for no new
  information.
- **`symbol-node`'s wall.** Same stone as the minting wall.
- **The purge of the angle PARSERS.** Dead only once nothing mints *and* nothing renders; needs a
  green floor to say so.

## The four questions

- **Obvious?** YES. One spelling everywhere, including in what the compiler says back to you.
- **Simple?** YES. Four `format!`s become one helper; five `if`s become no `if`s; two dead bindings
  go. The stone deletes more than it adds.
- **Honest?** YES, and this is the axis that is failing right now: the substrate refuses a spelling at
  the reader and then prints that same spelling in every type error. One of those is a lie.
- **Good UX?** YES. A type error you can copy back into your program is the whole point of printing
  the type.
