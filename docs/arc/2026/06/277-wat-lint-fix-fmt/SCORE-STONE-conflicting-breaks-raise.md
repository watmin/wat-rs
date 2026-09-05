# SCORE — STONE: conflicting Breaks raise

No commit. Floor and clippy left to the orchestrator. `claims-set` untouched. No rule file left changed. No `BlankBefore`. R11 still all-or-nothing.

## The wall, shown firing — AND staying silent

Throwaway on `defn-multi.wat`: a second Break on the `->` node.

**Disagreement** (`"align"` vs R1's `"block"`):

```
fmt: conflicting Breaks for node 11 — block vs align
```

**Agreement** (`"block"` vs R1's `"block"`): no raise. Output identical to the un-sabotaged run. `IDEMPOTENT=true`.

Then both rules were deleted.

## Site

`breaks-map` only. `assoc` is now: missing → insert; same kind → insert (redundant, silent); different kind → raise, naming the node and both kinds. `claims-set` still bare `assoc`.

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| disagreement sabotage | **raises**, names node 11 and both kinds |
| agreement sabotage | silent, output unchanged |
| every existing fixture | ruled shape, **IDEMPOTENT=true** |
| `run.wat` on `wat/io.wat` | **COMMENTS=28**, IDEMPOTENT=true |
| `grep -c ClaimedUnder` / `grep -c 'col'` over rules | **0** / all **0** |
| `grep -c BlankBefore wat/fmt.wat` | **0** |
| `every_wat_scripts_file_loads` | **1 passed** |

---

## ORCHESTRATOR VERDICT — 2026-09-05

**ACCEPTED. No edit.** Third strike in a row that needed none.

| what | result |
|---|---|
| ★ **the wall FIRES on disagreement** (my own probe) | `fmt: conflicting Breaks for node 11 — block vs align` |
| ★ **agreement stays SILENT** (my own probe) | no raise, output byte-identical |
| blast radius | `wat/fmt.wat` only, **17 insertions**; `git diff` on `rules/` EMPTY |
| `claims-set` untouched | `git diff` mentions it 0 times |
| floor | **5179 run, 5179 passed, 0 FAILED, 18 skipped** |
| clippy `--all-targets -D warnings` | **0** |

Both rows together were the stone, and both hold.

### ⛔ AND I MIS-AIMED THE SABOTAGE THREE TIMES BEFORE IT LANDED

This is the finding worth keeping, and it is about me, not the strike.

```
attempt 1  targeted the arg-spec vector by INDEX 2      -> rule never matched
attempt 2  re-aimed at the `->` node by NAME            -> still nothing
attempt 3  ...because run-all.wat loads rule files EXPLICITLY.
           A file dropped in rules/ is never loaded, so the probe did not exist.
```

**Each attempt produced a clean, non-raising run** — and a non-raising run is exactly what a *working
silent wall* looks like. Had I stopped at any of the three, I would have reported either *"the wall
does not fire"* (false, and a defamatory finding against a correct strike) or *"agreement is silent"*
(true by accident, proving nothing).

★ **The cure was to validate the probe BEFORE testing the gate**: point it at a node nothing else
breaks and confirm the OUTPUT CHANGES. `:fix::add` moved to its own line — only then was the aim
real, and the very next run raised. `[[feedback_a_green_from_a_mis_aimed_probe_is_indistinguishable_from_a_working_gate]]`

⚠ **The rule-loading trap generalises and is worth naming:** `collect-rules :fmt` gathers rules by
NAMESPACE, but only from files the driver has `load-file!`d. Adding a rule file to `rules/` does
NOT add the rule. Every future sabotage — and every future style rule — must edit a driver too.

### Not disputed

STOP-2 through STOP-5 all held: `claims-set` has no wall, R11 is still all-or-nothing, no
`BlankBefore`, no existing fixture raises, no test went red. `wat/io.wat` still **COMMENTS=28**. The
previous two walls hold — `ClaimedUnder` 0, columns 0 across all six rule files.

**Three walls now stand at the same site**: an unknown `Break.kind`, a rule positioning a grandchild,
and two rules disagreeing about one node. Each was sabotage-proven before it was trusted.
