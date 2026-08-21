# BRIEF — arc 109, binder strike β-i: the two `defrecord` macros accept `:- [T …]`

α wired the Rust declaration parsers. Two wat macros still gate on their own arity before Rust ever
sees the form, and they are siblings of a macro that ALREADY works. Make them match it.

```
(:wat::core::defrecord :user::R :- [T] [item <- T])   → "macro :wat::core::defrecord expects 2 arguments; got 4"
(:wat::core::defstruct :user::S :- [T] [f <- T])      → WORKS TODAY
```

Design: `DESIGN-STONE-the-declaration-binder.md`. α's score: `SCORE-STONE-binder-alpha.md`.

**ADDITIVE.** Every `<T>`-spelled `defrecord` must keep working. ③ hard-cuts, not this stone.

## ★ Read the worked example FIRST

**`wat/core.wat:1830`** — `:wat::core::defstruct`. It takes `[& args <- :wat::core::Vector<wat::WatAST>]`
— fully variadic — and picks its slots by position from the ends:

```clojure
[fqdn   (:wat::core::first args)
 fields (:wat::core::Option/expect (:wat::core::last args) "defstruct: missing field-vector")
 ...
```

Because it never counts its args, a `:- [T…]` pair sitting between the name and the field vector
rides through untouched. That is the whole reason it went green the moment α landed — verified by
running it, not by reading:

```
(:wat::core::defstruct :user::Box :- [T] [item <- T])  with :item 42    → #user/Box {:item 42}
                                                        with :item "hi" → #user/Box {:item "hi"}
```

`T` holds an i64 AND a String, so it is a genuine type variable. **This is the shape to copy.**

## Rooms

1. **`wat/core.wat:1830`** — the worked example above.
2. **`wat/Record.wat:108`** — `:wat::core::defrecord`, signature
   `[fqdn <- :wat::WatAST  fields <- :wat::WatAST]`. **Two FIXED args — this is the gate.**
3. **`wat/Record.wat:178`** — its emission:
   ```clojure
   `(:wat::core::do
      (:wat::core::recordtype ~fqdn :wat::core::Record
   ```
   The binder must land **between `~fqdn` and the parent keyword** — that is the slot α's
   `parse_aggregate` reads, verified:
   `(:wat::core::recordtype :user::R :- [T] :wat::core::Record [f <- T])` checks clean.
4. **`wat/Record.wat:207`** — `:wat::holon::defrecord`, the same 2-fixed-arg shape, emitting with
   the `:wat::holon::Record` parent instead. Same change, same place.

★ **`wat/Record.wat:156`** carries a comment worth reading before you touch the emission:
*"params ride ONLY on the recordtype decl"* — the type params already travel on that decl and not
on the kwargs companion, so the binder follows a path the macro has already chosen.

## The work

Give both macros `defstruct`'s shape: fully variadic, slots picked from the ends, plus a middle
that is either empty or the binder pair.

```clojure
[fqdn        (:wat::core::first args)
 fields      (:wat::core::Option/expect (:wat::core::last args) "defrecord: missing field-vector")
 binder      ;; [] when args is [fqdn fields]; [:- [T…]] when it is [fqdn :- [T…] fields]
 ...]
;; emission:
`(:wat::core::recordtype ~fqdn ~@binder :wat::core::Record ...)
```

The rest of each macro body is UNCHANGED — the kwargs-companion field walk still reads `fields`,
which is still the last arg.

## STOP triggers

1. **STOP-1** — if any existing `defrecord` stops expanding, STOP. Additive. `defrecord` is a core
   macro with hundreds of call sites; a change that alters what a NON-binder call expands to is a
   defect regardless of what the floor says.
2. **STOP-2** — if the binder cannot be forwarded into the `recordtype` emission without also
   changing the kwargs-companion branch, STOP and report. The companion deliberately does not carry
   type params (`Record.wat:156`); making it carry them is a different decision and not yours.
3. **STOP-3** — if making the signature variadic would require the body to count `args` anywhere,
   STOP and report where. `defstruct` proves the ends-based shape works; a count is what reintroduces
   the arity gate this stone exists to remove.
4. **STOP-4** — do NOT edit any `.wat` file other than `wat/Record.wat`. In particular do not migrate
   the 7 parametric `defrecord` CALL SITES to the new spelling — that is the codemod's job, and both
   spellings must work when you are done.

## Blast radius

`wat/Record.wat` only — two macro signatures, two bodies, two emissions. No Rust. No other `.wat`
file. No call site.

⚠ One diagnostic will change and it must not get worse: today a malformed `(:wat::core::defrecord :X)`
answers *"expects 2 arguments; got 1"*. After the change the arity gate is gone, so the body must say
something at least as clear — `defstruct`'s *"defstruct: missing field-vector"* is the precedent.

## How this lands

You are a rider. **Text edits only.** The orchestrator builds, floors and clippies centrally, once.
Do not run cargo, do not commit, do not stash, do not revert. Run everything in the FOREGROUND —
your turn ends when your edits are on disk and your report is written, and ending your turn ends you.

You MAY run `./target/release/wat --check <file>` on a scratch file to sanity-check an expansion —
it is ~0.2s and does not touch the build. It reflects the CURRENT binary, which already has α.

Report: the diff; both new missing-field diagnostics verbatim; whether `~@binder` spliced cleanly or
needed another shape; anything on disk that contradicts this brief.
