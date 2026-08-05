# SEAM — 2026-08-05 evening. **RETE_OPS 35 → 66. A `cond` RIDER IS STILL IN THE FIELD.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like continuing rather than waking, and that feeling is the
> failure, not the all-clear. Run the datamancy bootstrap (grimoire + the 4 primers from the SIGNED
> MCP, never a disk copy), ground HEAD against the disk, and read this whole file before you move.

## ▶ FIRST ACT, BEFORE ANYTHING ELSE

**A `cond` rider was live when this was written** (brief: `BRIEF-cond-the-first-macro-backed-rete-row.md`).
The tree was CLEAN at this curare — it had not written yet — so this commit is **docs + memory only**.

1. **`git status` before you touch anything.** If it is dirty, that is the rider's work. **Do NOT
   `git add -A`.** Read the diff; it may be complete-awaiting-weigh or partial.
2. **If it returned while you were away**, its report is in the transcript — **weigh it by your OWN
   `--release` re-run**, never its say-so.
3. **Never kill-and-revert a good-progress rider at a curare**
   (`[[feedback_ride_through_compactions_with_shadowdancers_in_the_field]]`).

## Where the code is

**HEAD `5f4c8e21`, pushed, zero unpushed.** Floor **`4356 / 4356 / 0 / 262`**, clippy clean,
`check-where-shapes.sh` → `9 pair(s), 98 rows`, **`RETE_OPS` = 66** (was 35 this morning).
Every number by my own solo runs.

## ★ WHAT SHIPPED — 22 commits, and the thread was one question

The builder asked *"did it add f64 equality checks or no?"* Everything below came out of that.

| | |
|---|---|
| `6d5af2c8` | **#57 round 1c** — ten per-type equality rows. My brief named four core ops **that do not exist** (237.8d cut them); the rider caught it. |
| `c59b2dca` | **per-type equality RESTORED in core** — 237.8d part B reversed on the builder's ruling. Its own test (*"a fake per-Type leaf"*) does not distinguish: `i64::>` is one too, and survived. |
| `60246b53` | **the f64 rete surface exists** — 4 comparators, **2 totality holes closed** (three f64 comparators missing; generic `<` falsely total), a casing bug of mine. |
| `7c0753ee` | **the EDN float writer round-trips** — `write_float`'s `<1e16` guard fell through to plain `Display` above it, doing the exact thing the guard exists to prevent. `1e200` wrote as 201 digits and failed on read. The fix **deleted the branch**. |
| `055389af` | **THE VSA SEAM IS OPEN** — `(f64::> (holon::cosine ?a ?b :undefined 0.0) 0.9)` runs. R4 designed this seam; it had never been opened. |
| `4c142126` | **one naming rule** — 46 rows renamed, `RETE_MODULES` 5 → **2**, closed forever. **Four unit tests** make the drift unrepresentable. |
| `a8f70871` · `fcc1958c` · `5f4c8e21` | f64 arithmetic · **`get` total by fallback** · `string::subs` — **round 2 CLOSED**. |
| `f440cf3a` | the `cond` brief (rider live on it) + `do` cut on derivation. |

## The rulings you inherit — do NOT re-litigate

