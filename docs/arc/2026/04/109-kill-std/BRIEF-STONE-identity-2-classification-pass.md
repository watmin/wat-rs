# BRIEF — identity stone 2, step 0: CLASSIFY `defservice`'s type-name sites

**This is a MEASUREMENT task. You change no source code.** Your deliverable is one committed
markdown table. The implementation brief will be written FROM your table, so its accuracy is the
whole job.

## Why this exists

`wat/service.wat`'s `defservice` macro builds type names as STRINGS and turns them into nodes with
`:wat::core::keyword/from-string`. Arc 109 is migrating the language off the angle spelling
(`Head<A,B>`) onto the `:-` form (`(Head :- [A B])`), so every one of those built names must change —
but **not all of them the same way**, and a uniform conversion would break things silently.

Measured proof that a uniform conversion is wrong — the SAME binding, two consumers:

```clojure
service-op-decl-kw   (:wat::core::keyword/from-string service-op-ty-str)      ; service.wat:1165

;; :1248 — a DECLARATION NAME slot. Under `:-` this must be a BINDER: `name :- [K V]`
`(:wat::core::defenum ~service-op-decl-kw :wat::enum::Pure ~@service-op-variant-items)

;; :1799 — a RUNTIME ARGUMENT to an intrinsic. This must stay a plain KEYWORD.
(:wat::core::match (:wat::kernel::retag-op op ~proto-op-ty-kw ~service-op-decl-kw) …)
```

One conversion breaks one of those. The rule is the SLOT, not the value — and finding which slot each
site lands in is what this pass produces.

## ⚠ THE POPULATION IS NOT KNOWN — establishing it is your FIRST deliverable

Three greps of mine returned **23**, **53** and **110**, each with a different pattern. **Do not
trust any of those numbers, including the last.** Derive the population yourself, state the exact
command you used, and say what that command can and cannot see. A count you cannot reproduce is not
a measurement.

Scope it to bindings whose value is or becomes a **TYPE name**. `defservice` also builds FUNCTION and
VALUE names (`init-name`, `serve-name`, `handle-new-kw`) — those are NOT in scope, and saying so
explicitly for each excluded binding is part of the deliverable, because "I didn't see it" and "I
ruled it out" must be distinguishable in your table.

## The roles — four found by sampling five sites, so expect more

| role | what it means | destination under `:-` |
|---|---|---|
| **ANNOTATION** | spliced into a type slot: `[x <- ~ty]`, `-> ~ty` | a reference FORM `(Head :- [args])` |
| **DECL-NAME** | the name slot of a declaration: `(defenum ~kw …)`, `(defrecord ~kw …)` | a BINDER `name :- [args]` |
| **CTOR-ARG** | a constructor's element type: `(Vector ~ty)` | the constructor's own `:-` slot |
| **RUNTIME-ARG** | passed to an intrinsic at runtime: `retag-op`, `derive` | stays a plain KEYWORD |
| **STRING-FRAG** | interpolated into another string, never a node | unchanged |
| **OTHER** | ★ anything that fits none of the above | **report it — do not force a fit** |

★ **`OTHER` is the most valuable column you can fill.** A site you cannot classify is a finding. A
site you force into a bucket is a defect shipped into the implementation brief.

## The table

One row per site. Every cell must cite a line number that a reader can open:

```
| binding | built at | consumed at | consumer form (verbatim, trimmed) | role | notes |
```

- **consumed at** — EVERY consumption, not the first. A binding with two consumers in two roles is
  the single most important thing this pass can surface; `service-op-decl-kw` is the known example
  and there may be others.
- **consumer form** — the actual splice site, quoted. Not a description of it.
- **notes** — anything that made you hesitate.

Write it to `docs/arc/2026/04/109-kill-std/TABLE-defservice-type-name-sites.md`, with a header
stating the command you derived the population from and what it cannot see.

## Boundaries

- **Change no source.** No edits to `wat/`, `src/`, or anything outside your one new markdown file.
- Do NOT run `scripts/floor.sh`, `cargo nextest`, or `cargo build` — nothing you do needs them.
- Do NOT commit, push, stash or amend. Leave the file in the working tree.
- Do NOT propose fixes. This pass classifies; the implementation brief is written later, from it.

## STOP triggers — report rather than continue

- **STOP-1.** If a binding's consumers cannot be found (built and never used, or reached only through
  another binding you cannot trace), record it as `OTHER` with what you tried. Do not guess.
- **STOP-2.** If the population you derive differs from ~110 by more than a factor of two in either
  direction, STOP and report your command and count before filling in 200 rows. One of us has the
  wrong instrument and it is cheaper to find out early.

## Your report

The command you used to derive the population, and its count. What that command cannot see. The
role distribution (how many of each). Every `OTHER` row, called out explicitly. Every binding with
MORE THAN ONE role. What surprised you.
