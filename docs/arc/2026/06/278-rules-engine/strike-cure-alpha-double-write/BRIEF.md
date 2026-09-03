# BRIEF — cure the alpha double-write, and gate it with the oracle

The native engine silently drops a derived fact. The defect, the trigger and the repro are all
established — **your job is the cure and the gate, not the diagnosis.**

## Read in order

1. `docs/arc/2026/06/278-rules-engine/strike-two-writers-one-alpha/SCORE.md` — the full diagnosis:
   the two writers, the parametric-record trigger, the four angles and which are dead.
2. `wat-scripts/scratch-pad/d7-two-writers-one-alpha.wat` — the repro. Run it:
   `./target/release/wat <path>` prints `"native=2 oracle=3"`. **This is your acceptance test: it
   must print `native=3 oracle=3`.**
3. `src/rete/kernel/fire/pass/alpha.rs:57-133` — the seed loop's three-way split and the batch loop
   whose `wm.alpha.insert(aid, els)` at `:130` replaces the entry.
4. `src/rete/kernel/fire/delta.rs:96-105` — writer 1's push.
5. `src/rete/kernel/session.rs:309` and its doc at `:256` — `pack_i64_row` tests **runtime values**
   while the doc says **declared** fields. The gap is the defect.
6. `src/rete/kernel/arm.rs:330-336` — each node filed under exactly one `pat.type_head`, which is
   why both writers land on one `aid`.
7. `wat-scripts/scratch-pad/d7-pack-width-controls.wat` — the negative control for row width
   (`wide=3 narrow=3`). It must keep printing that.

## ★ The invariant to establish

> **No `aid` may receive both a push and a replace in one seed pass.**

Three shapes satisfy it — **class-uniform batching**, **declared-schema packability**, **a
non-replacing writer 2**. The DESIGN lays out the trade of each. **Choose one, argue it against the
other two, and state the cost.** Two things to weigh with real numbers rather than taste:

- the fire path is hot; say what your cure costs there and how you measured it;
- `FireSession` holds **no `TypeEnv`**, so declared-schema packability needs new state threaded —
  price that before choosing it.

## The gate is half the job

Ship a **native-vs-oracle differential over a parametric-record workload**, on the floor. Nothing on
the floor drives one today, which is why a fact-dropping bug survived.

⛔ **Do not gate with the `leaf_occ` differential.** It is blind here: `predicted` is built with the
same predicate that decides batch membership, so it compares writer 2's output to writer 2's output.
It read `extra=[]` while the fact was being dropped.

## Blast radius

`src/rete/kernel/fire/pass/alpha.rs`, `src/rete/kernel/fire/delta.rs`, possibly `session.rs`, plus a
gate with adjacent fixtures. **This IS the fire path.**

## STOP triggers

1. **If your cure changes element ordering within `wm.alpha[aid]`**, stop and report. `d_alpha` holds
   *indices* into that vector; re-pointing them is how the observed aliasing happened.
2. **If the repro still prints `native=2`** after your cure, stop — do not adjust the repro.
3. **If the cure needs a `TypeEnv` threaded into `FireSession`**, stop and report the call chain and
   the cost before doing it.
4. **If the width control stops printing `wide=3 narrow=3`**, stop — you have narrowed batching
   further than the defect requires.
5. **If you find a THIRD writer**, stop and report.

## Mutation proofs — run all three, report all three

1. **Revert your cure** → the differential gate must go RED with the dropped fact named.
2. **Break the class-uniformity/schema decision in the other direction** (make a uniformly-packable
   class fail to batch) → the width control or a perf arm must show it, proving your decision is
   read and not merely present.
3. **The width control** (`wide=3 narrow=3`) before and after, unchanged.

## What to report

- The repro's output before and after, verbatim.
- Which of the three shapes you chose, the argument against the other two, and the measured
  hot-path cost.
- All three mutation results.
- Scoped nextest Summary lines including `binary_id(wat::lint)`.
- Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.
- **Anywhere this brief was thin or wrong.** Nine riders have run on this arc and every one found a
  real defect in the brief — the last one found that the instrument I told it to trust was
  structurally blind to the bug it was hunting. Be blunt.

Do not commit.
