## MORA — Cast Report (TARGET 3 — the grid harness) — **CLEAN**

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.
> ⭐ The only ward in this vigilia whose trigger fires on target 3 and nowhere else.

## SCOPE

Re-derived, from `wat-scripts/perf/grid`:

```
grep -nE '\bsleep\b' *.sh                 → empty (exit 1)
grep -nE 'poll|epoll|try_recv|-t 0' *.sh  → empty (exit 1)
grep -nE '\btimeout\b' *.sh               → run-axis.sh:157, run-axis.sh:163
```

**Delta against the handed-down measurement: none.** Zero `sleep`, zero readiness-snapshot surface, exactly the same two `timeout` lines. No `rune:mora` anywhere in the tree.

## Judgment

**Reading 1 — out of domain — holds**, argued through the four questions against `run-axis.sh` (read in full, 1–384):

- **Simple?** A coordination wait is expressible as one event-source select because a real event exists to wait for. Here there is none: `timeout`/`ulimit -v` at `:154-165` are the "BLAST DOOR" (`:124-137`) against a benchmark that **"CRASHED THE BUILDER'S MACHINE"** on 2026-07-30 by consuming 43 GiB. A hang or runaway allocation has no fd-event that fires at "this would have finished" — the thing being bounded is the *absence* of a terminating event, which a select cannot wait for.
- **Honest?** The wait names exactly what it is for: a ceiling, not an estimate of an event's arrival. `:246-251` confirms it — `WAT_RC` is inspected and the diagnostic separates `124` (*"TIMED OUT… Raise GRID_TIMEOUT, or the size is too big"*) from `137` (*"KILLED (SIGKILL)… the memory cap. That is the guard working"*). A script that knows and states which mechanism fired, not one pretending a deadline is a synchronisation point.
- **Good UX?** Does not need to scale with kernel speed, because that is not what it measures — an honest run finishes long before the ceiling on any kernel; the ceiling only matters for the pathological case the discipline does not model.
- **Obvious?** Yes — failure names its own arm.

`GRID_TIMEOUT` defaults to `300` (`:139`); `GUARD_KB` derives from `GRID_MEM_MAX` (default `6G`, `:138`) via `numfmt --from=iec` when `systemd-run` is unavailable. An operator who sets nothing gets a 300s wall-clock kill-guard plus a ~6 GiB address-space cap, both env-overridable.

## Reading 3, investigated as directed

- `run-axis.sh:236-256`: `WAT_RC=$?` is captured immediately after the guarded call, **outside `set -e`** (`set +e`/`set -e` bracket it at `:230,240`), so it is never lost to the `pipefail` trap. If no `#grid/Result` line was extracted, the script prints the RC-specific diagnosis and **hard-exits 1** (`:255`) — it does not emit a `#grid/Verdict` for that size, and does not drop into the mean/spread computation with a partial sample.
- `run-all.sh:139-144`: the caller does not swallow it — `if ! bash run-axis.sh …; then echo "run-all: axis '$axis' FAILED…" >&2; rc=1; fi`. A killed axis is announced and the sweep's exit code goes nonzero. This matches the doctrine at `check-grid-three-way.sh:39` (*"unreadable is a HARD FAILURE — never a skip"*) in spirit, in a different script.
- **The one gap I can point to in the code but cannot demonstrate as live:** `WAT_RC` is only consulted **inside** the `-z "$WAT_LINE"` branch (`:242-256`). If a process were killed by `timeout` *after* flushing a complete `#grid/Result` line but before exiting on its own, `WAT_LINE` would be non-empty, RC=124 would never be inspected, and that run's wall-clock (`:239,321`) would silently include time spent hung, feeding `:wat-wall-ms`/`:wall-ratio`/`:fire-share-pct`. I have no evidence this binary ever hangs post-print (it is a small parse→fire→print→exit program per `:6-7`), and a single-line stdout write is under `PIPE_BUF`, so it is not a partial-write race either. **I am not raising this as a mora finding — it is not a wait or a readiness snapshot, it is an RC-scoping completeness question — but it is the honest boundary of what I could confirm versus what the code merely makes plausible.**

## Findings

None. Both `timeout` sites are resource-bound kill-guards on a runaway benchmark subprocess (paired with a memory cap in the same breath), not a coordination wait between two parties — mora's *"what am I waiting FOR?"* discipline has no wire-event alternative to point to here, so it does not reach this surface. **No rune needed**, because no rune category (`calibration`/`external-api`/`no-kernel`/`no-reactor`) fits a runaway-process safety net, and forcing one would misuse the taxonomy rather than honour it. The timeout path was checked end-to-end (`run-axis.sh` → `run-all.sh`) and found to **fail loud, not quiet**.

**CLEAN**
