# DESIGN — four grid scripts that do not do what their headers say

**Status:** drawn 2026-09-08. Resolves **`3Q2`**, **`3F1`**, **`3T2`**, **`3M2`** — all L1, all in
`wat-scripts/perf/grid/`, all shell. ⭐ **Every one has a working model already in the same
directory**, which is what makes them a batch.

⚠ **Each site is a claim from the ledger. Fourteen rows resolved so far and SIX had something wrong
with the row itself.** Re-derive before curing; report the delta.

---

## `3Q2` — the outer non-vacuity is gated, the inner one is not

`check-where-shapes.sh:152` vs `:158`, `check-query-compat.sh:135` vs `:141`,
`check-spec-native.sh` — all three guard `if [ "$PAIRS" -eq 0 ]` (*did we discover any stems?*) and
**none guards `ROWS_TOTAL`**.

⛔ **So `PAIRS=5, ROWS_TOTAL=0` passes GREEN**, printing *"5 pair(s), **0 rows** — wat == Clara on
every shape"*. Five pairs each comparing nothing, and the success claim is vacuously true **with the
zero printed in its own message.**

**Cure — copy the sibling, do not invent:** `check-grid-speed.sh:85-90` already guards exactly this
class. Add the `ROWS_TOTAL` guard to all three, in that file's shape and wording.

## `3F1` — the file names the bug, fixes one side, keeps it on the other

`run-axis.sh:227-228` says it outright: *"stderr is CAPTURED, not discarded: `2>/dev/null` made a
wat-side failure loud but **REASONLESS** — you learned the axis produced nothing and never why."*
The wat side captures to `WAT_ERR` (`:236`) and `cat`s it on failure (`:253`).

⛔ **28 lines later the Clara side is `clojure … 2>/dev/null`** (`:264`), and its failure branch
(`:267-271`) echoes `$CLARA_OUT` — **stdout only**. A JVM exception or stack trace is discarded by
the one file that wrote down why discarding it is wrong.

**Cure:** capture and surface Clara's stderr symmetrically with wat's. The wat side is the model,
28 lines up.

## `3T2` — a header promising output the script never produces

`run-all.sh:23`: *"Emits every `#grid/Verdict` line from every axis, **then a summary tally on
stderr**."* **There is no tally anywhere in the file** — `tally|summary|total|count|seen` over all
143 lines returns the promise itself and one unrelated `fanout` dial comment.

**Cure — build the tally, do not delete the promise.** ⭐ `3T1` (the pairing row) found that
`ACCURACY`/`ORACLE_ACCURACY`/`PORT_ACCURACY` never reach the exit code; a tally counting `:match`
vs `:MISMATCH` and axes attempted vs completed is the natural place that signal becomes visible.
**Emit on stderr, as promised.** ⛔ Do not touch the exit code in this strike — that is `3T1`.

## `3M2` — a header claiming an architecture the file does not have

`check-where-shapes.sh:22`: *"Here the JVM tax is paid **ONCE no matter how large the corpus
grows**. Measured at 6 rows: wat 0.22 s + Clara 3.7 s."* But `clojure -Sdeps … -M "$clj"` sits at
`:103` **inside `check_pair()`**, called once per stem at `:149` — **38 cold JVM boots**, in a CI job
(`ci.yml:248`), under a header claiming one.

⛔ **PINNED: CORRECT THE CLAIM, DO NOT BATCH THE JVM.** Batching 38 boots into one is a real and
probably worthwhile change — and it is a *performance* strike with its own measurement, not a
one-line honesty fix. Doing it here would smuggle an unmeasured rewrite of a CI-invoked script into
a batch of prose cures. **Rewrite `:22` to say what the file does, state the measured cost of what it
actually does, and say plainly that batching is available and unmeasured.** That is an affirmative
statement of a known opportunity, not a deferral.

---

## The one contract decision, pinned

⛔ **`3Q2` AND `3F1` CHANGE BEHAVIOUR; `3T2` AND `3M2` DO NOT.** Keep them separable in the commit
history — if the floor or CI reddens, I need to know which half did it. Two commits, not one.

## Out of scope = rejected

- **Batching the JVM** (`3M2`) — see the pinned decision.
- **Wiring the accuracy signal to the exit code** — that is `3T1`, a different row.
- **`3W1`'s JDK provenance and `3P1`'s axis-pair coverage** — both need measurement, both are their
  own strike.
