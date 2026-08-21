# DESIGN — arc 109: `:- [T …]`, the declaration binder (three strikes)

**Status: DRAWN 2026-08-20, builder-agreed.** Written against `c95505498`.

## The form

Every `def*` takes an OPTIONAL param-spec immediately after its name — the same slot, the same
operator, in all of them:

```clojure
(wat.core/defn      my.ns/f        :- [T]   [x :- T]  :- T  body)
(wat.core/defrecord my.ns/Record   :- [T]   [first :- T  something :- wat.type/i64])
(wat.core/defenum   wat.core/Option :- [T]  wat.enum/Pure   Some :- [val :- T]  None :- [])
```

Builder: *"defrecord also needs the param-spec optional expr … right?"* Yes, and uniformly:
**name, then the optional binder, then whatever that declarator already took.**

This replaces the name-embedded `<T>` (`:my::ns::Wrapper<T>`), which arc 109 is annihilating.

## The RED baseline — MEASURED, all seven, one file each

None of them accepts the binder today. Each fails differently, which is itself the room map:

```
defrecord    "macro :wat::core::defrecord expects 2 arguments; got 4"
defenum      "malformed :wat::core::defenum declaration: triple is incomplete; expected `name …"
defstruct    "malformed structtype declaration: expected (:structtype :Name :Parent [fields])"
defsurface   "malformed :wat::core::defsurface declaration: expected `:nature :<kw>` after the…"
typealias    "expected (:wat::core::typealias :name :Expr); got 4 args"
newtype      "expected (:wat::core::newtype :name :InnerType); got 4 args"
defn         "malformed :wat::core::fn form: fn signature: expected a vector `[name <- :T ...]"
```

★ And the FIELD half already works — `[first :- T  something :- :wat::core::i64]` type-checks at
HEAD (`parse_argspec_triples` accepts `:-` since 251.4a). **Only the name slot is missing.**

## Why it is REQUIRED for 32 sites and OPTIONAL for 52

Measured (`NOTE-the-name-embedded-type-params-split-in-two.md`):

- **`defn`'s `<T>` is DECORATIVE.** `src/function/eval.rs:66` hardcodes `type_params: Vec::new()`;
  a free type variable in a signature generalizes implicitly. `(defn :user::ident<T> …)` and
  `(defn :user::ident …)` both print 42.
- **The type declarators' `<T>` is LOAD-BEARING.** `parse_declared_name` (`src/types.rs:4247`)
  turns it into `type_params`. Drop it and `T` becomes a **concrete type named `T`** —
  `"expects :T; got :wat::core::i64"`, a plausible message naming a type that does not exist.

```
32  REQUIRED   defenum 11 · defsurface 11 · defrecord 7 · defstruct 2 · defservice 1
52  OPTIONAL   defn — explicit quantification where today it is implicit
```

## The room map — TWO LAYERS, and the dependency is one-directional

| declarator | defined | layer |
|---|---|---|
| `defenum` | `src/types.rs:3726` | Rust parser |
| `defsurface` | `src/types.rs:3736` | Rust parser |
| `typealias` | `src/types.rs:3728` | Rust parser |
| `newtype` | `src/types.rs:3727` | Rust parser |
| `typeunion` | `src/types.rs:4065` | Rust parser |
| *shared* | `parse_declared_name`, `src/types.rs:4247` | Rust |
| `defrecord` | `wat/Record.wat:108` | wat macro, **2 fixed args** |
| `defstruct` | `wat/core.wat:1830` | wat macro |
| `defservice` | `wat/service.wat:180` | wat macro |
| `defn` | `wat/core.wat:673` | wat macro, **already variadic** (`& rest`) |

★ **The wat macros LOWER INTO the Rust parsers**, and `parse_declared_name` is where a name's
params are read. So the Rust half is a genuine prerequisite: until it can receive params from a
sibling slot, a macro has nowhere to hand them.

## The three strikes

- **α — the Rust parsers (REQUIRED, no corpus edits).** `parse_declared_name` gains a sibling-slot
  source for `type_params`; the five Rust-parsed declarators accept an optional `:- [T …]` after the
  name. Closes 22 of the 32 required sites (defenum 11 + defsurface 11). Rust-only, so it cannot
  break a `.wat` file by construction.
- **β — the wat macros that carry required sites.** `defrecord` (2 fixed args → optional third),
  `defstruct`, `defservice`. Closes the remaining 10. Depends on α.
- **γ — `defn` (OPTIONAL, doctrinal).** 52 sites. Its macro is ALREADY variadic
  (`[name <- WatAST & rest <- Vector<WatAST>]`), so the binder is a body change, not a signature
  change — the smallest of the three despite the largest site count. Makes quantification explicit
  where it is currently implicit, which is the no-inference doctrine applied to functions.

## Out of scope, affirmatively

- **Retiring the `<T>` name spelling.** All three strikes are ADDITIVE — dual-read, exactly as
  ②-i-b was. ③ hard-cuts. Nothing in this segment may make a `<T>` name stop parsing.
- **The corpus migration of the 84 sites** — that is the codemod's slot rule
  (`NOTE`: the codemod turns a binder into an application today), which lands after β.
- **`defn`'s implicit generalization.** γ ADDS an explicit form; it does not remove the implicit
  one. Whether implicit generalization is eventually banned is a separate ruling.
