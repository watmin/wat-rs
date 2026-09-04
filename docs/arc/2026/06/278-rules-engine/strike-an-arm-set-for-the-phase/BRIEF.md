# BRIEF — give the filter phase an arm set that measures the branch it takes

Six arms measure `exec_where`. The fire calls it **zero** times on this axis. Build arms over the
branch the fire actually takes, and let the reconstruction report coverage it has measured.

## Read in order

1. `src/rete/kernel/fire/mod.rs:2012-2070` — `dispatch_where_tests`. **The branch the fire takes**
   is the first one: `bind_view` → `where_tree.candidates` → two `HashSet` builds → the `tid` loop →
   `d_beta.entry().or_default().push()`. The `exec_where` call the benchmark measures is at ~`:2068`,
   in the `else`.
2. `src/rete/kernel/tests/node_share_cost.rs:63-65` — C6's own note: *"calls `exec_where` ZERO times.
   Every arm below is still scaled to `evals_per_round`."* The problem, already written down.
3. `src/rete/kernel/tests/node_share_cost.rs:180-260` — arms A–F, and the interleaved-reps/medians
   harness. **Copy this harness for the new arms; do not invent a second measurement idiom.**
4. `src/rete/kernel/tests/node_share_cost.rs:360-395` — the refusal C6 recorded, its six samples
   (684/693/734/723/686/698%), and the prose naming what has no arm. Your arms are that list.
5. `docs/arc/2026/06/278-rules-engine/strike-stale-reconstruction/SCORE.md` — why the check is
   refused rather than asserted, in C6's words.

## Driven by the orchestrator at HEAD `deecfac6e`

```
C  token clone      0.132 ms      F  compiled exec_where  2.617 ms
RECONSTRUCTION F+C = 2.749 ms  vs a LIVE `filter` of 0.419 ms  ( 656% accounted)
```

Arm C is **31.5%** of the phase and is the only arm measuring work the fire performs. **~68.5% is
unmeasured.** ⚠ That 656% is **one sample** against C6's six (684–734). **Re-measure the pre-value at
six samples before reasoning from any movement** — one reading cannot establish a shift.

## The change

1. **New cumulative arms** over the taken branch — `bind_view` alone, `+candidates`, `+the two
   HashSet builds`, `+the tid loop`, `+d_beta pushes` — in the existing interleaved-reps/medians
   harness, so each row is a delta from the one above.
2. **Relabel A/B/D/E/F** to say they measure the `exec_where` branch, which this axis does not take.
   Keep them: `B-E` is the compile-headroom story and it is real.
3. **The reconstruction reports the new arms against the live phase**, states coverage as a measured
   fraction **with its spread**, and asserts an **invariant** if one holds.
4. **If the new arms still do not reconstruct the phase, that is the result.** Record it the way C6
   recorded its refusal — with the samples in-code — and say what is still unaccounted.

## Blast radius

`src/rete/kernel/tests/node_share_cost.rs`. **No `src/` engine change** — if a mechanism cannot be
measured without changing the fire path, that is STOP-2, not a licence.

## STOP triggers

1. **If an arm needs the fire path modified to be measurable**, stop and report. A hot-path edit for
   an instrument's benefit is forbidden here (C10's standing ruling).
2. **If the new arms sum to more than ~120% of the phase**, stop and report the numbers — that is the
   same over-accounting the old ladder had, and it means an arm is measuring something outside the
   phase or at the wrong scale.
3. **If reconstruction requires choosing a tolerance to make it pass**, stop. Report the measured
   coverage and its spread instead; an asserted number chosen to fit is what C6 refused to ship.
4. **If the six-sample re-measure disagrees with C6's 684–734 band**, stop and report before building
   on either figure.

## Mutation proofs — run all three, report all three

1. **Zero one new arm's work** (make its loop body a no-op) → the reconstruction's coverage figure
   moves by that arm's share, and any invariant that depends on it REDs. Proves the arm is wired to
   the number.
2. ★ **Force the `else` branch** (make `is_pure_cmp` return false for one tid, in the TEST harness'
   captured copy — **not** in the fire) → the `exec_where` arms become relevant and the taken-branch
   arms lose share. Proves the two ladders measure *different branches* and that the new one tracks
   the branch actually taken. **If you cannot do this without editing the fire, say so — that is the
   honest answer and STOP-1 applies.**
3. **Halve the token count** → every taken-branch arm scales with tokens and the coverage fraction
   holds. Proves the arms scale with the phase rather than with a constant.

Restore by **hash** — `git checkout <sha> -- <path>` STAGES.

## What to report

- The full STEP 0 table before and after, and the six-sample pre-value.
- Measured coverage of the new arm set, **with spread**, and what is still unaccounted.
- All three mutation results.
- Whether the reconstruction is now assertable; if not, why not, with numbers.
- Scoped nextest `Summary` lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong. Be blunt.** Six consecutive strikes had their ★ be a false
  claim in a file the brief said to trust — **four of those were the orchestrator's own artifacts**,
  most recently a scorecard row his own measurement had already refuted. Assume there is a seventh.

Do not commit.
