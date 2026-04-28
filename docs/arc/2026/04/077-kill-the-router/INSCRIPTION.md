# wat-rs arc 077 — Kill the dim router — INSCRIPTION

**Status:** SHIPPED 2026-04-28 in commit 3f6cb8c (combined with
arc 076).

One program-d. Routing happens at the call site (capacity in `make`
calls), not via a multi-d dispatch table. Removed the dim-router
abstraction; replaced with `:wat::config::set-dim-count!` for
program-wide d.

---

## What shipped

- **`(:wat::config::set-dim-count! d)`** — sets the program's
  encoding d. One d per program; no router to consult.
- **Capacity at the call site.** `Hologram/make` reads the
  ambient dim-count and computes `capacity = floor(sqrt(d))`
  internally. Callers no longer pass `dims` or per-tier capacity
  config.
- **`bundle_capacity` and friends adapted.** Reporting now reflects
  budget=`floor(sqrt(d))` directly, not "tier 0 budget" /
  "tier 1 budget" / etc.
- **Router torn down.** `Router` struct, multi-tier dispatch
  table, `set-dim-router!`, `DEFAULT_TIERS = [10000]` — all
  removed. The flat-default lineage of arcs 067 settled here.

---

## Why a single d

Three reasons:
1. The router was paid for in code without ever paying back. Every
   consumer of `coincident?` / `cosine` / `Hologram` had to thread
   the d-routing concern through their type signatures and call
   sites; no caller actually USED multi-d in production.
2. The lab's trader runs at one d (10000); the multi-d substrate
   was speculative.
3. Routing belongs at the *consumer's* call site, not in an
   ambient dispatch table. If a future workload genuinely needs
   multi-d, that consumer can build its own router and pay for it
   locally.

---

## What this arc unblocked

- Arc 074 slice 2 (HologramLRU) — slate-clean callers; one d,
  capacity = floor(sqrt(d)).
- Arc 076 — Hologram routing internal to the type, no caller pos.
- Arc 078 — substrate cache services don't carry dim parameters
  in their service contract; the d is ambient.

## Successor (potential)

If a workload ever needs cross-d cosine readout (e.g., a system
with d=4096 hot-path and d=8192 archival), a new arc can ship a
*consumer-built* router. The substrate stays simple.
