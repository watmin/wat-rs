# EXPECTATIONS — a poisoned item costs one item

Scored against `DESIGN.md`. Write `SCORE.md` beside it.

## The null

⛔ **"The per-item fate cannot be determined for arm X"** is a full delivery — name X, say what the
coordinator does and does not know there, and land the verb covering the fates it *can* report.
A per-item slot filled with a fate the code cannot distinguish is worse than no slot.

## Rows

| # | what | how it is judged |
|---|---|---|
| 1 | **The sibling verb** | `map`/`each` signatures byte-identical; the 60 existing sites untouched (re-count them yourself). ⛔ **Not `try-`** — that prefix means non-blocking here. Name it and justify the name. |
| 2 | ⭐ **The outcome shape** | No precedent exists in this corpus — measured zero. Justify the shape chosen. ⛔ **Every fate it names must be one the coordinator can actually distinguish**; do not reintroduce a distinction `collect-gave-up!` had to drop as false. |
| 3 | ⭐ **`FrameTooLarge` becomes REPORT-FINAL in fact** | Record it, drop the item, **keep the runner**, continue. State what changes for the other arms and what does not — `Lost`/`Closed` re-dispatch stays right. |
| 4 | ⭐ **Control by MUTATION** | One poisoned item among good ones: the good results come back, the bad one is named, the fleet survives. Remove the per-item recording → the pool is lost again (`REPORT-GONE`). **Show both runs.** |
| 5 | **Non-vacuity** | The poisoned item really failed, and the other runners are **alive at the end** — that is the claim, and it is the one a green could fake. |
| 6 | **Scope wall** | `map`/`each`, thread tier, suppression, lineage-Admin, queue knobs, `RecvOutcome`, the 7 blocking sends — untouched. Say what you did not do. |
| 7 | **Floor** | `scripts/floor.sh`, release. Summary verbatim + `.floor/<stamp>/` + the tree it ran against. ⛔ **And sweep any bound you rely on** — the last stone's deadlock hid below a 500 ms cliff and a green floor missed it. A red: do not re-run, capture whole, name the arm. Clippy over the WHOLE output. |

## What would make this stone wrong

- **A fate the code cannot distinguish**, filled in to make the type look complete.
- **Widening `map`** after all. It is ruled out; the sibling is the ruling.
- **`try-` in the name.**
- **A control that only tests all-failure or all-success.** The claim is *partial* — mixed input is
  the only case that proves it.
- ⚠ **Claiming the pool survives without checking it.** Row 5.
- **A single-bound test.** The previous stone's deadlock was invisible at 200 ms and fatal at 1000 ms.

## Deliverable

`SCORE.md`: the verb and its name's justification, the outcome shape and why, the arm-by-arm change
table, mutation evidence both ways, proof the fleet survived, floor Summary + `.floor/` path + tree
statement + the bound sweep.

⚠ Paths in pulsare messages are repo-root-relative (`wat-rs/docs/…`).
