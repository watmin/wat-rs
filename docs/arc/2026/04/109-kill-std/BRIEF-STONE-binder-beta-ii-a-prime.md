# BRIEF — arc 109, β-ii-a′: `defservice` accepts `:- [K V]`, and the old `<K,V>` string becomes a derived shim

`defservice` decides parametricity by asking whether its NAME ends in `>`, and carries the params as
a bracketed STRING it interpolates into ~50 generated type names. This stone makes the **binder the
source of truth** and leaves the string as a derived compatibility shim, so all ~50 consumers keep
working untouched.

Design: `DESIGN-STONE-binder-beta-ii.md`, including the 2026-08-21 amendment explaining why the
first decomposition was inverted. Read it first.

```clojure
(:wat::service::defservice :wat::cache::lru-svc :- [K V]      ; ← the new spelling
  :satisfies …)
(:wat::service::defservice :wat::cache::lru-svc<K,V> …)       ; ← must KEEP working
```

## Rooms

1. **`wat/service.wat:180`** — the macro signature: `[fqdn <- WatAST & clauses <- Vector<WatAST>]`.
   Already variadic, so a `:- [K V]` pair arrives as the first two elements of `clauses`.
2. **`wat/service.wat:217–227`** — `fqdn-parametric?` / `fqdn-base` / `fqdn-tp`, the three bindings
   that derive params by splitting the name on `<`. `fqdn-tp` is the bracketed SUFFIX, `"<K,V>"`.
3. **`wat/service.wat:~231–295`** — the clause fold. It rejects any key not in `known-clauses` via
   `macro-error` (`:290`). **The binder must be peeled before this runs**, or `:-` is rejected as an
   unknown clause.
4. **`wat/cache.wat:195`** — `lru-svc<K,V>`, the one parametric service. **Do not migrate it.** It
   is your dual-read acceptance target in the OLD spelling.
5. **`wat-tests/service-cache-lru.wat`** — starts the service, dials two clients, exercises put/get
   at concrete types. Read it; do not edit it.

## The work

**Peel, then derive, in this order:**

1. If `(first clauses)` is the `:-` keyword node, take it and the `WatAST::Vector` after it, and
   continue with the REMAINING clauses. Otherwise proceed unchanged.
2. `fqdn-tp-syms` — the param symbol nodes: `ast->children` of that vector, or an EMPTY vector when
   there is no binder. `ast->children` is allow-listed in macro bodies (145 uses); so is `ast-name`
   (131). **You need no new primitive** — if you find yourself wanting one, that is STOP-2.
3. **Dual-read** for where the params come from:
   - binder present → params are `fqdn-tp-syms`; `fqdn-base` is `fqdn-str` as written.
   - no binder, name ends in `>` → the EXISTING split path, unchanged.
   - neither → monomorphic; empty vector, `""`.
4. **Derive `fqdn-tp` from the params** — `"<" + names joined by "," + ">"`, or `""` — so every one
   of the ~50 consumers keeps reading exactly what it reads today. `ast-name` gives each name.
   This is the compatibility shim, and it is the thing β-ii-b/c will delete.

`proto-tp` (`:364–372`) is NOT in this stone — a `:satisfies` surface still names its params in its
own keyword, and nothing about that changes here.

## The contract decision, pinned

**The binder is the source of truth; the string is derived FROM it, never the reverse.** A reader
must be able to see that the string is downstream. Do not compute both independently from the
name — that is two derivations that can disagree, and the disagreement would surface ~50 sites away.

## STOP triggers

1. **STOP-1** — if `lru-svc<K,V>` in its CURRENT spelling stops expanding, STOP. Dual-read; ③ cuts.
2. **STOP-2** — if you need a new intrinsic, or a top-level `defn`, or any helper that is not
   already used inside a macro body, STOP and report. The F5 gate is default-deny: a top-level
   `defn` called from a macro body is refused AT DEFINITION and takes the whole stdlib down (3029
   failures, measured). A `let`-bound closure does not rescue it either — `mapv` refuses a bare
   builtin-primitive keyword as a value (`expected "wat::core::fn"`, measured). If the work seems to
   need one, the shape is wrong and the orchestrator re-plans.
3. **STOP-3** — if the binder cannot be peeled before the clause fold without restructuring the
   fold, STOP and report. The fold's `macro-error` on unknown keys is load-bearing and not yours.
4. **STOP-4** — edit `wat/service.wat` ONLY.

## Blast radius

`wat/service.wat` — the peel, the syms binding, the dual-read branch, and the derived string. **No
emission changes. No other file. No new primitive.**

## How this lands

You are a rider. **Text edits only.** Do not run cargo, build, commit, stash, or revert.

⚠ **You cannot test this edit.** `wat/service.wat` is baked into the binary by `include_str!` at
RUST-compile time, so `--check` reflects the LAST BUILD and will print a staleness warning. Trace
correctness by reading. Say plainly in your report what you verified by reading and what you could
not verify at all.

Report: the diff; how you detect the `:-` node; how the empty case is represented; the exact derived
string for `[K V]`; and anything on disk that contradicts this brief. I have been wrong about this
macro's size three times and about which helpers are legal once — the brief is my claim, not the
ground.