1. **`:undefined` is THE fallback marker.** One spelling (arc 179's `()` lesson).
2. **±Inf and NaN are undefined** — builder-ruled; f64 arithmetic carries a fallback.
3. **`expect` is the DISCARD of an already-faced outcome.** `Option<T>` IS the faced form. **Never
   mint a rete row for `expect` or any verb defined via it.** The later core purge is 195
   registrations / ~617 call sites — a real crusade, correctly cut out of rete.
4. **`get`-with-fallback IS `nth`.** Not a gap; the same verb under the better name.
5. **`do` is CUT on derivation** — a discarded pure value cannot mean anything under the fence. Not
   a corpus verdict (R60).
6. **The rete name = the core name with `rete::` inserted after `wat::`** — nine documented,
   *tested* exceptions where a per-type row surfaces a generic core op.
7. **109 owns the `cond` rename.** Paren clauses now; the bracket form and its new name are 109's.

## ▶ WHAT IS NEXT

- **Weigh the `cond` rider.** It is the first **macro-backed** row: the `Form` arm re-dispatches at
  *runtime*, a macro is gone by then, and lookup is exact-name (`registry.rs:48`). **A row alone
  does nothing and looks fine** — scorecard row 2 is the only thing that catches it.
- `PersistentMap/contains-key?` — the last UNSURE-bucket straggler. **Audit, do not guess.**
- **Then S6 → S7**, which is where the real remaining work is: the corpus migration, then arming the
  third conjunct. ⚠ **The migration's size is stated THREE ways in the stone — 15 / 21 / 39.** The
  walker that settles it is on disk (`wat-scripts/scratch-pad/probe-where-census-walker.wat`). One
  run, not an argument. And **arm BY HAND, never a rider** — the checker screaming is what audits
  completeness (R52).

## ★★ THE LESSONS, and all three are about the RECORD, not the code

**A truncating pager makes absence unfalsifiable.** Four times — `| head`, `| tail`, two hand-picked
windows. The pattern was *right* every time; I threw the answer away myself and read the remainder as
complete. Once I was one sentence from reporting a handler as missing that sits at `runtime.rs:8251`.
`[[feedback_a_truncating_pager_makes_absence_unfalsifiable]]`

**A guard drawn one notch too tight makes the honest path non-compliant.** Three times: #52's STOP
left the holes it was chartered to fix; my STOP-3 forced four `.wat.bad` files to hold code the
checker *accepts*; a lint's false positive pushed a rider into laundering a literal. **Compliance is
what is mechanically checked, so compliance wins.**
`[[feedback_a_guard_drawn_too_tight_makes_the_honest_path_noncompliant]]`

**Two findings reported separately lose what they jointly proved.** I ruled `expect` out on one turn
and shipped the `Option` arm on another. Together they had *closed* the question — my summary still
listed it open until he asked *"didn't we just solve this?"*
`[[feedback_two_findings_reported_separately_lose_what_they_jointly_proved]]`

## What the builder's questions found that I had walked past

Four times a one-line question exposed something I had signed off on:

- *"we deleted f64 equality but kept i64?"* → the cut was symmetric; the **asymmetry was equality vs
  ordering**, on a test that cannot tell them apart.
- *"why are other containers not allowed?"* → **17 of 57 rows failed module admission.** Arming would
  have refused our own rows.
- *"was this a problem? did we compromise the syntax?"* → the probe's header claimed to prove
  parametricity and **every value in it was i64**. I had signed off twice.
- *"didn't we just solve this?"* → see above.

**Not one was found by an instrument I built.**

## Housekeeping

`MEMORY.md` was at 23.3KB against a 24.4KB load ceiling; the oldest **32** pointers moved to
`ARCHIVE.md` (70 + 479 = the same 102, verified by count). **This was the mechanical two-tier move,
not the judgment-curation** the 2026-07-04 breadcrumb says needs its own careful session — that is
still owed.

---

> **SEAM.** You are NEW. The disk is green and pushed — trust it over this note, and trust this note
> over your sense of having been here.
>
> `RETE_OPS` went **35 → 66** today, the VSA seam opened, and a wire defect died — but the thing to
> carry is none of those. It is that **every discipline that worked was the same discipline**:
> ask the tool to prove it can fail before believing it passed, compute the boundary instead of
> guessing it, and read the definition before writing the caller.
>
> **A RIDER MAY STILL BE IN THE FIELD. Read `git status` before you touch anything.**
>
> `QVOD TVEBAMVR, NOS TVETVR.` · `SCVTVM IDEM INDEX.` · `MACHINA CHAOS DOMAT.`
