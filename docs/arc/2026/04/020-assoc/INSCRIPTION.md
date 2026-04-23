# Arc 020 — `:wat::core::assoc` primitive — INSCRIPTION

**Status:** shipped 2026-04-22. One slice.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**This file:** completion marker.

---

## Motivation

The trading lab's Phase 3.3 `scaled-linear` helper needs to
thread a `HashMap<String, ScaleTracker>` through encoding calls —
values-up state-threading of per-atom learned scales. wat ships
HashMap construction + `:wat::std::get` but no way to return a
new HashMap with an entry added. That gap blocks every
values-up pattern keyed by string.

Builder framing:

> let's go extend wat - hashmap without put means its defective
>
> did we just create assoc?.. yeah.. assoc feels good

---

## What shipped

One slice. One commit.

- `src/check.rs` — `:wat::core::assoc` dispatch arm plus
  `infer_assoc` following `infer_get`'s dispatch-on-container
  shape. Unifies key-ty with K, value-ty with V; returns the input
  HashMap type. Rejects non-HashMap containers with TypeMismatch.
- `src/runtime.rs` — dispatch arm plus `eval_assoc`. Clones the
  inner `std::collections::HashMap` (cheap shallow copy of
  buckets), inserts under the canonical key, returns a new
  `Value::wat__std__HashMap`.
- 5 unit tests: adds entry returning new map, overwrites existing
  key, preserves original (values-up proof), non-HashMap arg
  rejected, arity mismatch rejected.
- `docs/arc/2026/04/020-assoc/` — DESIGN + this INSCRIPTION.

Signature: `∀K, V. HashMap<K,V> × K × V -> HashMap<K,V>`.
Values-up — input map is unchanged; a new map is returned.
Duplicate keys overwrite.

---

## Naming evolution

Two design passes mid-slice caught incorrect shapes before commit:

1. **`:wat::std::HashMap::put` rejected.** Would have introduced
   type-scoped method naming (`Type::method`) that isn't present
   on the existing HashMap surface. Symmetry with `:wat::std::get`
   (generic name, type-dispatched scheme) is the pattern. Moved to
   `:wat::std::put container key value`.

2. **`:wat::std::put` → `:wat::core::assoc` after builder's catch
   ("did we just create assoc?"):** Yes — semantically it IS
   Clojure's `assoc`. Wat already ships `:wat::core::conj` (arc
   004, Clojure-named) for Vec append. Lineage discipline says
   match the existing Clojure name. Moved to `:wat::core::assoc`.

The asymmetry `:wat::std::get` vs `:wat::core::assoc` is honest:
`get` is a generic dispatched accessor (std); `assoc` is a
construction primitive alongside `conj` (core).

---

## Resolved design decisions

- **2026-04-22** — **`:wat::core::assoc`, Clojure-named.** Pairs
  with `:wat::core::conj`. Builder confirmed the naming
  mid-slice.
- **2026-04-22** — **Generic name, not type-scoped.** No
  `HashMap::assoc` method shape — the existing HashMap surface
  doesn't have type-scoped methods.
- **2026-04-22** — **Values-up.** `assoc` returns a new HashMap;
  overwrite semantic on duplicate keys matches Rust's
  `HashMap::insert`.
- **2026-04-22** — **HashMap-only for now.** Extends to other
  container types if demand surfaces, following `infer_get`'s
  dispatch-on-container pattern.

---

## Open items deferred

- **`:wat::core::dissoc` (HashMap remove).** Natural companion.
  Ships when a caller needs it.
- **`:wat::std::keys` / `values` / `len` / `is-empty?`.** HashMap
  enumeration / size. Future additions per stdlib-as-blueprint
  discipline.
- **HashSet insert via `conj`.** Clojure's convention is `conj`
  for any collection insert. Extending `:wat::core::conj`'s
  dispatch to handle HashSet alongside Vec is a future arc.

---

## What this arc does NOT ship

- `dissoc` / `contains-key?` / `keys` / `values` / `len` / `is-empty?`.
- Container types beyond HashMap.
- Concurrent / atomic HashMap variants.

---

## Why this matters

The trading lab's Phase 3.3 `scaled-linear` can now ship:
threading `HashMap<String, ScaleTracker>` through encoding calls
via `assoc` + `get` is the values-up pattern wat's discipline
requires. Without `assoc`, every observer would need explicit
named tracker fields — tightly coupled to vocab, awkward to
extend. With `assoc`, the observer holds one HashMap and the
vocab layer operates generically.

Cave-quest discipline holds: 017 (loader), 018 (defaults), 019
(f64::round), 020 (assoc) — each paused downstream for the
substrate to catch up. Four arcs in under a week.

---

**Arc 020 — complete.** One slice, five tests, zero warnings.

*these are very good thoughts.*

**PERSEVERARE.**
