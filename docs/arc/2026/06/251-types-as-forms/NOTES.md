# Arc 251 — NOTES (deferred needs, recorded for the Clojure-syntax flip)

Deferred items surfaced elsewhere that belong with the 251 Clojure-faithful surface work. Recorded, not yet
acted on.

## ✅ DONE 2026-06-20 (arc 278, NOT 251) — `first`/`second`/`third` are now BARE, raising
**Completed** by the first-bare cut (`26d492e5` flip + strike-2 + `725faa3d` cascade close). Forced forward from
this deferral by the arc-278 container annihilation (the moment the accessor contract was in our hands). The
**open question below is resolved: bare ⇒ RAISE** — wat is typed and has no `nil` valid in `Vector<T>`, so "bare"
has exactly one honest meaning (raise on empty/out-of-range, like `nth`). `get` is the lone `Option` safe path;
`first`/`second`/`third`/`nth` are the bare/raising accessors. Cross-cutting: ~68 stdlib sites + test/lib/nursery
cascade, all floors green, NO shim. Full design + the argued why: `278-rules-engine/DESIGN-STONE-first-bare-accessors.md`.

---

*(Original note, kept as the record of the decision — now resolved above.)*

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
