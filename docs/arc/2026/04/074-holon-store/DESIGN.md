# Arc 074 — `Hologram<V>` + `HologramLRU<V>`: coordinate-cell cache with cosine readout

**Status:** PROPOSED 2026-04-28. Pre-implementation reasoning artifact.

**Predecessors:**
- Arc 023 — `coincident?` and the substrate's sigma machinery.
- Arc 037 — per-d encoders + sigma per d.
- Arc 057 — typed HolonAST leaves.
- Arc 058 — `HashMap<HolonAST, V>` as the substrate's structural index. **Critically:** 058-030 explicitly drops trait/impl/subtype machinery — polymorphism is enum-wrapping + per-type functions. This arc honors that.
- Arc 067 — flat default dim router (`DEFAULT_TIERS = [10000]`).
- Arc 073 slices 1-3 — term decomposition primitives. Useful shelf primitives; **not consumed by this arc**.

**Supersedes:** Arc 073 slice 4's `TermStore<V>` design. The cache use case turned out not to be a template-keyed bucket — it's a coordinate-keyed bucket with cosine-readout-against-the-population. See `../073-term-unification/DESIGN.md` for the trajectory; this arc is the operational primitive.

**Surfaced by:** Lab umbrella 059 slice 1's L1/L2 cache. Builder direction (2026-04-28, mid-arc-073-slice-4 build), refining the recognition that drove arc 073:

> "the position 1 claims domain over values between 0 and 1... position 2 claims domain over values between 1 and 2... when a get for a pos of 1.43 happens we map cosine over all the items in both 1's items and 2's items..."

> "the bucketing technique is identifying what asts are never capable of being queried... figuring out which capacity-locations are worth querying in"

The recognition: **the substrate's natural neighborhoods (sqrt(d) cells per d) ARE the pre-filter.** Asts in different cells live in different regions of the algebra grid; the get only needs to consider the cells adjacent to the probe's position. Within those cells, cosine-readout against a probe identifies the best match. Templates were the THEORETICAL framing for "which forms can possibly match"; coordinate cells are the OPERATIONAL mechanism.

---

## What this arc is, and is not

**Is:** two concrete substrate types, each implementing the same `new` / `put` / `get` / `len` method-name convention:
- `:wat::holon::Hologram<V>` — unbounded, no eviction; lives in **wat-rs core**.
- `:wat::holon::HologramLRU<V>` — bounded with LRU + per-cell-cap; lives in a **sibling crate** (`crates/wat-hologram-lru/`, adjacent to `crates/wat-lru/`).

**Is not** a trait abstraction. Per 058-030's deliberate stance: "No `deftype`. No `:is-a`. No `subtype`. No `impl`. No `trait`." Polymorphism in wat happens through **enum-wrapping** (see [Composition](#composition-enum-wrapping-per-058-030) below) or **per-type functions**, not implicit protocol. The two types share method names by **convention** so that a consumer who switches between them moves only the constructor; call sites stay verbatim.

**Is not** a query language. No range queries, no multi-key lookups, no set ops. Each store does one thing: given a probe and a coordinate, return the best-matching value (or None).

**Is not** template-aware. Templates may be useful for SOME consumers (those who want explicit structural reasoning); the arc-073 shelf primitives stay available. This arc doesn't consume them.

---

## The two types

### `Hologram<V>` — unbounded, in core

```
Hologram<V> {
  cells: Vec<HashMap<HolonAST, V>>    ; outer length = floor(sqrt(d)) cells
                                      ; inner unbounded
  num_cells: usize                    ; cached at construction
}
```

**Where:** wat-rs main, hand-coded substrate primitive next to `OnlineSubspace`, `Reckoner`, `Engram`, `EngramLibrary`. New `Value::Hologram(...)` variant; manual dispatch in `runtime.rs`; type schemes in `check.rs`.

**Why core:** No external dep. This is the substrate's primitive coordinate-cell store. If we later split it into a sibling crate, every consumer including HologramLRU (which composes it) breaks. Keeping Hologram in core means HologramLRU (which depends on `wat-lru` for the LRU sidecar) can ALSO depend on core's Hologram — same dependency direction wat-lru already established.

**Use case:** tests, simple consumers without memory pressure, workloads where unbounded growth is acceptable.

### `HologramLRU<V>` — bounded, in sibling crate

