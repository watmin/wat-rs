# wat-rs arc 076 — Therm-routed Hologram + filtered-argmax — INSCRIPTION

**Status:** SHIPPED 2026-04-28 in commit 3f6cb8c (combined with
arc 077).

Routing moved from the caller into the Hologram itself: the slot
is derived from the form's structure (Thermometer inside →
bracket-pair lookup; non-therm → slot 0). Filter binds at
construction; reads do filtered-argmax against the matching cells.

---

## What shipped

- **Routing inside Hologram.** No more caller-supplied `pos`. The
  substrate inspects the key's structure to pick the slot:
  - Therm in the form → therm-coordinate-derived bracket pair.
  - Non-therm form → slot 0 (singleton bin).
- **Filter at make time.** `Hologram/make` takes the filter
  (`filter-coincident`, `filter-cosine-floor`, etc.); `get` /
  `find-best` apply it on read. Removes per-call filter parameters
  from the surface.
- **Filtered-argmax.** `find-best` returns `Option<(matched-key,
  val)>` — the cell inhabitant with maximum cosine to the probe,
  subject to the filter passing. On no candidate clearing the
  filter, returns `:None`.
- **`pos-to-idx` + `remove-at-index`** added to support the LRU
  sibling's eviction path (drop the matching cell entry without
  re-routing).

---

## Why this shipped combined with 077

Arc 077 killed the dim router (one program-d, capacity per call
site). Arc 076 moved routing inside Hologram. Together they form
one coherent narrative: the substrate decides routing internally,
and there's only one d to route over. Splitting the commits would
have left an awkward intermediate state where Hologram routes
internally but a multi-d router still exists upstream.

---

## What this arc unblocked

- HologramLRU's clean compositional shape (slice 2 of arc 074):
  pure-wat, no Rust shim, filter-once-at-construction.
- The lab's L1 cache: thinkers don't compute slot positions —
  they call `HologramCache/put` with the key, substrate routes.
- Arc 078's `HologramCacheService`: caches a single d, no per-key
  routing ceremony in the request enum.

## Subsequent rename

Arc 078 renamed `:wat::holon::HologramLRU` (the bounded sibling
that exercises this routing) to `:wat::holon::lru::HologramCache`.
The unbounded `:wat::holon::Hologram` retained its name.
