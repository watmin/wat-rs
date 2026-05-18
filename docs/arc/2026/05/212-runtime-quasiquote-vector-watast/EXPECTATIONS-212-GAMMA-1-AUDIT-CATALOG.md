# Arc 212 stone γ-1 — EXPECTATIONS

## Independent prediction

- **Runtime band:** 20-40 min Mode A. Grep enumerates sites; per-site classification is fast (read enclosing fn; pick one of four classifications).
- **Sites total inspected:** 80-120 sites across src/ + crates/*/src/
- **Walker (already migrated):** ~12 (the prior spawn's work, atomically committed)
- **Walker (pending migration):** 1-3 (`walk_for_bare_primitives` known; possibly more the prior audit missed)
- **Walker (sharpening target):** 2 (`validate_comm_positions`, `collect_process_calls`)
- **Leaf-decomposition:** ~60-100 (parsers, classifiers, single-shape handlers — bulk of sites)
- **Surprises expected:** 0-2 (a previously-unflagged walker; an ambiguous classification site)

## Predicted catalog breakdown

| Class | Predicted count |
|---|---|
| Walker (already migrated) | 12 |
| Walker (pending migration) | 1-3 |
| Walker (sharpening target) | 2 |
| Leaf-decomposition | 60-100 |

If counts diverge significantly, that's calibration data worth noting in the SCORE.

## Honest-delta watch

1. **A walker neither in the "12 migrated" nor "known pending" list shows up as pending** — that's a previously-unaudited walker. Catalog it. The orchestrator queues a δ-N stone for it.

2. **A site is ambiguous between Walker and Leaf-decomposition** — e.g., a function that pattern-matches on List for ONE purpose but ALSO has internal recursion via a helper. Classify with reasoning; orchestrator decides if a δ-N or no action.

3. **A site is found in a file the BRIEF didn't pre-name** (e.g., a Rust file in a crate outside the named list). Catalog it; orchestrator decides scope.

4. **One of the "already-correct" walkers (per BRIEF list) turns out to NOT use children() but to have explicit Vector + List arms in a match.** That's still correct (it handles Vector explicitly). Classify as "Walker (already migrated)" — explicit Vector arm IS the children() shape's manual equivalent. Note in the table's reason column.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | All sites classified into one of four classes | YES |
| 2 | The 12 known-migrated walkers appear as "Walker (already migrated)" | YES |
| 3 | The 2 known sharpening targets appear as "Walker (sharpening target)" | YES |
| 4 | `walk_for_bare_primitives` appears as "Walker (pending migration)" | YES |
| 5 | SCORE file written at the named path | YES |
| 6 | Zero code edits, zero git operations, zero cargo invocations | YES |
| 7 | Any STOP trigger hit is reported honestly in SCORE | YES |

## Mode classification

- **Mode A:** catalog complete; all sites classified; SCORE captures the table; no STOP trigger fired.
- **Mode B:** catalog partial; you hit a STOP trigger; SCORE captures what's classified + names the trigger. Honest stop = Mode A's sibling, not a failure.
- **Mode C:** you broke a STOP rule (started editing, started investigating an out-of-scope failure, ran tests for failure investigation). The work is invalid.

## Calibration metadata

- **Orchestrator confidence:** HIGH. The pattern (grep + classify) is bounded. The four classifications are explicit. The 12 known-migrated + 2 known-sharpening + 1 known-pending give a strong scaffold.
- **Risk factors:** ambiguous classifications (some walkers have subtle recursion patterns); fragmentation across many files (sonnet may lose place); urge to migrate when finding pending walkers (the STOP triggers explicitly catch this).
- **Why this matters:** the catalog is the foundation for every subsequent δ/ζ/η stone. It confirms coverage. It tells the orchestrator what stones to queue next. Without it, future stones operate on assumed completeness — that assumption is what dirty-tree work proved fragile.

## Tooling-proven-by-use note

γ-1 is a stepping-stone proof of the stone discipline itself: ONE concern, ONE deliverable, explicit STOP triggers, no workspace-failure-count framing. If γ-1 lands Mode A, the discipline is validated for δ-N + ζ + η stones to follow the same shape.

## Cross-references

- Arc 212 DESIGN § "Scope EXPANDED 2026-05-18 (post-L4-conversation)" — the L4 endgame trajectory
- Arc 212 DESIGN § "Locked stone chain (L0 → L4 trajectory)" — where γ-1 fits
- INTERSTITIAL § 2026-05-18 (post-compaction, mid-arc-212) — the session that produced this BRIEF
- BRIEF-212-GAMMA-1-AUDIT-CATALOG.md — the brief itself
- Inscribed reasoning at `src/check.rs:2137` (`validate_comm_positions`) + `src/check.rs:~3596` (`collect_process_calls`) — the sharpening targets pre-classified
