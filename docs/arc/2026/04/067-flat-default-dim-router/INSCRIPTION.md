# Arc 067 — Flat default dim router — INSCRIPTION

**Status:** DISCARDED 2026-04-29.

The arc proposed flattening `DEFAULT_TIERS` from
`[256, 4096, 10000, 100000]` to `[10000]` — a halfway step that
would have eliminated the multi-tier router's complexity without
fully retiring the routing concept.

[Arc 077](../077-kill-the-router/INSCRIPTION.md) shipped the
aggressive answer instead: kill the dim router entirely. One
program-`d` lives at `EncodingCtx.dim_count` set via
`:wat::config::set-dim-count!`. No tiers, no routing, no
auto-promotion path. The substrate is one dim per program; the
consumer picks it once at config time.

Halfway-steps are sometimes the right shape; this one wasn't.
The aggressive path that shipped settled the substrate at a
cleaner equilibrium than this arc's incremental simplification
would have reached.

[`DESIGN.md`](./DESIGN.md) stays in-tree as the historical record
of the path explored and rejected.

---

This arc is closed by supersession, not by ship.