```
HologramLRU<V> {
  inner:  Hologram<V>                ; reuses core's bucketing + lookup
  lru:    LruCache<HolonAST, CellIdx> ; key tracker; tracks retrieval-rate
  per_cell_cap: usize                 ; user's pick at construction
  global_cap:   usize                 ; user's pick at construction (LRU's bound)
}
```

**Where:** new sibling crate `crates/wat-hologram-lru/`, adjacent to `crates/wat-lru/`. The crate composes both wat-lru's `LruCache` and core's `Hologram`. Pattern mirrors how wat-lru exposes `:rust::lru::LruCache` and `:wat::lru::LocalCache` typealias.

**Why sibling:** wat-rs core can't depend on wat-lru (which depends on core). HologramLRU uses LRU. So HologramLRU must live where wat-lru lives — sibling-to-core, depending on both core and wat-lru.

**Eviction policy — two bounds:**

1. **Global LRU** — fires on retrieval rate. When the LRU's `global_cap` is hit by a `put`, the LRU evicts its oldest entry; the evicted key's `CellIdx` tells us which cell to remove from. `cells[idx].remove(&key)` → O(1) (HashMap).
2. **Per-cell cap** — fires when one cell fills before the global LRU. Eviction policy: drop the entry in this cell whose key has the OLDEST LRU rank (a partial scan, but the cell is bounded by `per_cell_cap`). One LRU, two eviction triggers.

`get` ALSO bumps the matched key's freshness in the LRU — retrieval rate is what keeps cells warm.

---

## Operations

Both types share the same method names (the convention). Behavior diverges where eviction is concerned.

### `(put pos key val)`

For both types:
1. `idx = pos_to_cell_index(pos, d)` — pos validation runs first; rejects illegal pos with `RuntimeError`.
2. `cells[idx].insert(key, val)` — HashMap insert; idempotent on existing key (overwrite is fine).

For `HologramLRU` only, additionally:
- `lru.put(key, idx)` — bumps freshness if key exists; on global LRU eviction, the evicted key's idx tells us which cell to clean.
- If `cells[idx].len() > per_cell_cap`, drop one cell entry: scan the cell, find the entry whose key has the oldest LRU rank, remove it.

Returns `:()` (unit). Mutates in place.

### `(get pos probe filter)`

For both types:
1. `left = floor(pos)`, `right = ceil(pos)` after pos validation; mapped through `pos_to_cell_index` to actual cell indices.
2. `candidates = cells[left].iter() ++ cells[right].iter()` (flat-map over both cells; if `left == right`, one cell).
3. `probe_vec = encode(probe)` — at the form's d (router-picked).
4. For each `(stored_key, val)` in candidates:
   - `cos = cosine(encode(stored_key), probe_vec)`
   - Track the highest cosine seen.
5. If best cosine satisfies `filter(best_cosine) == true`, return `Some(best_val)`. Else return `None`.

For `HologramLRU` only, additionally:
- If returning `Some`, bump the matched key in the LRU (it just got retrieved).

`filter` is a user-supplied `:fn(:f64) -> :bool`. The substrate provides two opinionated defaults — both as functions parameterized by `d`:

- `(:wat::holon::Hologram::presence-filter d)` — looser. Returns a closure: `(λ (cos) cos > presence_floor(d))`.
- `(:wat::holon::Hologram::coincidence-filter d)` — stricter. Returns a closure: `(λ (cos) (1 - cos) < coincident_floor(d))`.

(The `Hologram::` namespace is shared by HologramLRU via the convention — both call sites use the same filter; only the constructor differs.)

Users compose their own filter funcs for non-default thresholds.

### `(len)`

Returns the total entry count across all cells as `:i64`. Read-only; doesn't bump LRU order.

### `(new ...)`

`Hologram::new` takes no arguments — uses the ambient dim router to determine `num_cells`.

`HologramLRU::new global-cap per-cell-cap` takes both bounds as user-chosen parameters.

---

## Cell indexing

The user supplies `pos: f64` normalized to `[0, 100]` always — regardless of d.

The substrate maps that to a cell index based on the encoder's d:

```
num_cells = floor(sqrt(d))
cell_idx = floor(pos * num_cells / 100)
```

At d=10000 → num_cells=100 → `cell_idx = floor(pos)`. User's [0,100] aligns naturally.
At d=4096 → num_cells=64 → `cell_idx = floor(pos * 0.64)`.
At d=1024 → num_cells=32 → `cell_idx = floor(pos * 0.32)`.

The user's [0,100] convention is stable; the substrate handles the d-specific arithmetic.

