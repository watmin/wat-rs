# Arc 042 — INSCRIPTION

**Closed:** 2026-04-24.
**Commits:**
- `95f9a2b` — DESIGN + BACKLOG opened
- `971c0c8` — Slice 1: service::Cache → lru::CacheService
- `<this commit>` — Slice 2: INSCRIPTION + cross-references

## What this arc fixed

`wat-rs/docs/ZERO-MUTEX.md` had only one drift surface to address:
three `:wat::std::service::Cache<K,V>` references that needed to
become `:wat::lru::CacheService<K,V>` per arcs 013 (LocalCache
externalization to wat-lru) + 036 (wat-lru namespace promotion
to `:wat::*`).

Everything else in the doc was current:

- Three-tier framing (immutable / thread-owned / program-owned).
- HandlePool discipline.
- spawn / send / recv / select primitives.
- Empirical claim about the trading lab running 30+ threads with
  zero Mutex.
- Arc 003 (TCO) and arc 004 (stream stdlib) references.
- `:wat::std::service::Console` mention (Console didn't move).

The concurrency architecture is invariant under the arcs that
shipped between 028 and 037 — those are about config, namespace
organization, and algebra surface, not threading.

## What this arc proved

**Drift is unevenly distributed across docs.** Arcs 038 (USER-GUIDE)
and 039 (README) had heavy drift because they cite shipped surface
constantly. Arc 040 (CONVENTIONS) had medium drift because it
codifies rules that arc-by-arc work touches. Arc 041 (wat-tests
README) had drift-by-omission. Arc 042 (ZERO-MUTEX) had drift in
exactly one path because the architectural story it tells is
substrate-shape-agnostic.

The audit shape — read the file, list every claim, walk each claim
against shipped state — is the same across all four. The output
varies because the input varies.

**Builder predicted "I doubt there's much to do" — correct.** The
prediction was load-bearing for arc-scoping: had we expected
heavy drift, we'd have over-budgeted slices. The light drift made
this the smallest implementation slice in the doc-audit set
(three Edits, ~11 line diff).

## Out of scope (lab-side work)

- `holon-lab-trading/CLAUDE.md` — the last item in the audit set.
  Different repo, different domain (lab architecture, not wat
  substrate). Will ship as a lab arc rather than a wat-rs arc.
  The cross-repo cwd discipline (memory entry
  `feedback_cross_repo_cwd.md`) applies — use `git -C` for git
  ops; cd back immediately if shell context shifts.

## Files touched

- `docs/ZERO-MUTEX.md` — three Edits.
- `docs/arc/2026/04/042-zero-mutex-drift-sync/{DESIGN,BACKLOG,INSCRIPTION}.md`
  — the arc record.
- `docs/README.md` — arc index extended.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — cross-repo audit trail row.

## The doc-audit set, finished

Five arcs in the audit set total:
- 038: USER-GUIDE.md (recovery + sync)
- 039: README.md (drift)
- 040: CONVENTIONS.md (drift)
- 041: wat-tests/README.md (drift by omission)
- 042: ZERO-MUTEX.md (drift)

Plus one preserved-as-historical:
- 005 INVENTORY.md (per builder: "keep it as is for the record" —
  USER-GUIDE Appendix is the live forms reference now)

All wat-rs user-facing docs are current through arc 037. The lab's
`CLAUDE.md` is the remaining out-of-scope item (lab-side arc).
