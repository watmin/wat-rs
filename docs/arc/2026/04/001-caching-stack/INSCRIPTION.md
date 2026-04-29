# Arc 001 — Caching stack — INSCRIPTION

**Status:** DISCARDED 2026-04-29.

Pre-arc-convention session notes from 2026-04-19. The doc style had
not yet solidified (no Predecessors / Surfaced-by / Slice-plan
structure; just running design notes + a deadlock postmortem).

The actual cache lineage shipped through a sequence of subsequent
arcs:

- [Arc 013 — `wat-lru` externalization](../013-external-wat-crates/INSCRIPTION.md)
  — `:wat::lru::LocalCache<K,V>` + `:wat::lru::CacheService<K,V>`
  outside wat-rs as a sibling crate.
- [Arc 074 — `HolonStore<V>`](../074-holon-store/INSCRIPTION.md)
  — coordinate-cell cache primitive (the operational shape arc
  073 was reaching for).
- [Arc 076 — therm-routed Hologram](../076-therm-routed-hologram/INSCRIPTION.md)
  — routing inside the Hologram type.
- [Arc 077 — kill the router](../077-kill-the-router/INSCRIPTION.md)
  — one program-d via `:wat::config::set-dim-count!`; retired the
  `DEFAULT_TIERS` machinery this arc had assumed.
- [Arc 078 — service contract](../078-service-contract/INSCRIPTION.md)
  — Reporter + MetricsCadence + null-helpers + typed Report enum;
  the canonical service shape every cache service follows.

The substantive design discoveries from this arc's session — the
ThreadOwnedCell ownership story (clarified through the deadlock
postmortem), the L1-vs-L2 split, the thread-owned-cache discipline
— are folded into those subsequent arcs' INSCRIPTIONs.

[`DESIGN.md`](./DESIGN.md) and [`DEADLOCK-POSTMORTEM.md`](./DEADLOCK-POSTMORTEM.md)
stay in-tree as historical context only.

---

This arc is closed by supersession, not by ship.
