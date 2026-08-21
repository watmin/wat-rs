# BRIEF — arc 109, β-ii-a: `defservice`'s type params gain a LIST representation, alongside the string

The stepping stone for β-ii. `defservice` carries its type parameters as a STRING with the brackets
baked in (`"<K,V>"`) and interpolates that into every generated type name. β-ii-b/c convert those
~50 emissions to forms. **This stone changes NO emission** — it only makes the list exist.

Design: `DESIGN-STONE-binder-beta-ii.md` (sibling). Read it first.

★ **This stone must be OBSERVATIONALLY INERT.** Nothing it adds is consumed yet. The floor must be
green with **zero golden churn** — if any golden moves, something downstream read the new value and
the stone did more than it claims.

## Rooms

Two blocks, identical in shape, in `wat/service.wat`:

**1 — the service's own name, `:217`–`:227`:**
```clojure
fqdn-parametric? (:wat::core::string::ends-with? fqdn-str ">")
fqdn-base        (:wat::core::if fqdn-parametric?
                   (:wat::core::first (:wat::core::string::split fqdn-str "<"))
                   fqdn-str)
fqdn-tp          (:wat::core::if fqdn-parametric?
                   (:wat::core::string::subs fqdn-str
                     (:wat::core::string::length fqdn-base)
                     (:wat::core::string::length fqdn-str))
                   "")
```
`fqdn-tp` is the bracketed SUFFIX — `"<K,V>"` — not the names.

**2 — the satisfied protocol's name, `:364`–`:372`:** the same three bindings spelled
`proto-parametric?`-less (the `if` tests inline), producing `proto-base` and `proto-tp`.

**3 — read but do not touch:** `wat/cache.wat:195`, the ONE parametric service in the corpus
(`lru-svc<K,V>`), and `wat-tests/service-cache-lru.wat`, which starts it and exercises put/get at
concrete types. They are your acceptance targets, not your edit targets.

## The work

Add, beside each existing `-tp` binding, a sibling that holds the parameter NAMES as a
`Vector` of `WatAST` symbol nodes:

```clojure
fqdn-tp-syms   ;; [] when monomorphic; [K V] (symbol nodes) when `fqdn-tp` is "<K,V>"
proto-tp-syms  ;; same, from proto-tp
```

- Derive them from the SAME source the string uses, so the two cannot disagree.
- `:wat::core::symbol-node` builds a symbol node from a string — the macro already uses it
  (`wat/service.wat:32` documents the idiom).
- A monomorphic service yields an EMPTY vector, not nil — the empty case is a first-class rung, the
  same rule the param-spec itself follows.
- Strip the brackets and split on `,`. **Trim each name**: `"<K, V>"` with a space is legal EDN-ish
  input today and would otherwise yield a symbol named `" V"`.

**Do not** change `fqdn-tp`, `proto-tp`, `fqdn-base`, `proto-base`, or any consumer of them. Both
representations live side by side until β-ii-b.

## The contract decision, pinned

**ONE derivation, shared by both blocks.** Do not write the split-and-symbolize logic twice. The
file already regrets this shape — `:345` says outright *"helper as `fqdn-base`/`fqdn-tp` above —
one spelling, two sides."* Add a `defn` (or a `let`-bound fn) that takes the bracketed suffix string
and returns the symbol vector, and call it from both blocks.

## STOP triggers

1. **STOP-1** — if any existing binding's VALUE changes, STOP. This stone is additive; the string
   path must be byte-identical.
2. **STOP-2** — if a golden moves or the floor goes red, STOP and report rather than adjusting the
   golden. A red here means something consumed the new value, which this stone does not do.
3. **STOP-3** — if `symbol-node` cannot build the nodes you need (wrong arity, wrong type, needs a
   span), STOP and report what it wants. Do not reach for `keyword/from-string` and convert — that
   is a different node KIND and the difference is load-bearing (stone 251.8's whole subject).
4. **STOP-4** — edit `wat/service.wat` ONLY. Not `wat/cache.wat`, not `wat-tests/`.

## Blast radius

`wat/service.wat` — two new bindings and one shared helper. No emission. No other file.

## How this lands

You are a rider. **Text edits only.** Do not run cargo, do not build, do not commit, stash, or
revert. Run everything FOREGROUND; your turn ends when the edits are on disk and the report is
written, and ending your turn ends you.

⚠ **You cannot test this edit.** `wat/service.wat` is baked into the binary by `include_str!` at
RUST-compile time (`src/stdlib.rs`), so `--check` reflects the LAST BUILD, not your edit — it will
report stale results and say so. This is not a limitation to route around; it is why the
orchestrator builds. Trace your correctness by reading, the way β-i's rider did, and say plainly in
your report what you verified by reading versus what you could not verify at all.

Report: the diff; the exact derivation you used; how the empty/monomorphic case is represented; and
anything on disk that contradicts this brief.
