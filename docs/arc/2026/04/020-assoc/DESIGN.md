# Arc 020 — `:wat::core::assoc` primitive

**Status:** opened 2026-04-22.
**Motivation:** wat ships HashMap construction via
`(:wat::std::HashMap :(K,V) k1 v1 ...)` and lookup via
`(:wat::std::get m k)`, but no way to return a new HashMap with
an entry added. That gap blocks every values-up state-threading
pattern that keys persistent data by string — first concrete
surface: the trading lab's Phase 3.3 `scaled-linear` helper,
which threads a `HashMap<String, ScaleTracker>` through encoding
calls.

Builder framing:

> let's go extend wat - hashmap without put means its defective
>
> did we just create assoc?.. yeah.. assoc feels good

Single-primitive arc. Tight scope.

---

## Naming — `:wat::core::assoc`, not `:wat::std::HashMap::put`

Design evolved mid-slice in two passes:

**First catch** — the existing HashMap surface has NO type-scoped
methods. Constructor is the bare type path (`:wat::std::HashMap`);
accessor is a generic stdlib function (`:wat::std::get container
locator`). `HashMap::put` would introduce type-scoped method
naming (`Type::method`) that isn't there and break symmetry with
`get`. Moved to generic-name form: `:wat::std::put container k v`.

**Second catch** — the builder observed that `put` IS Clojure's
`assoc` semantically. Wat already ships `:wat::core::conj` (arc
004, Clojure-named) for Vec append. Lineage discipline says match
the existing Clojure name: **`:wat::core::assoc`**. Pairs with
`:wat::core::conj`; both values-up, both Clojure-authentic.

(The asymmetry `:wat::std::get` vs `:wat::core::conj` stays — `get`
is at std because it's a generic dispatched accessor; `conj` is at
core because it's a construction primitive. `assoc` is
construction, so core.)

---

## UX target

```scheme
(:wat::core::let*
  (((m0 :rust::std::collections::HashMap<String,i64>)
    (:wat::std::HashMap :(String,i64)))           ;; empty
   ((m1 :rust::std::collections::HashMap<String,i64>)
    (:wat::core::assoc m0 "count" 1))             ;; {count: 1}
   ((m2 :rust::std::collections::HashMap<String,i64>)
    (:wat::core::assoc m1 "count" 2)))            ;; {count: 2}  (overwrite)
  (:wat::std::get m2 "count"))                    ;; → :Some 2
```

Signature: `∀K, V. HashMap<K,V> × K × V -> HashMap<K,V>`. Returns
a new HashMap; the input is unchanged (values-up; no mutation).
Duplicate keys overwrite.

---

## Non-goals

- **`dissoc` (HashMap remove), `contains-key?`, `keys`, `values`,
  `len`, `is-empty?`.** Each a valid future addition; ship when a
  caller surfaces demand. `assoc` is what the lab's Phase 3.3
  needs. Others come when required.
- **HashSet insert via `assoc`.** Clojure uses `conj` for set
  insert (same as Vec append). If wat needs HashSet insert later,
  extend `:wat::core::conj`'s dispatch to handle HashSet alongside
  Vec.
- **Mutation-in-place.** Substrate is values-up; `assoc` returns
  a new HashMap. Inner clone is a cheap shallow copy of hash
  buckets.

---

## What this arc ships

One slice.

- `src/check.rs` — `:wat::core::assoc` dispatch arm plus
  `infer_assoc` following the existing `infer_get` shape.
  Unifies key-ty with K, value-ty with V; returns the input
  HashMap type. For now rejects non-HashMap containers with a
  TypeMismatch; extends if other containers want `assoc` later.
- `src/runtime.rs` — dispatch arm plus `eval_assoc`.
  Clones the inner `std::collections::HashMap`, inserts under the
  canonical key, returns a new `Value::wat__std__HashMap`.
- Unit tests — round-trip via assoc + get, overwrite semantics,
  parametric types, arity + type mismatches.
- `docs/arc/2026/04/020-assoc/` — DESIGN + closing INSCRIPTION.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  row.

---

## Resolved design decisions

- **2026-04-22** — **`:wat::core::assoc`, Clojure-named.** Pairs
  with `:wat::core::conj` (arc 004). Builder confirmed mid-slice:
  "did we just create assoc?.. yeah.. assoc feels good."
- **2026-04-22** — **Generic name, not type-scoped.** No
  `HashMap::put`-style method naming — the existing HashMap
  surface doesn't use it.
- **2026-04-22** — **Values-up.** `assoc` returns a new HashMap;
  overwrite semantic on duplicate keys matches Rust's
  `HashMap::insert`.
- **2026-04-22** — **HashMap-only for now.** Extends to other
  container types if demand surfaces, following `infer_get`'s
  dispatch-on-container pattern.

---

## What this arc does NOT ship

- `dissoc` / `contains-key?` / `keys` / `values` / `len` / `is-empty?`.
- HashSet / Vec assoc (Vec insertion would need index, not key).
- Concurrent / atomic HashMap variants.
- A type-scoped method family under `:wat::std::HashMap::*`.
