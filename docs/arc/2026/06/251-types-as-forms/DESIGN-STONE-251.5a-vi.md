# DESIGN — Stone 251.5a-vi: `fix-source`, the recursive role-inversion (THE HEART)

**Status: feasibility mapped; first rule STRIKE-READY target. The fixer's will, written in wat.**

`fix-source : :wat::WatAST -> :wat::WatAST` recursively walks a form tree (read by
`read-string`) and rewrites each node per the role-inversion table, rebuilding faithfully
(`with-children`) so only what a rule changes changes. Written ENTIRELY IN WAT over the now-
complete bridge (read-string · write-forms · ast->children · with-children · ast-kind ·
ast-name · symbol-node · keyword-node).

## The walk (the skeleton — grows rule by rule, each probe-gated)

```
(defn fix-source [node :- :wat::WatAST] :- :wat::WatAST
  (cond
    ;; --- leaf rules (token swaps; no new primitive) ---
    (and (= (ast-kind node) "symbol") (= (ast-name node) "<-"))  (keyword-node ":-")
    (and (= (ast-kind node) "symbol") (= (ast-name node) "->"))  (keyword-node ":-")   ; later
    ;; --- structural: recurse + rebuild SAME kind ---
    (structural? node)  (with-children node (map fix-source (ast->children node)))
    ;; --- otherwise unchanged ---
    :else node))
```

`structural?` ≡ `(ast-kind node)` ∈ {`"list"`,`"vector"`,`"set"`,`"map"`}. The recursion is the
proven `ast->children`→`map`→`with-children` cycle; `with-children` preserves kind so a binder
`[x <- T]` stays a Vector while its `<-` child becomes `:-`.

## The feasibility map (crawl 2026-06-09 — what each rule needs)

wat string vocab present: `split` `join` `concat` `length` `starts-with?`/`ends-with?`/`contains?`
`trim`. **Absent: `substring`/`slice`/`replace`.** So, rule by rule:

| rule | from → to | needs | when |
|---|---|---|---|
| binder/return arrow | `<-` / `->` (Symbol) → `:-` (Keyword) | equality + `keyword-node` — **on the shipped bridge** | **251.5a-vi FIRST** |
| call head | `:wat::core::map` (Keyword) → `wat.core/map` (Symbol) | the INVERSE of the resolver's `ns_to_wat_path` — exposed as a thin Rust primitive `keyword-head->symbol`, NOT wat string-surgery | 251.5a-vii |
| scalar type | `:wat::core::i64` / `:i64` → `wat.type/i64` | same inverse-path primitive (type ns) | 251.5a-vii |
| rust interop | `:rust::a::b::C` → `rust.a.b/C` | same inverse-path primitive | 251.5a-vii |
| parametric / fn type | `:wat::core::Vector<T>` / `:…::Fn(A)->R` | in-keyword-body string surgery (`<`,`>`,`(`,`)` inside one keyword token) — needs `substring`/`split-on-char`; the hardest | 251.5a-viii (LAST) |

**Decomplect note (load-bearing):** the `::`↔`./` path grammar lives in ONE place — the
resolver's `ns_to_wat_path` (`edn_shim`). The fixer must call its INVERSE, never re-encode the
grammar in wat string ops; re-encoding would be a duplicated-encoding braid (same class the
251.1 keystone pulled out). This is WHY the head/type rewrites get a Rust primitive, not wat
surgery — it is the decomplected choice, not a capability gap.

## The contract decision (pinned, this stone)

251.5a-vi ships ONLY the leaf-swap rule (`<-`→`:-`) + the structural recursion + `:else`
passthrough — the smallest transform that proves the recursive walk end-to-end on a real
program. `->` , the head/type/rust rules, and body surgery are affirmatively OUT (named stones
above), not deferred-without-a-home.

## Where fix-source lives

A wat source file — proposed `wat/fix.wat` (a new corpus file, itself clojure-faithful). For
the PROBE, define `fix-source` inline in the test's startup source (proves it without committing
a home yet); the home is pinned when 251.5b drives the corpus.

## Probe (RED at HEAD target)

`tests/probe_arc251_stone5a_fix_source.rs`: define `fix-source` (leaf-swap + recursion) inline;
- C01: `(write-forms (fix-source (read-string "[a <- b]")))` contains `":-"` and NOT `"<-"`.
- C02: nesting — `<-` deep inside `(f [x <- T])` is rewritten (proves recursion reaches it).
RED at HEAD? No — every primitive exists; the probe is GREEN-on-build IF the wat skeleton is
correct. So this stone's "probe" is a forward proof of the wat transform, not a missing-Rust
disconfirmer: the risk is the wat skeleton (cond/map/recursion shape), and the probe pins it.

## Gate

- `cargo test --release --test probe_arc251_stone5a_fix_source` → green.
- Full 5a spine still green; suite unchanged (the 4 nursery deadlock-reds only).
