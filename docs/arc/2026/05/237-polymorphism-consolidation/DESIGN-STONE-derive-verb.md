# DESIGN — Stone (237 follow-on): the user-facing `:wat::core::derive` verb

> Opened 2026-06-14 as the arc-237-named follow-on. Stone S-A built the `typesub` hierarchy
> (Clojure's `isa?`/`derive` axis) + `is_subtype` + `:wat::core::subtype?`, seeded edges only in Rust
> roots, and EXPLICITLY deferred the user-facing verb: *"a user-facing derive verb ships only when a
> caller needs it"* (DESIGN-STONE-S-A:178). The arc-209 host seam is that caller — the spawn handles
> need a **marker bound** (`:Spawned`) they derive, with NO methods. Grounded against HEAD `0a649ffd`.

## Why derive, not a protocol

The handle bound is a **marker** ("this is a spawn handle"), not behaviour. Clojure separates these
axes and wat mirrors it: `typeunion` (closed sum) · `defprotocol` (methods/behaviour) · `typesub`
(`isa?`/`derive` taxonomy). A marker belongs on the `typesub` axis. A zero-method protocol (deviation
from Clojure) or a fake method (dishonest) were both rejected; `derive` is Clojure's own verb — the
faithful default, no high bar to clear.

## What it delivers

```wat
(:wat::core::derive :Child :Parent)   ;; registers a typesub edge Child→Parent; no methods
```
A `:Parent`-typed binder then accepts any deriver (`is_subtype` consults the edge; for a parametric
child like `Thread'<I,O>`, the arc-267 `assignable` head-arm already applies). A new transport's
handle joins by `(derive :RemoteHandle' :Spawned)` — zero central edit (the organic-evolution
requirement).

## Grounded facts (from the FM-2-bis probe + crawl)

- `:wat::core::derive` does not exist (`grep` empty in `src/`). It's silently swallowed today by the
  permissive `:wat::` check path (check.rs:5466 "silent-by-intent"), so it no-ops → the probe is RED
  (edge absent → bound rejects).
- **A marker name needs no separate type declaration.** The probe's `m <- :t::Marker` did NOT error
  as unknown-type — annotations resolve permissively; only the *edge* drives assignability. So
  `derive` need only register the edge; `:Spawned` is usable as a bound with no `TypeDef`.

## The one contract decision

`derive` is **`extend-type`'s edge-registration half, minus the method-impls and minus the
protocol requirement.** It registers a `typesub` edge via `register_subtype` (the same call
`extend-type` and the Rust roots use, types.rs:450/1571), at the same pre-check point so `assignable`
sees it. It is a **declaration form** (like `declare-acronyms` / `defprotocol` / `extend-type`):
type-checks to unit, no runtime artifact beyond the edge. The cycle check in `register_subtype`
already rejects a derive that would close a cycle.

## Build shape (mirror the precedents)

1. `parse_derive_form(form) -> (child, parent)` — two keyword args (runtime.rs; model
   `parse_extend_type_form` minus the method-impl loop).
2. Register the edge at the same pre-expansion/splice point `extend-type` uses
   (`env.register_subtype(&child, &parent, span)` — types.rs:1571 is the extend-type site; mirror it).
3. A check-side arm so `(:wat::core::derive …)` type-checks as a declaration returning unit (model the
   `extend-type` / `declare-acronyms` arms in `infer_list` + `collect_splice_defs_ctx`).

## Scope / out

- **No methods, no dispatch** — that's `extend-type`/`defprotocol`. `derive` is pure taxonomy.
- **`:Spawned` + deriving the handles + wiring defservice's `Handle`** — the NEXT stone (this one
  ships the verb + proves it on plain Records). Keeps the verb provable in isolation.
- **No change to `is_subtype`/`subtype?`/`register_subtype`/the 267 `assignable` arm** — all consumed
  as-is.

## Probe

`tests/probe_arc237_derive_verb.rs` (committed RED) — two Records derive `:t::Marker`; a fn
`[m <- :t::Marker]` accepts both. RED at HEAD (derive no-ops → edge absent). GREEN when the edge
registers.