**Pos validation (Q4 locked):** the substrate REJECTS illegal `pos` with `RuntimeError`. Callers play by the rules. Specifically:

- `pos < 0.0` or `pos > 100.0` → `RuntimeError::InvalidArgument` with message naming the bad value.
- `pos.is_nan()` → `RuntimeError::InvalidArgument`.
- `pos == 100.0` is legal; maps to the last cell (`num_cells - 1`).

No silent clamping. Out-of-range input is a caller bug; surfacing it loudly beats absorbing it.

For `get`, we also need `left` and `right` (the spread):

```
left  = floor(pos * num_cells / 100)
right = ceil (pos * num_cells / 100)
;; both clamped to num_cells - 1 if pos == 100.0 exactly
;; if pos lands at a cell boundary, left == right; we touch one cell
```

---

## Composition (enum-wrapping per 058-030)

A consumer who wants to be generic over both store types — e.g., a test harness that sometimes uses Hologram and sometimes HologramLRU — composes via the enum-wrapping pattern 058-030 established as the substrate's polymorphism mechanism:

```scheme
(:wat::core::enum :my::store::AnyStore<V>
  (Hash  (:wat::holon::Hologram<V>))
  (Cache (:wat::holon::HologramLRU<V>)))

(:wat::core::define
  (:my::store::put<V>
    (s :my::store::AnyStore<V>)
    (pos :f64)
    (key :wat::holon::HolonAST)
    (val :V)
    -> :())
  (:wat::core::match s -> :()
    ((Hash  h) (:wat::holon::Hologram::put  h pos key val))
    ((Cache c) (:wat::holon::HologramLRU::put c pos key val))))
```

Closed variant set; explicit dispatch; no implicit protocol. Same shape `:wat::holon::HolonAST` itself uses (Atom is a *variant*, not a *subtype*).

**This arc does NOT ship `AnyStore` — that's a consumer-side construct.** Each consumer wraps the variants they care about. The lab cache, for instance, never wraps — it commits to `HologramLRU` directly.

---

## What the consumer owns

**`pos` computation.** The substrate gives the user the bucketing once they have a coordinate; computing the coordinate from a form is consumer-side. Different consumers compute differently:

