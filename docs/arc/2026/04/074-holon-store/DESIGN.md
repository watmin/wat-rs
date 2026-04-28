# Arc 074 — `HolonStore<V>`: coordinate-cell cache with cosine readout

**Status:** PROPOSED 2026-04-28. Pre-implementation reasoning artifact.

**Predecessors:**
- Arc 023 — `coincident?` and the substrate's sigma machinery.
- Arc 037 — per-d encoders + sigma per d.
- Arc 057 — typed HolonAST leaves.
- Arc 058 — `HashMap<HolonAST, V>` as the substrate's structural index.
- Arc 067 — flat default dim router (`DEFAULT_TIERS = [10000]`).
- Arc 073 slices 1-3 — term decomposition primitives. Useful shelf primitives; **not consumed by this arc**.

**Supersedes:** Arc 073 slice 4's `TermStore<V>` design. The cache use case turned out not to be a template-keyed bucket — it's a coordinate-keyed bucket with cosine-readout-against-the-population. See `../073-term-unification/DESIGN.md` for the trajectory; this arc is the operational primitive.

**Surfaced by:** Lab umbrella 059 slice 1's L1/L2 cache. Builder direction (2026-04-28, mid-arc-073-slice-4 build), refining the recognition that drove arc 073:

> "the position 1 claims domain over values between 0 and 1... position 2 claims domain over values between 1 and 2... when a get for a pos of 1.43 happens we map cosine over all the items in both 1's items and 2's items..."

> "the bucketing technique is identifying what asts are never capable of being queried... figuring out which capacity-locations are worth querying in"

The recognition: **the substrate's natural neighborhoods (sqrt(d) cells per d) ARE the pre-filter.** Asts in different cells live in different regions of the algebra grid; the get only needs to consider the cells adjacent to the probe's position. Within those cells, cosine-readout against a probe identifies the best match. Templates were the THEORETICAL framing for "which forms can possibly match"; coordinate cells are the OPERATIONAL mechanism.

---

## What this arc is, and is not

**Is:** a substrate primitive — `HolonStore<V>` trait + concrete impls — that exposes a coordinate-cell cache with cosine-readout retrieval. The lab cache slice (umbrella 059 slice 1) consumes one of the impls directly.

**Is not:** a query language. No range queries, no multi-key lookups, no set ops. The store does one thing: given a probe and a coordinate, return the best-matching value (or None).

**Is not:** template-aware. Templates may be useful for SOME consumers (those who want explicit structural reasoning), but they're not load-bearing for cache hit/miss decisions. The arc-073 shelf primitives stay available; this arc doesn't consume them.

---

## The shape

### Trait

```scheme
(:wat::core::trait :wat::holon::HolonStore<V>
  ((put (pos :f64) (key :wat::holon::HolonAST) (val :V) -> :())
   (get (pos :f64) (probe :wat::holon::HolonAST) (filter :fn(:f64) -> :bool)
        -> :Option<V>)))
```

One signature; three concrete impls below.

### Impl 1 — `HolonHash<V>` (unbounded, no eviction)

```
HolonHash<V> {
  cells: Vec<HashMap<HolonAST, V>>    ; outer length = sqrt(d) cells; inner unbounded
}
```

Use when the consumer wants a simple coordinate-keyed dictionary with cosine readout — no LRU, no eviction, total memory grows with put rate. Simplest impl; useful for tests and for consumers that manage their own memory pressure externally.

### Impl 2 — `HolonCache<V>` (bounded, LRU + per-cell-cap)

```
HolonCache<V> {
  cells: Vec<HashMap<HolonAST, V>>    ; outer sqrt(d); inner per-cell-cap'd
  lru:   LruCache<HolonAST, CellIdx>  ; key tracker; tracks retrieval-rate
  per_cell_cap: usize                 ; user's pick at construction
}
```

The default cache shape. Two bounds:

1. **Global LRU** — fires on retrieval rate. When the LRU's global cap is hit by a `put`, the LRU evicts its oldest entry; the evicted key's `CellIdx` tells us which cell to remove from. `cells[idx].remove(&key)` → O(1) (HashMap).
2. **Per-cell cap** — fires when one cell fills before the global LRU. Eviction policy: drop the entry in this cell whose key has the OLDEST LRU rank (a partial scan, but the cell is bounded by `per_cell_cap`). One LRU, two eviction triggers.

`get` ALSO bumps the matched key's freshness in the LRU — retrieval rate is what keeps cells warm.

### Impl 3 — `HolonDatabase<V>` (durable, SQLite-backed)

```
HolonDatabase<V> {
  handle: rusqlite::Connection
}
```

Same trait shape; SQL-backed cells; persists across process restarts. Out of scope for slice 1 ship — recorded here so the trait is forward-compatible. Future arc when persistence surfaces.

---

## Operations

### `(put pos key val)`

