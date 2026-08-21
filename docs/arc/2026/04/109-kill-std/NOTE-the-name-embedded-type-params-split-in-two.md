# ⛔ NOTE (arc 109) — `<T>` in a declaration name is DECORATIVE for `defn` and LOAD-BEARING for the seven type declarators

**Filed 2026-08-20. MEASURED at `c9938cc7b`.** Corrects a claim I made earlier this session.

## The correction

I told the builder that 251's declaration migrator *"drops `<T>` and leaves `T` free with nothing
binding it,"* and that **54** generic declarations *"have no destination today."* Half of that is
wrong, and the half that is right is worse than I described.

Measured, both halves, by running them:

```clojure
;; defn — the <T> is DECORATIVE. Both forms print 42.
(:wat::core::defn :user::ident<T> [x <- T] -> T x)      → 42
(:wat::core::defn :user::ident    [x <- T] -> T x)      → 42

;; defrecord — the <T> is LOAD-BEARING.
(:wat::core::defrecord :user::Box<T> [item <- T])       → #user/Box {:item 42}
(:wat::core::defrecord :user::Box    [item <- T])
    → TypeMismatch: ":user::Box: parameter #1 expects :T; got :wat::core::i64"
```

## Why the two differ, traced on disk

- **`defn` never consulted the name's `<…>` at all.** `src/function/eval.rs:66` hardcodes
  `type_params: Vec::new()`. A free type variable in a signature is implicitly generalized, so
  `T` works whether or not the name advertises it. The `<T>` is documentation that the checker
  never reads.
- **The seven type declarators DO consult it.** `parse_declared_name` (`src/types.rs:4247`) parses
  `:my::ns::Wrapper<T>` → `("my/ns/Wrapper", ["T"])`, and it is called from exactly seven sites:
  `types.rs:3782` (aggregate/defrecord), `:3972` (newtype), `:4014` (typealias), `:4065`
  (typeunion), `:4168`, `types/surface.rs:530` (defsurface), `types/defstruct.rs:520` (defstruct).
  Drop the `<T>` and `T` stops being a variable — it becomes a **concrete type named `T`**, and the
  failure is a TypeMismatch against a type that does not exist.

## What that does to the numbers

The 84 declaration sites the codemod would corrupt split:

```
52  defn        — <T> decorative.     Stripping it loses NOTHING semantically.
32  type decls  — <T> load-bearing.   Stripping it SILENTLY changes T from a variable
                                       to a nonexistent concrete type.
    (defenum 11 · defsurface 11 · defrecord 7 · defstruct 2 · defservice 1)
```

★ **The failure mode is the dangerous direction.** It is not "the binder is missing so nothing
compiles"; it is "`T` is now a concrete type," which produces a *plausible* TypeMismatch naming a
real-looking type. A reader debugging `expects :T; got :wat::core::i64` will look for a type called
`T` before suspecting the declaration lost its type parameters.

## Consequences for the queue

- **The binder is REQUIRED for the 32, and OPTIONAL for the 52.** `:- [T …]` in a type declarator's
  name slot relocates a load-bearing binder and must land before anything strips those names. For
  `defn` it would be a NEW capability — explicit quantification where today generalization is
  implicit — desirable under the no-inference doctrine, but not a correctness prerequisite.
- **The codemod bug is worse for the 32 than the 52.** For a `defn` name, wrapping
  `recv-all-loop<I,O>` into `(recv-all-loop [I O])` is a shape error the checker will reject loudly.
  For a `defrecord` name it is a shape error too — but the *simpler* mistake nearby (strip and emit
  nothing) is the silent one, and that is the one a later "just drop the params like 251 does" fix
  would reach for.
- **251's `c02_defn_generic_name_drops_type_params` golden is CORRECT** for what it tests. Its
  input is a `defn`. It does not generalize to the seven type declarators, and nothing on disk ever
  claimed it did — I generalized it.

`[[feedback_an_adjacent_implementation_is_not_the_subject]]` — twice in one session on this arc:
first `infer_list_constructor` (Vector's fn wearing List's name), now `defn`'s `<T>` standing in for
every declarator's `<T>`. Both times I read one member of a family and spoke for the family.
