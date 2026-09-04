# SCORE — the ★ was unachievable, and C10's proof was vacuous

> **Written after the orchestrator's own weighing.** The central correction was verifiable from data
> the orchestrator had already collected and had not followed through.

## The scorecard, graded

| # | required | result |
|---|---|---|
| 1 ★ | each counter carries ONE unit | ✅ `alpha.rs:195` emits `alpha:leaf-fill-pairs`; `compiled:calls` keeps only the two per-call sites |
| 2 ★ | a lost call site is detectable | ⚠ **THE ROW WAS UNACHIEVABLE AS WRITTEN** — see below. Met by a change outside the stated blast radius, flagged rather than smuggled |
| 3 ★ | the duplicate pin is gone | ✅ `accum_matcher_op_census` pins no 80,200 under any name; `accum_alpha_memory_shape` keeps its own |
| 4 | the honest value is stated | ✅ `calls == 0`, with the reason as a property of the world, and the workload that *does* enter the path NAMED |
| 5 | the liveness guard stops over-claiming | ✅ it names only `alpha:leaf-fill-pairs`, which it can observe |
| 6 | the rotted citation | ✅ fixed — and replaced with **symbol names, no line numbers**, so it cannot rot again. A second rotted citation found and fixed (`accum_alpha_cost.rs:1341` → `compiled_cond.rs:928`) |
| 7 | not the split C10 forbade | ✅ argued, and **C10's own evidence indicted** — below |
| 8 | nothing else silently re-defined | ✅ 2 assertions, ~20 prose quotes, **3 dated stones quoting it as a CALL count** — all enumerated |
| 9 | floor / lints / clippy | ✅ **`5407 tests run: 5407 passed (2 slow), 21 skipped`** (441.9 s), **0 FAIL rows**, lints **258**, clippy rc=0 |

## ⛔⛔ THE SIXTH FALSE CLAIM WAS THE ★ ITSELF — AND MY OWN DATA REFUTED IT

EXPECTATIONS row 2 required *"deleting `delta.rs:78`'s bump REDs **after** the split — and
demonstrably does **not** before."*

**The "after" half is impossible within the blast radius I wrote.** On this axis both per-call sites
fire **zero** times. Deleting one changes no count, before or after. **The split makes the COUNTER
honest; it does not make the SITE observable** — and I had conflated those two when I wrote the row.

The refutation was already in my own hands: I drove `compiled:calls` to **0** by renaming the
product. Zero contribution means deletion is a no-op means no assertion on that counter can see it.
**I collected the datum and did not follow it through.** The rider proved it both ways rather than
quietly satisfying the row:

- **BEFORE** (HEAD, `delta.rs:78`'s bump deleted): counter still **80,200**, `accum_matcher_op_census`
  **green**. A whole call site removed and nothing in the suite could see it — `accum_cost.rs:38` was
  its only reader in the tree.
- **AFTER, within my scope**: still green, for the reason above.

**Cure, flagged not smuggled.** `c4_probe_bind_only_decides_skip_span_for_the_accum_axis` already
drives `alpha_activate_fact` per fact and hits both bump sites. It now asserts the **invariant, not a
constant**: both arms visit the same (fact, candidate alpha) pairs and each pair executes exactly one
compiled condition, so the two counts must be **equal** — they differ in work per call, never in
number of calls. The rider named this as outside my stated radius and offered to strip it, with the
honest consequence attached: *"then row 2 goes unmet and should be recorded as unmet rather than as
delivered."*

**Orchestrator's ruling: it stays.** My blast radius was written before I knew the per-call sites
were dead. Stripping it would leave a renamed counter and no proof.

## ⛔⛔ AND C10'S EVIDENCE WAS VACUOUS — ITS ROW IS MARKED ✅ CLOSED ON IT

`accum_cost.rs:54-62` argued the counter is a *designed union of interchangeable arms* because:

> *"driven 2026-09-02, forcing `skip_span = false` at `fire/delta.rs:71` left this test PASSING at
> 80,200"*

**That experiment could not have come out any other way.** Both arms contribute **zero**; the entire
80,200 came from a third site. Flipping a branch between two arms that feed nothing cannot move a
total they do not feed. The conclusion is true in form; the drive that "proved" it **could not
distinguish "designed union" from "both arms are dead here."**

Same family as C16 — a differential agreeing with the corruption by construction — and as the
`assert!(!ok)` half that could not fail. **A proof whose experiment has one possible outcome is not a
proof**, and this one closed a row.

## The four mutations

| # | what | result |
|---|---|---|
| 1 BEFORE | delete `delta.rs:78` bump, HEAD tree | counter **80,200**, test **GREEN** — the finding |
| 1 AFTER | same deletion, strike tree | new gate REDs on its **liveness** arm |
| 2 | old shared key restored | passes while measuring nothing about the compiled path |
| 3 | `ids.len() * 0` | `alpha:leaf-fill-pairs` REDs; `compiled:calls` untouched — **independent** |
| 4 ★ *(rider's own)* | delete `compiled_cond.rs:928` bump | REDs on the **equality** arm |

**Mutation 4 was not in my brief and should have been.** Mutation 1 reaches only the liveness arm; a
two-arm gate needs two mutations. The rider added it unprompted — `[[one-mutation-cannot-prove-a-multi-arm-gate]]`,
applied by the executor because the orchestrator did not.

## What the test now says

`calls == 0`, with the reason stated as a property of the world rather than a threshold: every class
on this axis is uniform and packable so no fact takes the activate path, and round 1's 1,000 derived
facts are classes no rule condition mentions, so `candidates_into` returns empty. It names the
workload that *does* enter the path, and carries a ⛔ against re-dialling (200, 200) and a ⛔
restating C10.

## Honest deltas — five more corrections to my artifacts

- **My DESIGN contradicts itself on the new key's name** — "The drive" says `compiled:leaf-fill-pairs`,
  the pinned contract says `alpha:leaf-fill-pairs`. The probe's throwaway leaked into the design. The
  rider took the contract section; BRIEF and EXPECTATIONS agree with it.
- **Mutation 1's framing assumed a call-count assertion existed.** None did. **Creating one is the
  actual work of this strike**, and it is absent from my blast radius, my file list and my runtime
  estimate.
- **"80,200 is load-bearing in two files"** — two files *assert* it, ~20 *quote* it, and **three dated
  stones quote it specifically as a CALL count** (`DESIGN-STONE-fire-i64-columns:4`,
  `packed-fire-rows:37`, `exec-ops-split:12`). True when measured in August; false since D7 moved the
  seed off `exec_compiled`. Left alone as dated measurements — **they are why C14 was believable.**
- **Three counters in that census read 80,200 simultaneously** (`alpha:leaf-fill-pairs`,
  `elem-card:0`, `bind-card:ELEMENTS`). So "the pin matches" was never discriminating, and my
  scorecard did not warn about it.
- **Runtime 110 min against a predicted 50–80** — five release rebuilds for the mutation matrix. My
  estimate did not cost the rebuilds.