1. `idx = pos_to_cell_index(pos, d)`
2. `cells[idx].insert(key, val)` — HashMap insert; idempotent on existing key (overwrite is fine).
3. **HolonCache only:**
   - `lru.put(key, idx)` — bumps freshness if key exists; on global LRU eviction, the evicted key's idx tells us which cell to clean.
   - If `cells[idx].len() > per_cell_cap`, drop one cell entry: scan the cell, find the entry whose key has the oldest LRU rank, remove it.

Returns `:()` (unit). Mutates in place.

### `(get pos probe filter)`

1. `left = floor(pos)`, `right = ceil(pos)` — clamped to `[0, num_cells - 1]`.
2. `candidates = cells[left].iter() ++ cells[right].iter()` (flat-map over both cells; if `left == right`, one cell).
3. `probe_vec = encode(probe)` — at the form's d (router-picked).
4. For each `(stored_key, val)` in candidates:
   - `cos = cosine(encode(stored_key), probe_vec)`
   - Track the highest cosine seen.
5. If best cosine satisfies `filter(best_cosine) == true`, return `Some(best_val)`. Else return `None`.
6. **HolonCache only:** if returning `Some`, bump the matched key in the LRU (it just got retrieved).

`filter` is a user-supplied `:fn(:f64) -> :bool`. The substrate provides two opinionated defaults:

- `(:wat::holon::HolonStore::presence-filter)` — looser. Returns `cos > presence_floor(d)`.
- `(:wat::holon::HolonStore::coincidence-filter)` — stricter. Returns `(1 - cos) < coincident_floor(d)`.

Each substrate-provided filter closes over the ambient `d` (looked up from the dim router at construction or at call time). Users compose their own filter funcs for non-default thresholds.

---

## Cell indexing

The user supplies `pos: f64` normalized to `[0, 100]` always — regardless of d.

The substrate maps that to a cell index based on the encoder's d:

```
num_cells = floor(sqrt(d))
cell_idx = clamp(floor(pos * num_cells / 100), 0, num_cells - 1)
```

At d=10000 → num_cells=100 → `cell_idx = floor(pos)` (with clamping). User's [0,100] aligns naturally.
At d=4096 → num_cells=64 → `cell_idx = floor(pos * 64 / 100)`. User's [0,100] gets compressed.
At d=1024 → num_cells=32 → `cell_idx = floor(pos * 32 / 100)`. Further compression.

The user's [0,100] convention is stable; the substrate handles the d-specific arithmetic.

For `get`, we also need `left` and `right` (the spread):

```
left  = clamp(floor(pos * num_cells / 100), 0, num_cells - 1)
right = clamp(ceil(pos * num_cells / 100),  0, num_cells - 1)
;; (if pos exactly at a cell boundary, left == right; we touch one cell)
```

---

## What the consumer owns

**`pos` computation.** The substrate gives the user the bucketing once they have a coordinate; computing the coordinate from a form is consumer-side. Different consumers compute differently:

