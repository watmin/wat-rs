# SCORE — the fence is empty, and the golden set I told the rider to trust was short

> **Written after the orchestrator's own weighing.** The ★ was that the artifact naming the
> golden-pinned files under-reported them — and the missing entry was **in this strike's own edit set**.

## The result

`DEFERRED` **34 → 0**, const and its `deferral_seen`/`unused` machinery deleted (verified: zero live
consts; the two remaining `DEFERRED` greps are prose recording that the fence existed).
**15 re-pointed, 19 paths deleted.** 26 files, +60/−121.

## ⛔⛔ THE ★ — THE GOLDEN-PINNED SET WAS UNDER-REPORTED, INCLUDING A FILE THIS STRIKE EDITS

`strike-a-cited-line-must-exist/SCORE.md` says *"Four golden EDNs hard-pin absolute `wat/core.wat`
line numbers."* Verified by the orchestrator:

- **`wat/core.wat` — FIVE goldens.** The fifth,
  `tests/wat_lang/wat_core_cond__cond_refuses_missing_else.edn`, pins **the same line** as the arc258
  golden. An edit above it would have reddened a test nobody was watching.
- **`wat/service.wat` — FOUR goldens** (`probe_arc278_peers_bijection__case{1,2,4,5}`), lines 864/881.
  **This strike edits `wat/service.wat`.** Named in no artifact.
- **`wat/Record.wat` — one**, line 145.

My DESIGN framed *"the strike must determine which other files are"* as an open question. **The answer
included the file it was about to edit.** The instruction to determine rather than assume was right
and load-bearing; the number it sat beside was not.

**24 of 25 `.wat` edits are exactly line-count-neutral.** The exception is `wat/rete.wat` (533 → 534)
— not golden-pinned, and growth only *relaxes* the `cited > len` check.

## ★★ THE TRAP ROW: BOTH EASY ANSWERS REFUSED

I predicted `kernel/tests.rs` → `src/macros/tests.rs` was a false target, and instructed that an
unconfirmable row be treated as GONE.

**It proved the false target properly** — `src/macros/tests.rs` is 1,583 lines (cannot hold the cited
`:3068`) and contains **zero** occurrences of *"overlay"*, the word the sentence vouches for.

**Then it declined my fallback.** `git show f98226353^:src/rete/kernel/tests.rs` at line 3068 holds
the arm-lease assertions *"last lease drop removes the intern"* / *"next fire after release must
rebuild"* — both live verbatim today in `src/rete/kernel/tests/arm_lease.rs:120,126`, a file with 40
uses of *"overlay"*. **The citation was recoverable by content.**

*"When in doubt, delete"* applied mechanically would have discarded information the history still
held. That is a real correction to my instruction.

## ★ A SHAPE MY BRIEF DID NOT NAME — three false targets, not one

Of the 12 basename matches, **three** are false, and two are a kind I never described: **the basename
resolves to the citing file itself**, because the sentence is a *provenance clause about a rename*.

- `wat/gen.wat` cites `wat-scripts/lib/gen.wat` — the correct successor **is** `wat/gen.wat`, so
  re-pointing makes the sentence read *"PROMOTED from wat/gen.wat"* inside `wat/gen.wat`.
- `wat/kernel/readln.wat` cites `wat/kernel/services/stdin.wat`, renamed **to the citing file itself**.

**There a re-point is verifiably correct as a file mapping and still wrong as a cure.** Both had the
path deleted and the name kept as provenance. (`wat/repl.wat` and `multiline-roundtrip.wat` are the
same *shape* but re-pointable, because line 1 of each is a file-identity header, not a provenance
clause — so self-reference is correct there.)

## Re-points earned by content or a git rename, never a name match

Fifteen, each with evidence: `R100` renames for `probe-rule-lits`, `repl.wat`, `seq-fold-aliases →
core-reduce`, `probe_arc279_format`, `probe_arc272_6b`, `multiline-roundtrip`; probe-body matches
line-for-line for the two arc-170 probes; **`#[test]` counts matching the citing prose** — exactly 5
threading mints, exactly 6 and exactly 18 — for the three `tests/` reorg rows; and the
`validate.rs → validate/mod.rs` split traced through *"─── The `:not` bind wall ───"* moving
`:1404 → :469`.

## STOP-4 — no prose left lying, and four sentences made honest

Four citations pointed at files deleted **in the same commit that added the citing file** without
saying so. Rather than reword silently, the rider added the fact — `(deleted with this commit)` /
`(since deleted)` — matching phrasing a **fourth sibling already used**. Flagged as a wording change
beyond path removal, which is the right disclosure.

## STOP-3 — two citations the gate structurally cannot see

Both are bare `basename:LINE` with **no slash**, dropped by `citations_in_comments`'
`path.contains('/')` filter:

1. **`wat/query.wat:176` cites `rete.wat:2150`** — that file is **534** lines. Past EOF, and the
   gate's own `out_of_range` arm would fire if the token were slashed.
2. **`wat/rete.wat:165` cites `arm.rs:572`** for *"a fact overlay over circuits it does not own"*.
   Line 572 is `pub(crate) struct NetworkEdges`; the sentence's source is `arm.rs:661`. **This is
   in-range drift — which the gate says outright it cannot detect.** An anchored sweep of the 18 bare
   `name.ext:N` citations found only the one past-EOF case, precisely because the others are "valid"
   and wrong.

**Reported, not absorbed.** Correct call — it is a new class, not this strike's population.

## Mutations — three, all RED

| | mutation | result |
|---|---|---|
| 2 ★ | all 34 cured, `DEFERRED` left in place | RED — *"34 DEFERRED row(s) no longer match anything"*. Also proves the stale assert (which runs first) passed: every citation cured, none newly broken |
| 1 ★ | const deleted, one citation reverted | RED — *"names `tests/probe_arc279_format.rs`, which does not exist"* |
| 3 | const deleted, all 25 `.wat` restored | RED — *"34 path(s) that do not exist"*. Removal is earned |

Restores by copy + `sha256sum -c` (26/26), never `git checkout`. Clippy proved non-cached: injected
`(0..3).map(|i| i)` → rc=101 → restored → rc=0.

## Honest deltas

- **My EXPECTATIONS row 8 does not fit this strike.** *"The floor must be ≥ 5,418 plus every arm you
  drive"* — the mutation arms are transient, and the only permanent change **removes** an assert
  branch. The count is necessarily 5418. **A test count is the wrong instrument for a strike whose
  deliverable is a deleted allowlist**, and read literally the row is unsatisfiable.
- **"12 rows / 11 distinct targets" was right and under-sold the trap** — three false targets, not the
  one I predicted.
- The header stamp reads `2026-09-05`: **UTC**, which is what `.floor/` uses. Local is 09-04. Not an
  error.

## Gates

Floor **`5418 tests run: 5418 passed, 21 skipped`**, 0 FAIL rows · `wat::lint` **265** · clippy rc=0,
zero warnings, proven non-cached.
