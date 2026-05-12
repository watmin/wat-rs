# Arc 186 — Renumber 2026/05 arcs to start at 001

**Status:** stub opened 2026-05-13 per user direction.
**Tracking:** arc 109 v1 milestone blocker (low priority; queued cleanup).

## Motivation

May 2026 was opened with arc numbers continuing from April (which ended at 119). The convention should have been `2026/05/NNN-slug` starting at `001`. Current May arcs are numbered 120-186+.

This arc executes the renumber when all in-flight May arcs have closed.

## Canonical source

See [`docs/arc/2026/05/RENUMBER.md`](../RENUMBER.md) for:

- The mistake's framing
- Why the rename can't execute mid-flight (cross-references in committed SCORE prose)
- Execution plan (build current rename table from `ls`; `git mv` each; grep-sweep cross-references)

The RENUMBER.md table is intentionally stale-friendly — it's rebuilt from disk at execution time. This stub is the arc-shaped handle for the work; RENUMBER.md is the working plan.

## Prerequisites

Cannot execute until all in-flight May arcs have closed (no open BRIEF / EXPECTATIONS / SCORE without an accompanying INSCRIPTION). At time of stub:

- arc 170 — IN FLIGHT (Phase 2a gap slices running)
- arc 119 — in progress
- arc 130 — in progress
- arc 163 — in progress
- arc 174-185 — stubs (no in-flight work)

Plus: this arc itself will be renumbered by its own execution. Name (`may-renumber`) survives the renumber.

## Sketch

Per RENUMBER.md execution plan:
1. Confirm no arc dir under `docs/arc/2026/05/` has uncommitted work (git status clean for arcs)
2. Rebuild the rename map from current `ls`
3. `git mv` each directory (atomic single commit)
4. Sweep cross-references in committed prose: `grep -rn "arc 1[2-8][0-9]" docs/ src/ crates/ wat/ wat-tests/ tests/ examples/ .claude/` — and (importantly) sibling holon-lab-trading repo where wat docs may be referenced
5. Workspace verifies — ~30-60 min of mechanical work

## Cross-references

- `docs/arc/2026/05/RENUMBER.md` — canonical working plan
- arc 109 v1 milestone closure — this arc gates that closure
- Task #229 (arc 109 v1 milestone closure) — pending
