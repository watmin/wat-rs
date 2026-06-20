# Arc 251 — NOTES (deferred needs, recorded for the Clojure-syntax flip)

Deferred items surfaced elsewhere that belong with the 251 Clojure-faithful surface work. Recorded, not yet
acted on.

## `first`/`second`/`third` should return BARE, not `Option` (Clojure semantics)
**Recorded 2026-06-19** (arc 278 P12c accessor work). Today `first`/`second`/`third` on every sequence
(Vec/List/PersistentVector/WatAST) return `Option<T>` (the *safe* accessor), while `nth` is the get-or-raise
*bare* accessor (`get` is the safe `Option` map/seq accessor). In Clojure, `(first xs)` is **bare** — the
element, or `nil` on empty — NOT wrapped in `Option`. The builder: *"the first not being an option is a legit
arc — that's a core piece of tooling, not a network service we may make — i don't want to deal with this now;
251 can record the need."*

The open question for the flip: should `first`/`second`/`third` become bare (Clojure-aligned), and if so, what
is `(first [])`? — raise, or a typed nil (wat is typed; `nil` isn't a valid element of `Vector<T>`). This is a
cross-cutting change across ALL sequence types (Vec/List/PV/WatAST), a core-tooling decision — to be settled
when 251 flips the surface to Clojure syntax, not piecemeal. `nth`-bare (the get-or-raise accessor) already
exists and works on Vec + PersistentVector (arc 278 fix); it's the bare positional accessor in the meantime.