- Cosine readout against a reference vector (proof 019's pattern)
- SimHash-like positional hashing
- Domain-specific projection (e.g., the trader projects on price-trajectory)

The substrate doesn't pick a default. The user passes `pos` in `[0, 100]`.

**Filter func.** Substrate provides `presence-filter` and `coincidence-filter`; user picks one or supplies their own.

**`per_cell_cap` (HolonCache only).** User chooses based on memory budget and expected cell density.

**`val` shape.** Parametric over V. The trader's two caches use `V = HolonAST` (chain-walking returns the next form to feed back). Other consumers use `V = wat::holon::Vector` (encoded result), `V = SomeStruct`, etc.

---

## Decisions to resolve

### Q1 — Where does `HolonStore<V>` live?

**Option a — wat-rs main, hand-coded substrate primitive.** Like `OnlineSubspace`, `Reckoner`, `Engram`, `EngramLibrary` — Value variant + manual dispatch in `runtime.rs`, type schemes in `check.rs`, no external crate. The trait expressed via the wat type system; the impls are concrete `Value::HolonHash<V>`, `Value::HolonCache<V>`, `Value::HolonDatabase<V>` variants (with phantom V at wat level, V → `Value` at Rust level).

**Option b — new sibling crate (`crates/wat-holon-store/`).** Like `crates/wat-lru/`, with its own `#[wat_dispatch]`-annotated impl. Cleaner if the database impl pulls in `rusqlite` (heavyweight dep we don't want in wat-rs main).

**Recommendation: Option a for slices 1-2 (HolonHash + HolonCache), Option b's path open for slice 3 (HolonDatabase) when SQL surfaces.** Hand-coded keeps the substrate-internal feel for the in-memory impls; the persistent impl can land in a sibling crate if/when needed.

### Q2 — Slice plan

**Slice 1 — `HolonHash<V>` (unbounded)** — minimal viable shape; tests cover the pos-to-cell math + cosine-readout + filter compositions.

**Slice 2 — `HolonCache<V>` (LRU + per-cell-cap)** — adds bounded variant. The trader's cache slice (lab umbrella 059 slice 1) lands on this directly.

**Slice 3 — `HolonDatabase<V>`** — durable variant. Out of scope for the immediate trader unblock; recorded for forward compatibility.

### Q3 — Trait machinery

Wat already supports `:wat::core::trait` (per `:wat::core::trait :wat::holon::HolonStore<V>` syntax in the user's spec). Confirm this exists and is fit-for-purpose for runtime polymorphism, OR establish that consumers branch at the call site (pick which impl directly; no dynamic dispatch).

**Open**: pinned by reading `src/check.rs` for trait-related TypeScheme support before slice 1 starts.

### Q4 — `pos` validation

User passes `pos: f64`. Should the substrate validate `0 ≤ pos ≤ 100`, or clamp silently?

**Recommendation: clamp at boundaries** — matches the user's "less than 1 → cell 1, greater than 99 → cell 99" framing. Out-of-range is treated as edge-cell. Garbage input (NaN) panics with a diagnostic.

---

## What this arc deliberately does NOT do

- **Pos-computation primitives.** Different consumers compute pos differently; the substrate doesn't pick. Future arc if a "default pos function for HolonAST" emerges.
- **Cross-process / cross-machine coordination.** Tier 3 (program-owned) services compose around the in-memory store; this arc ships the per-thread Tier 2 primitive.
- **Range queries / set ops.** `HolonStore` does single-key lookup. Range queries over pos-cells are a future-arc spec.
- **Custom eviction policies.** LRU + per-cell-cap is the only HolonCache shape. If a consumer wants LFU, it ships as a separate impl in a future arc.

---

## What this unblocks

- **Lab umbrella 059 slice 1** — directly. Two `HolonCache<HolonAST>` instances (next-cache, terminal-cache) plus one `HolonCache<wat::holon::Vector>` (encode-cache). The trader's hot path consumes them.
- **Future engram library** — engrams ARE position-keyed populations of exemplars. `HolonStore<Engram>` is the natural fit.
- **MTG / truth-engine domains** — same primitive; different V; same lookup pattern.
- **Cross-domain reuse** — anywhere a consumer wants "fuzzy lookup keyed by coordinate" lands here.

---

## Test strategy

### HolonHash<V>

- T1 — put+get roundtrip: pos in cell N retrieves the val.
- T2 — pos spread: probe at cell-boundary pos finds entries from both adjacent cells.
- T3 — cosine readout: probe-vector cosine against stored-key-vector picks highest match.
- T4 — filter rejection: presence-filter rejects below-floor matches; same probe with coincidence-filter rejects more strictly.
- T5 — different-cell isolation: entries in distant cells don't pollute each other.
- T6 — pos-to-cell math: at d=10000, pos=50.0 → cell 50; at d=4096, pos=50.0 → cell 32; clamping at boundaries.

### HolonCache<V>

- T7 — global LRU eviction: filling past LRU cap drops the oldest-retrieved entry.
- T8 — per-cell-cap eviction: filling one cell past its cap drops one entry from that cell (the oldest by LRU rank).
- T9 — get bumps freshness: retrieved entries stay warm; stale entries get evicted preferentially.
- T10 — cell discovery on eviction: LRU evicts → cell.remove(key) → entry actually gone from get.

### Filter funcs

- T11 — presence-filter at known cosine value passes / fails as expected at d=10000.
- T12 — coincidence-filter is stricter than presence-filter on the same input.
- T13 — user-supplied filter: arbitrary `:fn(:f64) -> :bool` works.

---

## The thread

- **Arc 023** — `coincident?` predicate. The substrate's "are these the same point" / "is this signal present" machinery.
- **Arc 037** — per-d encoders + sigma machinery. The presence/coincident floor at every d.
- **Arc 057** — typed HolonAST leaves; algebra closed under itself.
- **Arc 067** — single-tier dim router (`DEFAULT_TIERS = [10000]`); arc 074 reads d from the router for cell-count math.
- **Arc 073** — term decomposition (template/slots/ranges/matches?). Shelf primitives; arc 074 doesn't consume them but they remain useful for explicit-template-reasoning consumers.
- **Proof 018** — `wat-tests-integ/experiment/022-fuzzy-on-both-stores/` — flat-fuzzy reference impl. Surfaced the categorical flaw that arc 073 attempted to fix structurally; arc 074 fixes it coordinate-wise.
- **Proof 019** — `wat-tests-integ/experiment/023-population-cache/` — cosine-readout-over-flat-population prototype. The architectural claim arc 074 makes operational.
- **2026-04-28 (mid-arc-073-slice-4 build)** — the recognition: "the bucketing technique is identifying what asts are never capable of being queried." Coordinate cells ARE the substrate's pre-filter, in operational form.
- **Arc 074 (this)** — the coordinate-cell cache primitive made concrete.
- **Next** — lab umbrella 059 slice 1 consumes `HolonCache<HolonAST>` + `HolonCache<Vector>` directly. Cache slice ships as a thin wrapper.

PERSEVERARE.
