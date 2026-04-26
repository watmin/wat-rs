# wat-rs arc 058 — HashMap surface completion — INSCRIPTION

**Status:** shipped 2026-04-26. One slice, one commit, ~3 hours
of focused work matching the DESIGN's estimate.

Builder direction (2026-04-26):

> "let's go make wat core better - if its missing it shouldn't be -
> let's get an arc written for us to go build"

The arc closes the gap arcs 020 (assoc) and 021 (HashMap-to-core
namespace move) explicitly deferred: `dissoc`, `keys`, `values`,
plus the `empty?` polymorphism extension that should have shipped
with `length`'s polymorphism.

---

## What shipped

| Slice | Module | LOC | Tests | Status |
|-------|--------|-----|-------|--------|
| 1 | `src/check.rs` — 4 dispatch arms + 4 `infer_*` functions (`infer_dissoc`, `infer_keys`, `infer_values`, `infer_empty_q`); the fixed `:wat::core::empty?` Vec-only scheme registration retired. `src/runtime.rs` — 3 dispatch arms (`dissoc`/`keys`/`values`); `eval_vec_empty` rewritten as `eval_empty_q` (polymorphic). `docs/USER-GUIDE.md` — the polymorphism table gains `dissoc`/`keys`/`values`/`empty?` rows; the surface-table flat list gains four entries. | ~330 Rust + ~10 doc | 18 new (5 dissoc, 5 keys, 5 values, 3 empty?) | shipped |

**wat-rs unit-test count: 612 → 630. +18. Workspace: 0 failing.**

---

## Architecture notes

### Mechanical mirror of arc 020's shape

Each new `infer_*` function lifts arc 020's `infer_assoc` template:
arity check, container type inference, `reduce` + `apply_subst`,
`HashMap<K,V>` arm with key-type unification (where applicable),
TypeMismatch on non-HashMap inputs. Same `Some(reduce(&ct, ...))`
return discipline.

Each `eval_*` function follows `eval_assoc`: extract container,
clone (or read-borrow), mutate-the-clone (or gather), wrap in the
appropriate `Value::*` variant. Values-up; original input unchanged.

### `keys` / `values` walk the inner-tuple storage

Per arc 021's storage shape, `Value::wat__std__HashMap` holds
`Arc<HashMap<String, (Value, Value)>>` — the canonical-key string
keys the underlying map; the tuple carries both the original
key-Value (preserving the wat-side type) and the entry-Value. So
`keys` is `m.values().map(|(k, _v)| k.clone()).collect()` — read
the first slot of each tuple, not `m.keys()` which would give back
canonical strings.

### `empty?` polymorphism — same shape as `length`

`length` shipped polymorphic (arc 035) but `empty?` stayed Vec-only.
This arc moves `empty?` to the same dispatch model: `infer_empty_q`
returns `:bool` for any of `Vec<T>` / `HashMap<K,V>` / `HashSet<T>`,
errors otherwise. The runtime side is a 4-arm match on the `Value`.

### `dissoc` semantics

Mirrors Clojure: returns a new `HashMap` without the key; missing
key is no-op (clone of input). Original unchanged. No `Result`
return — there's no failure mode worth surfacing as a value
("missing key was a no-op" is the same shape as "key was removed";
both produce the new map).

### Order is unspecified

Rust's `std::collections::HashMap` iteration order depends on hash
randomization. `keys` / `values` reflect that. Callers wanting
deterministic order sort the resulting Vec post-call. The DESIGN
documents this; no pretense of insertion-order maps.

### What `contains-key?` would have been

A redundant alias. `contains?` is already polymorphic over HashMap
and answers "key present?" directly. Adding a Clojure-named
duplicate would split a clear concept across two surface names. The
USER-GUIDE polymorphism table makes this explicit.

---

## What this unblocks

- **Lab experiment 008 (Treasury program).** Treasury holds
  `HashMap<i64, Paper>` keyed by paper-id; per-tick `check-deadlines`
  iterates via `(values m)` to find expiring papers; resolution
  removes via `(dissoc m k)`; the "no brokers yet" guard uses
  `(empty? m)`. The full toolbox now exists.
- **Future programs holding HashMap state** — broker prediction
  caches, observer recalibration trackers, regime caches, etc.
  Each gets the same shape.
- **Arc 030 slice 2 (encoding cache).** `(length cache)` and
  `(empty? cache)` round out the cache-stats telemetry surface;
  `(keys cache)` would enable a "what's in the cache" introspection
  primitive if a future consumer wants it.

---

## What this arc deliberately did NOT add

Reproduced from DESIGN's "What this arc does NOT add":

- **HashMap merge / union / intersection.** Compositional ops;
  multi-arity. Future arc when a consumer surfaces.
- **HashSet completion.** `disjoin` (analog to dissoc for sets) +
  iteration. Separate small arc; same template as this one.
- **Lazy iteration / iterator type.** `keys` and `values`
  materialize a Vec. A separate Iterator-shaped substrate primitive
  is out of scope; build it when a consumer needs lazy ops.
- **Insertion-order maps.** Out; sort the result if you need order.
- **Capacity hints / `with-capacity`.** Defer until measurement
  shows allocation overhead matters.
- **Mut-cell HashMaps.** Substrate is values-up; mut would be a
  different surface (separate arc, maybe never).
- **`contains-key?` alias.** Q3 — `contains?` is already
  polymorphic.

---

## The thread

- **2026-04-25** — Treasury experiment (008) opens; surface gap
  surfaces as the Treasury layout starts using HashMap.
- **2026-04-26 (morning)** — DESIGN.md drafted; this arc opens.
- **2026-04-26 (this session)** — slice 1 lands in one commit:
  3 new ops + 1 polymorphism extension + 18 tests + USER-GUIDE
  rows + this INSCRIPTION.
- **Next** — Treasury layout (lab experiment 008) consumes the
  full surface; arc 030 slice 2 (encoding cache) picks up the
  unblocked rest of its telemetry surface.

PERSEVERARE.
