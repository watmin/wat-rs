# Arc 187 — Post-arc-109 modularization audit per MODULARIZATION-NOTES.md

**Status:** stub opened 2026-05-13 per user direction.
**Gates on:** arc 109 v1 milestone closure.

## Motivation

> *"after 109 closes -> check the MODULARIZATION-NOTES.md"*

The substrate's `src/runtime.rs` (23,801 lines) + `src/check.rs` (15,108 lines) = 39k lines together, 63% of `src/`. Both grew organically; natural boundaries visible inside but the files are hard to navigate.

`docs/MODULARIZATION-NOTES.md` (queued 2026-05-08) holds the rationale + approach. This arc is the executing handle.

## Canonical source

See [`docs/MODULARIZATION-NOTES.md`](../../../MODULARIZATION-NOTES.md) for:

- File-size audit + candidate-boundary analysis
- Why-not-yet (arc 109 foundation must be impeccable first)
- Approach: **incremental extraction, not big-bang** (avoid landing in a half-extracted state)
- Each extraction = one numbered arc with full DESIGN/BRIEF/EXPECTATIONS discipline

## Sketch (placeholder)

Per MODULARIZATION-NOTES execution discipline:

1. Confirm arc 109 v1 milestone closure has shipped
2. Re-audit file sizes (MODULARIZATION-NOTES counts are 2026-05-08; refresh)
3. Identify natural extraction boundaries (substrate concerns that earn their own modules)
4. Sequence extractions as a series of arcs — each surgical (one move per commit; blame preserves through `--follow`); avoid bundled "rename + edit" commits
5. Workspace stays green throughout

User direction (cross-ref): *"once 109 wraps up - we'll have what we believe to be an incredibly solid foundation to begin the next leg of work... i cannot begin any of that work until the foundation is impeccable."* Modularization is post-foundation work.

## Cross-references

- `docs/MODULARIZATION-NOTES.md` — canonical working plan
- Arc 188 (perf + Rust impl scrutiny) — builds on this arc's clean module boundaries
- Arc 109 v1 milestone closure (task #229) — gates this arc