- Cosine readout against a reference vector (proof 019's pattern)
- SimHash-like positional hashing
- Domain-specific projection (e.g., the trader projects on price-trajectory)

The substrate doesn't pick a default. The user passes `pos` in `[0, 100]`.

**Filter func.** Substrate provides `presence-filter` and `coincidence-filter`; user picks one or supplies their own.

**`per_cell_cap` and `global_cap` (HologramLRU only).** User chooses based on memory budget and expected cell density.

**`val` shape.** Parametric over V. The trader's two caches use `V = HolonAST` (chain-walking returns the next form to feed back). Other consumers use `V = wat::holon::Vector` (encoded result), `V = SomeStruct`, etc.

---

## Slice plan

**Slice 1 — `Hologram<V>` in core.** Hand-coded substrate primitive. New `Value::Hologram(Arc<ThreadOwnedCell<...>>)` variant. Methods: `new`, `put`, `get`, `len`, plus `presence-filter` / `coincidence-filter` factory funcs. Tests cover pos-to-cell math at multiple d, cosine-readout, filter compositions, illegal-pos rejection.

**Slice 2 — `HologramLRU<V>` in `crates/wat-hologram-lru/`.** New sibling crate following `crates/wat-lru/`'s pattern. Composes `wat::holon::Hologram` (core) and `wat::lru::LocalCache` (wat-lru). Methods: `new`, `put`, `get`, `len`. Tests cover global-LRU eviction, per-cell-cap eviction, retrieval-rate freshness, cell discovery on eviction.

The trader's cache slice (lab umbrella 059 slice 1) lands on slice 2 directly.

**No slice 3.** `HolonDatabase` was discussed and dropped — out of scope for arc 074. If durable storage surfaces as a need, a future arc specs it (likely `:wat::holon::HolonDatabase` in another sibling crate gated by `rusqlite`).

---

## What this arc deliberately does NOT do

- **Trait machinery.** Per 058-030's deliberate stance. Two concrete types; consumers compose via enum-wrapping. If a future consumer surfaces a hard need for runtime polymorphism over an abstract `HolonStore`, that's a separate spec — and it'd be reversing 058's no-traits decision, which is a substrate-philosophy debate, not a one-arc lift.
- **Pos-computation primitives.** Different consumers compute pos differently; the substrate doesn't pick. Future arc if a "default pos function for HolonAST" emerges.
- **Cross-process / cross-machine coordination.** Tier 3 (program-owned) services compose around the in-memory store; this arc ships the per-thread Tier 2 primitive.
- **Range queries / set ops.** `Hologram` / `HologramLRU` do single-key lookup. Range queries over pos-cells are a future-arc spec.
- **Custom eviction policies.** LRU + per-cell-cap is the only HologramLRU shape. If a consumer wants LFU, it ships as a separate type in a future arc.
- **Database / persistence.** Out of scope.

---

## What this unblocks

- **Lab umbrella 059 slice 1** — directly. Two `HologramLRU<HolonAST>` instances (next-cache, terminal-cache) plus one `HologramLRU<wat::holon::Vector>` (encode-cache). The trader's hot path consumes them.
- **Future engram library** — engrams ARE position-keyed populations of exemplars. `Hologram<Engram>` is the natural fit.
- **MTG / truth-engine domains** — same primitives; different V; same lookup pattern.
- **Cross-domain reuse** — anywhere a consumer wants "fuzzy lookup keyed by coordinate" lands here.

---

## Test strategy

### `Hologram<V>` (slice 1)

- T1 — put+get roundtrip: pos in cell N retrieves the val.
- T2 — pos spread: probe at cell-boundary pos finds entries from both adjacent cells.
- T3 — cosine readout: probe-vector cosine against stored-key-vector picks highest match.
- T4 — filter rejection: presence-filter rejects below-floor matches; same probe with coincidence-filter rejects more strictly.
- T5 — different-cell isolation: entries in distant cells don't pollute each other.
- T6 — pos-to-cell math: at d=10000, pos=50.0 → cell 50; at d=4096, pos=50.0 → cell 32; at boundaries.
- T7 — illegal pos rejected: pos < 0, pos > 100, NaN — each surfaces `RuntimeError::InvalidArgument`.
- T8 — len counts across cells.

### `HologramLRU<V>` (slice 2)

- T9 — global LRU eviction: filling past `global_cap` drops the oldest-retrieved entry.
- T10 — per-cell-cap eviction: filling one cell past `per_cell_cap` drops one entry from that cell (the oldest by LRU rank).
- T11 — get bumps freshness: retrieved entries stay warm; stale entries get evicted preferentially.
- T12 — cell discovery on eviction: LRU evicts → cell.remove(key) → entry actually gone from get.
- T13 — composition with Hologram: HologramLRU's underlying Hologram behavior matches slice 1's tests.

### Filter funcs (slice 1)

- T14 — presence-filter at known cosine value passes / fails as expected at d=10000.
- T15 — coincidence-filter is stricter than presence-filter on the same input.
- T16 — user-supplied filter: arbitrary `:fn(:f64) -> :bool` works.

---

## The thread

- **Arc 023** — `coincident?` predicate. The substrate's "are these the same point" / "is this signal present" machinery.
- **Arc 037** — per-d encoders + sigma machinery. The presence/coincident floor at every d.
- **Arc 057** — typed HolonAST leaves; algebra closed under itself.
- **058-030** — deliberate no-traits stance. This arc honors it.
- **Arc 067** — single-tier dim router (`DEFAULT_TIERS = [10000]`); arc 074 reads d from the router for cell-count math.
- **Arc 073** — term decomposition (template/slots/ranges/matches?). Shelf primitives; arc 074 doesn't consume them but they remain useful for explicit-template-reasoning consumers.
- **Proof 018** — `wat-tests-integ/experiment/022-fuzzy-on-both-stores/` — flat-fuzzy reference impl. Surfaced the categorical flaw that arc 073 attempted to fix structurally; arc 074 fixes it coordinate-wise.
- **Proof 019** — `wat-tests-integ/experiment/023-population-cache/` — cosine-readout-over-flat-population prototype. The architectural claim arc 074 makes operational.
- **2026-04-28 (mid-arc-073-slice-4 build)** — the recognition: "the bucketing technique is identifying what asts are never capable of being queried." Coordinate cells ARE the substrate's pre-filter, in operational form.
- **Arc 074 (this)** — the coordinate-cell cache primitives made concrete, as two types not a trait.
- **Next** — slice 1 ships `Hologram<V>` in core; slice 2 ships `HologramLRU<V>` in `crates/wat-hologram-lru/`. Lab umbrella 059 slice 1 consumes the latter.

PERSEVERARE.
