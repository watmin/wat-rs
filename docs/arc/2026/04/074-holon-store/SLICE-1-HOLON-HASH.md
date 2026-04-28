# Arc 074 Slice 1 — `HolonHash` shipped

**Status:** SHIPPED 2026-04-28 at commit `3820009`.

**What landed:** `:wat::holon::HolonHash` — substrate-internal coordinate-cell store with cosine-readout retrieval. HolonAST → HolonAST. Unbounded, no eviction (slice 2 adds the bounded variant).

The `<V>` parametric in the original DESIGN dropped during slice-1 implementation review. The actual consumers (the trader's next-cache and terminal-cache) are both HolonAST → HolonAST. Encode-cache (HolonAST → Vector) was the one case that wanted V ≠ HolonAST — but it doesn't need cosine readout (form→Vector is a deterministic HashMap lookup), so it belongs in a different cache type, not `HolonHash`.

## Surface (six entry points)

| Op | Signature | What it does |
|----|-----------|--------------|
| `:wat::holon::HolonHash/new` | `(d :i64) -> :wat::holon::HolonHash` | Construct empty store sized for d. `num_cells = floor(sqrt(d))`. d=10000 → 100 cells. |
| `:wat::holon::HolonHash/put` | `(store :HolonHash) (pos :f64) (key :HolonAST) (val :HolonAST) -> :()` | Insert at cell determined by `pos`. Idempotent on existing key (overwrite). Mutates in place. |
| `:wat::holon::HolonHash/get` | `(store :HolonHash) (pos :f64) (probe :HolonAST) (filter :fn(:f64)->:bool) -> :Option<HolonAST>` | Walk adjacent cells; encode candidates; cosine vs probe; apply filter to best; return matched val or None. |
| `:wat::holon::HolonHash/len` | `(store :HolonHash) -> :i64` | Total entries across all cells. Read-only. |
| `:wat::holon::presence-floor` | `(d :i64) -> :f64` | Substrate's presence floor at d (`σ(d)/√d` with the presence sigma). User-composable into filter funcs. |
| `:wat::holon::coincident-floor` | `(d :i64) -> :f64` | Substrate's coincident floor at d (same shape, tighter sigma). |

## How get works

```
(get store pos probe filter)
  ;; 1. validate pos in [0, 100]; reject NaN/<0/>100 with RuntimeError
  ;; 2. compute cell spread:
  ;;      left  = floor(pos * num_cells / 100)
  ;;      right = ceil (pos * num_cells / 100)
  ;;    (clamped to [0, num_cells-1])
  ;; 3. flat-map over cells[left] and cells[right] (or just one if left == right)
  ;; 4. encode probe to a vector ONCE
  ;; 5. for each (stored_key, val) candidate:
  ;;      cos = cosine(encode(stored_key), probe_vec)
  ;;      track highest
  ;; 6. apply filter(best_cos):
  ;;      true  → return Some(best_val)
  ;;      false → return None
  ;; 7. if no candidates: return None
```

The filter is invoked AFTER the best is picked — answers "is the best good enough?", not "which candidate wins?" Filter is invoked exactly once per get (not once per candidate).

## Pos validation (strict, per Q4)

`pos: f64` must satisfy `0.0 ≤ pos ≤ 100.0` and not be NaN. Out-of-range or NaN → `RuntimeError::MalformedForm` with the bad value rendered. No silent clamping; callers play by the rules.

The substrate maps the user's `[0, 100]` to the cell index based on the encoder's d:
- d=10000 → 100 cells → `cell_idx = floor(pos)` (with clamping at 99 for pos==100)
- d=4096 → 64 cells → `cell_idx = floor(pos * 0.64)`
- d=1024 → 32 cells → `cell_idx = floor(pos * 0.32)`

## Files

| Path | What |
|------|------|
| `src/holon_hash.rs` | `HolonHash` struct + `pos_to_cell_index` / `pos_to_cell_spread` helpers. 12 Rust unit tests. |
| `src/lib.rs` | `pub mod holon_hash;` registration. |
| `src/runtime.rs` | `Value::HolonHash` variant; six dispatch entries; six eval functions. |
| `src/check.rs` | Six type schemes. |
| `wat-tests/holon/holon-hash.wat` | 11 wat-side integration tests. |

## Tests

**Rust unit tests (`src/holon_hash.rs`)** — 12 pass:
- `new_d_10000_yields_100_cells`, `new_d_4096_yields_64_cells`, `new_d_1024_yields_32_cells`
- `pos_to_index_at_d_10000`, `pos_to_index_at_d_4096`
- `pos_validation_rejects_negative`, `pos_validation_rejects_above_100`, `pos_validation_rejects_nan`
- `spread_at_cell_boundary`, `spread_clamps_at_boundary`
- `put_and_len_track_inserts_across_cells`, `put_idempotent_at_same_key`

**Wat-side tests (`wat-tests/holon/holon-hash.wat`)** — 11 pass:
- `test-new-empty` — fresh store has len 0
- `test-put-increments-len` — len grows
- `test-put-idempotent-on-same-key` — same key overwrites
- `test-get-self-hit` — putting key=val and getting with the same probe returns val
- `test-get-rejects-via-filter` — filter returning false → None
- `test-get-empty-returns-none` — no candidates → None
- `test-get-distant-probe-still-returns` — accept-any filter returns SOMETHING from candidate set
- `test-get-distant-cell-misses` — different pos → no candidates → None
- `test-len-across-cells` — len counts across multiple cells
- `test-presence-floor-positive`, `test-coincident-floor-positive` — floor accessors return positive values at d=10000

## Filter funcs in wat

Defined functions can't (yet) be referenced as values by their keyword path — the wat type checker treats `:my::foo` as a keyword literal, not a function reference. Use inline `:wat::core::lambda` to construct filters:

```scheme
;; A filter that uses the substrate's coincident floor at d=10000:
(:wat::core::let*
  (((floor :f64) (:wat::holon::coincident-floor 10000))
   ((tight :fn(f64)->bool)
    (:wat::core::lambda ((cos :f64) -> :bool)
      (:wat::core::< (:wat::core::- 1.0 cos) floor)))
   ((got :Option<wat::holon::HolonAST>)
    (:wat::holon::HolonHash/get store pos probe tight)))
  ...)
```

The substrate provides `presence-floor` and `coincident-floor` as raw f64 accessors; users compose them into filter funcs of whatever strictness they want.

## What slice 2 adds

`HolonCache` in `crates/wat-holon-cache/` (sibling crate adjacent to `wat-lru`). Same surface (`new` / `put` / `get` / `len`); composes:

- `wat::holon::HolonHash` (slice 1, this crate) for the bucketing + cosine readout
- `wat::lru::LocalCache` (existing wat-lru crate) for global LRU eviction
- A `per_cell_cap: usize` parameter for in-cell ceiling

Two eviction triggers: global LRU on retrieval rate, per-cell-cap when one cell fills before the LRU. The trader's lab cache (umbrella 059 slice 1) consumes `HolonCache` directly.

## Open

- **Filter func ergonomics.** Defined functions need an inline lambda wrapper to be passed to `get`. If wat ever grows function-as-value semantics for keyword-path references, this gets cleaner. Not blocking.
- **Multi-tier d.** Today's substrate is single-tier (arc 067, `DEFAULT_TIERS = [10000]`). If multi-tier surfaces, `HolonHash/new` may need to accept a tier list rather than a single d, and `pos_to_cell_index` may need to know which tier the form lands in. Future arc.
- **Filter func evaluation cost.** Filter is invoked once per get (not once per candidate). Cheap. If a future shape wants per-candidate filtering, a different surface ships then.

PERSEVERARE.
