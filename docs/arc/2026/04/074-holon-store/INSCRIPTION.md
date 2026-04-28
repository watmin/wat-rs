# wat-rs arc 074 — `Hologram` + `HologramLRU` — INSCRIPTION

**Status:** SHIPPED 2026-04-28 across two slices.

The cache lineage's foundation: coordinate-cell store with cosine
readout, plus a bounded sibling that composes wat-lru's
`LocalCache` for LRU eviction. Subsequent arcs (076, 077, 078)
refined the surface; this arc is where the abstraction landed.

---

## What shipped

| Slice | Surface | Where | Commit |
|-------|---------|-------|--------|
| 1 | `:wat::holon::Hologram` — unbounded coordinate-cell store. `make`, `put`, `get`, `find`, `find-best`, `remove`, `remove-at-index`, `pos-to-idx`, `len`, `capacity`. Cosine readout against the inhabitants of each cell; sqrt(d) cells per d. | `src/hologram.rs` + `wat/holon/hologram.wat` (later moved to `wat/holon/Hologram.wat`) | (slice 1, arc 074) |
| 2 | `:wat::holon::HologramLRU` — bounded sibling. Pure-wat composition: `Hologram/put` + `wat::lru::LocalCache::put` (eviction-aware); `Hologram/find` + filter on read. Per-cell-cap via wat-lru's eviction-aware put returning `Option<(K,V)>`. | `crates/wat-hologram-lru/wat/holon/HologramLRU.wat` (later renamed to `wat-holon-lru/wat/holon/lru/HologramCache.wat` per arc 078) | b91a3a9 |

The wat-lru `LocalCache::put` change to return `Option<(K,V)>` for
the evicted entry shipped in 2275b6f as slice-2 prep — needed so
the bounded sibling could drop the matching Hologram cell entry.

---

## Architecture as shipped

**Hologram is the population; HologramLRU is the bounded
population.** Both live under `:wat::holon::*` because both are
HolonAST-keyed coordinate stores. The bounded sibling shipped in a
sibling crate (`crates/wat-hologram-lru/`, later `wat-holon-lru/`)
because it depends on `wat-lru` — keeping the core `wat` crate free
of that dep.

### Pure-wat composition

`HologramLRU/put` is wat code: route, `Hologram/put`,
`LocalCache::put` returning `Option<(K,V)>`, on Some — call
`Hologram/remove`. No Rust shim beyond the registrar. Slice 2's
own `register()` is a no-op; the contract uses `wat_sources()` only.

### Reused everything

The substrate primitives needed by HologramLRU (`Hologram/find-best`,
`Hologram/remove-at-index`, `Hologram/pos-to-idx`) all already lived
in core. No new substrate for slice 2.

---

## What this arc unblocked

- The lab's L1/L2 cache (umbrella 059) — the trader needed a
  HolonAST-keyed coordinate store with eviction; this arc shipped
  it.
- Arcs 076 (therm-routed routing inside Hologram) and 077 (kill the
  dim router) — both built on this abstraction.
- Arc 078 (service contract) — wrapped HologramLRU in
  `HologramCacheService`, lifted to substrate.

## Subsequent renames

- Arc 078 slice 0: crate dir `wat-hologram-lru/` → `wat-holon-lru/`
  to match the namespace path.
- Arc 078 slice 1: `:wat::holon::HologramLRU` →
  `:wat::holon::lru::HologramCache`. The "LRU" qualifier moved from
  the type name to the namespace; the type name describes what the
  thing IS.

The DESIGN.md still uses the original names; the source of truth
for current naming is CONVENTIONS.md and the wat source files.
