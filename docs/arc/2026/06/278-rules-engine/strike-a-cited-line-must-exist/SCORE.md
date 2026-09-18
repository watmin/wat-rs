# SCORE — the gate was right and pointed at the wrong third of the repo

> **Written after the orchestrator's own weighing.** The ★ was a false claim **inside the six
> citations the brief ordered cured** — and the prescribed cure would have propagated it.

## The result

`every_location_named_in_a_doc_comment_exists` now checks **both halves** of a `path:line` and scans
`wat/` + `wat-tests/` alongside `src/rete`. `.wat` (`;;`) comments read too; line counts memoised.
**Gate runtime 0.075 s** — no nextest budget needed.

| | citations | with `:LINE` |
|---|---|---|
| `src/rete` (the old root) | 174 | 30 |
| **newly examined** | **436** | **42** |
| total | **610** | **72** |

**All 40 defects were in the newly-examined 436. `src/rete` was clean on both halves** — the old gate
worked; it was aimed at the wrong third of the repo.

## ⛔⛔ THE ★ — A FALSE CLAIM INSIDE THE CITATIONS I ORDERED CURED

Three of the six attribute a quotation — *"the native kernel is the fast impl, the spec keeps it
honest."* — to `insert-all-spec`'s **own sibling comment** (`wat/seq.wat:262`, `wat/core.wat:1586`,
`core-foldl-spec.wat:8`).

**That comment does not exist.** It was true at `30725034f`, where `wat/rete.wat:1525` carried it, and
died in the kernel split.

**My DESIGN's "wrong three ways" table lists name, file, line — not the attribution.** So the mandated
cure, *"name the live symbol, drop the line"*, **would have carried a false attribution forward
verbatim into all three sites** while looking like a fix. The rider reworded the attribution and kept
the quote as a recorded formulation, claiming nothing about where it lives.

## ⛔ AND I MISSED A WHOLE OUT-OF-RANGE CITATION

`wat/query.wat:100` and `:116` both cite `wat/rete.wat:1971` — a **533-line** file. Present in
**none** of DESIGN, BRIEF, EXPECTATIONS, or the work-list row; verified absent by grep. Mandatory to
cure, since the gate reds on it. And `defrule` is not in `wat/rete.wat` at all any more — it is
`wat/rete/syntax.wat:202`.

**My BRIEF also cited `wat/core.wat:1585`; the citation is at `:1586`. Off by one, in a strike about
line citations.** (The work-list row says the same.)

## STOP-1 fired hard — 34 stale paths, fenced not fixed

Enumerated verbatim in a `DEFERRED` const as exact `(naming file, cited path)` pairs, so a **new**
stale path in those same files still reds and a row that stops matching is a **hard failure**. **The
list can only shrink.** Classification is a same-basename hint, not a verdict: ~10 look like a
directory reorg the citation never followed, ~17 genuinely deleted.

★ Among them: **`wat/rete.wat` names `kernel/tests.rs`** — the very citation this gate's own opening
paragraph is about, sitting in `wat/` the whole time where nothing was scanning.

**Deleting that const is what "absorb it" looks like, and it is the orchestrator's call.**

## ★ THE FLOOR WENT RED ONCE, FROM A COMMENT-ONLY EDIT

```
expected  :file "wat/core.wat"  :line 1919 … actual :line 1920
```

**Four golden EDNs hard-pin absolute `wat/core.wat` line numbers.** A comment cure that replaced 2
lines with 3 shifted everything below it. The rider made the edit line-count-neutral and both went
green.

**My blast radius — *"plus the six comment lines. No `src/` logic, no `.wat` code"* — was wrong.** A
comment-only edit to the stdlib is not free: the stdlib's line numbering is load-bearing test state.
**And it is this strike's own class one level down** — a line number as an unchecked claim, living in
a golden where the new gate cannot see it. The four are `probe_arc258_stone2b`, `probe_arc249_threading`,
and both `probe_arc279_format` goldens.

## Mutations — five, all on the shipped gate

| | mutation | result |
|---|---|---|
| 1 ★ | restore a `:1508` citation | RED — *"cites `wat/rete.wat:1508`, but wat/rete.wat is 533 lines"* — **both numbers** |
| 2 ★ | cite len+1 (534) | RED, same shape |
| 3 ★ | cite len exactly (533) | **PASS** — the boundary is `> len`, not off-by-one |
| 4a | roots → nothing | RED — *"only 0 path reference(s) found"* |
| 4b | blind the line extractor | RED — *"only 0 of 537 citation(s) carried a :LINE"* |
| 5 | bogus `DEFERRED` row | RED — *"1 DEFERRED row(s) no longer match anything"* |

**4b is the rider's own and it is the sharp one**: a path-count vacuity guard **cannot see a blind
line extractor**. Two independent vacuity guards, because the gate has two halves. My brief asked for
one.

## Also cured, not exempted

One false positive: `wat/service.wat:164` writes `spawn.wat/bracket.wat` as prose meaning *"and"*.
Rather than rune it, the rider drove that **no directory in the repo is named `*.rs`/`*.wat`** and
made a token whose parent component is a source-file name reject as prose. **A cure beats an
exemption.**

## Gates

- Floor **`5418 tests run: 5418 passed, 21 skipped`**, 0 FAIL rows. 5411 + 7 confirms the pre-value.
- `binary_id(wat::lint)` **265 passed** (258 + 7 extractor unit tests).
- clippy rc=0, zero warnings — **proven non-cached**: inserting a deliberate lint made clippy exit 101.
- `wat-scripts/fixes/rete-oracle-sigil.wat` md5 unchanged — the codemod that retired the name is
  untouched, as required.

## Remaining corrections to my artifacts

- **"Six citations" is 5 + 1 non-citation.** `core-foldl-spec.wat:7` is a bare retired *name* with no
  path and no line — which the DESIGN's own ⛔ rules legitimate. It is not in the gate's population
  and never could be. Row 3's "six" and row 1's "five times across four files" count different things.
- **The work-list row's *"grep finds nothing but the three citations themselves"* is stale** — five
  sites under `wat/` + `wat-tests/`.
- **Verified accurate**: the codemod is correct, `insert-all$oracle` is at `wat/rete/oracle/insert.wat:45`,
  `wc -l wat/rete.wat` = 533, and the citation **was** true when written (`30725034f`: 3660 lines,
  line 1508 = `(:wat::core::defn :wat::rete::insert-all-spec`). **The DESIGN's "true when written"
  framing holds.**
