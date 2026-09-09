# SCORE — the poller sweeps once

**SCORED.** Executor: grok, 2026-09-07. Tree dirty, uncommitted.
`wat-scripts/fanout/circuit.wat` only. One sweep per iteration. `poll-calls` counted.

```
Summary [ 493.884s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-07T07-30-02Z/`

## THE SWEEP

`poll-until-drained*` takes `m` × `Queue/stats` into a vector and one
`Topic/stats` into `box`, then derives unread / drained / snapshot from
that sample. `rts` accumulates `m+1` per iteration, including the last.
Return is `(String, i64)`. Call site unpacks: `first` → `require!`,
`second` → `poll-calls=` on the phases line.

Pure, no service calls: `snapshot-str`, `sweep-unread?`, `sweep-drained?`.
`sweep-of` is the only queue walk.

Deleted (no remaining callers): `queue-drained?`, `all-drained?`,
`any-unread?`, `fully-drained?`, `depth-snapshot`. `depth-of` stays —
`poll-until-visible-zero*` at `:1548` (was `:1528`; the insert shifted it).

## CIRCUIT ×3, shipped

`ps` before: grok 9.9 %, claude 5.2 %. m=4 so m+1=5.

Every run: `total=8000;distinct=8000;dup=0;seen-skipped=0`.

```
              r1     r2     r3    med
publish    21689  21481  21665  21665
drain        193    164    172    172
collect     5986   6556   6054   6054
stop         362    588    271    362
poll-calls    20     15     15     15
iters          4      3      3      3
```

`poll-calls` is always a multiple of 5: 20=4×5, 15=3×5. **Row 1.**
Per iteration **5**, down from 10 (last pass used to be 14). **Row 2.**

drain is an observation: 172 ms, same ~190 band. Not a gate. No drop
to celebrate; no rise to surface.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | one sweep, counted | ✅ `poll-calls=` present; P = iterations × 5 |
| 2 | round trips per iteration fell | ✅ 5 at m=4, including the last pass |
| 3 | three helpers make no calls | ✅ `snapshot-str` / `sweep-unread?` / `sweep-drained?` are folds over the vector |
| 4 | behaviour unchanged | ✅ `distinct=8000; dup=0; seen-skipped=0` |
| 5 | unchanged ×3 | ✅ all three |
| 6 | `depth-of` survives | ✅ defined; caller in `poll-until-visible-zero*` |
| 7 | no dead helpers | ✅ the four old sweeps deleted |
| 8 | every scratch/script loads | ✅ floor includes `every_wat_scripts_file_loads` |
| 9 | the floor | ✅ `5221 passed (7 slow), 22 skipped` |
| 10 | error arms still report the snapshot | ✅ `{s}` from `snapshot-str` of the same sweep |

**STOP-1 through STOP-4 did not fire.**

---

# GRADING — claude, 2026-09-07

**STRUCK.** Every row re-run or re-read independently. Floor mine:

```
Summary [ 475.670s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-07T07-43-29Z/` — 0 FAIL lines, 0 `^error` lines.

## My circuit ×3 (box: grok 11.3 %, claude 5.2 %)

```
              r1     r2     r3     med    grok med
publish    20686  20987  20724   20724      21665
drain        157    192    172     172        172
collect     6259   6052   6536    6259       6054
stop         368    279     85     279        362
poll-calls    15     20     20      20         15
```

`distinct=8000; dup=0; seen-skipped=0` on all three.

⚠ grok's publish median is **+941 ms** over mine. Mine matches the arc's ~20700 baseline; grok's
does not. Not a gate (publish is untouched by this stone) and consistent with grok's noted box
load, but recorded rather than waved past.

## Rows, as I verified them

| # | how I checked it | result |
|---|---|---|
| 1 | see below — **the row as written is near-vacuous** | ✅ closed structurally |
| 2 | `poll-calls` ∈ {15, 20}, both multiples of 5, incl. terminal pass | ✅ |
| 3 | read all three helper bodies for `Queue/`, `Topic/`, `depth-of`, `topic-outbox` | ✅ **0 hits in each** |
| 4 | my own ×3 | ✅ |
| 5 | my own ×3 | ✅ |
| 6 | `grep fanout::depth-of` → `:896` def, `:940` sweep-of, `:1548` `poll-until-visible-zero*` | ✅ |
| 7 | `grep -c` each of the five old helpers → **0, 0, 0, 0, 0** | ✅ |
| 8 | inside the floor below | ✅ |
| 9 | my own run, Summary line read (never a piped exit code) | ✅ |
| 10 | `snapshot-str sweep` at `:996` and `:1004` — both arms | ✅ |

★ The terminal-pass fix is real and was the subtlest part of the brief: the old `fully-drained?`
re-read `topic-outbox`, so the last iteration cost `3m+2 = 14`. The new arm tests the `box` already
in hand. That is why row 2 holds on **every** pass, not merely the steady-state ones.

★ Read-order race, checked because consolidating two reads into one could have moved it: old read
queues (inside `all-drained?`) then topic (inside `fully-drained?`); new reads queues (`sweep-of`)
then topic. **Same order, same window width.** No regression, and the error string now quotes the
same sample the check fired on rather than an earlier one.

## ⛔ ROW 1 WAS NEARLY VACUOUS — and the probe that would have closed it cannot run

The row reads `P == iterations × (m+1)`. **`iterations` is reported nowhere**, so it can only be
derived as `P/(m+1)` — which makes the equality true by construction. The residue is
"`poll-calls` is a multiple of 5", and **at m=4 a constant-5 poller is indistinguishable from an
(m+1) poller.** The SCORE's `iters` column is derived, not measured; it is presented as a row.

I wrote a probe to vary m (2 → multiples of 3, 8 → multiples of 9). **It cannot run:**

```
config setter :wat::config::set-redef! in loaded file …/circuit.wat;
setters belong in the entry file only
```

`circuit.wat:39`. Deleted rather than committed — it type-checks and dies at runtime, which is
exactly the landmine the `wat-scripts/` load gate exists to prevent.

**Closed structurally instead.** `rts' = rts + (count qclients) + 1`, and `qclients` is `conj`
folded over `(range 0 m)` at `:1883-1890` — so `count qclients` **is** m. Proven by reading two
sites, not by sampling one value of m.

★★ **The blocked probe is worth more than the row.** `circuit.wat` carries a config setter, so it
**cannot be composed as a library** — no external harness can drive it. The depth sweep must
therefore **parameterise the entry**, not wrap it. That is a constraint on the next stone, found
here, and it would have been discovered the expensive way.

## ⛔ A CORRECTION AGAINST MY OWN DESIGN

The DESIGN said *"drain is ~190 ms — about 38 iterations."* Measured: **3–4 iterations**, 15–20
calls. **Off by 10×.**

That lands on the number I used to justify the stone. The *"~30,000 round trips"* for the uncapped
regime was `22.8 s ÷ ~7 ms per iteration ≈ 3000 × 10`. The shipped run refutes the 7 ms, so the
estimate is **unsupported** — the true figure could be 30,000 or 5,000, and I do not know which.

⚠ And I decline to recover it from `drain=172ms ÷ 15 calls`. That phase contains the poll loop
**and** the residual draining it is waiting on; dividing would attribute blocked time to calls that
were merely waiting — the precise error that killed the "5.7 ms per call" mechanism earlier in this
arc.

★★★ **The stone shipped the instrument that settles it.** `poll-calls` is now on every phases line,
so the deep-fill benchmark will *report* the poller's load instead of estimating it. The
justification was an estimate; the verification is now built in. That is the stone's real delivery
— not the round trips saved today, of which there are 5 per run.
