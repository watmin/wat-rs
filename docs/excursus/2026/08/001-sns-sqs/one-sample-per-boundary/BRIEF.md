# BRIEF — one sample per boundary

Collapse eleven `Queue/stats` round-trips per queue into one sample per phase boundary, and use those
samples to give **every** phase the busy metric only `drain` has today. One file.

**Read in order:**

1. `docs/excursus/2026/08/001-sns-sqs/one-sample-per-boundary/DESIGN.md` — the count, and why the
   instrument and the fix are the same change.
2. `wat-scripts/fanout/circuit.wat:1956-2030` — `:fanout::sum-calls`, `sum-ticks`, `sum-store-calls`,
   `sum-store-ns`, `sum-handler-ns`. **Five folds, five identical round-trips, one field each.**
3. `wat-scripts/fanout/circuit.wat:2606` — `:dbms (/ (- hn-after hn-before) (* 1000000 m))`. **This is
   the arithmetic to generalise**, not to invent.
4. `wat-scripts/fanout/circuit.wat:2500-2530` — the `sc-before`/`ns-before`/`hn-before` samples and
   `hn-after`, and the `t-drain0` / `t-collect0` boundaries they sit against.
5. `wat-scripts/fanout/circuit.wat:2526-2565` — collect's body: `sum-calls`, `sum-ticks`,
   `sum-store-calls`, `sum-store-ns` in sequence. **Four round-trips per queue, back to back.**
6. `wat-scripts/fanout/circuit.wat:1185` `:fanout::sweep-of` — the exemplar for a fold that takes one
   stats reply and keeps several fields. Copy its shape.

**Sketch:**

```
(defrecord :fanout::Sample [receive-calls ticks store-calls store-ns handler-ns])   ; all i64

:fanout::sample-of qclients -> :fanout::Sample
  foldl: ONE (Queue/stats q) per queue, add all five fields          ; like sweep-of

;; at each phase boundary, one sample:
s-fill0 … s-drain0 … s-collect0 … s-stop0
;; every former sum-* reader takes its field off the nearest sample
;; busy-ms for a phase = (handler-ns delta across its two boundaries) / (1e6 * m)
```

**Blast radius, as a property:** `wat-scripts/fanout/circuit.wat` only. **The compiler and the corpus
gate are the census** — if either forces a change elsewhere, make it and name it in the SCORE.

**The correctness gate is the IDENTITIES, not the digits.** Today's eleven round-trips read fields at
eleven different instants, so the reported numbers are already mutually inconsistent; one sample makes
them consistent. So:

- `put + delete + count + scan` calls and ns **must** sum to `store-calls`/`store-ns`, **remainder 0**,
  on every tier of every run
- `distinct = n×m`, `dup = 0`, inbox `accepted = n`
- **any absolute figure that moves must be named in the SCORE with its direction and why.** Do not
  quietly accept a shift, and do not force a shift to zero by re-adding a round-trip.

**Measure before and after**, `2000 4 3 8192 true 1000` ×3 each, box verified quiet, reporting every
phase: `setup fill arm drain collect stop total` plus `drain-busy-ms` and your new busy figures.

**STOP-1** — if any reconciliation identity breaks, **STOP and report it.** A remainder that is not 0
means the fields in one reply are not self-consistent, which would be a finding about the queue, not
about this stone.

**STOP-2** — if collapsing to boundary samples means some reader needs a value at a point where no
boundary exists, **STOP and report which reader and which point** before adding a mid-phase sample. A
sample inside a phase inflates that phase — the thing this design exists to avoid.

**STOP-3** — if `collect` does **not** drop materially, **STOP and report the numbers.** The whole
premise is that its 4583 ms is round-trip latency. If removing ~8 round-trips per queue barely moves
it, the cost is somewhere else and that is worth more than the tidy result.

**STOP-4** — on any red in the floor or corpus gate: do **not** re-run. Capture whole, name the exact
arm, surface it. ⛔ From earlier today: I piped a chaos run through `grep`, lost the failure text, then
re-ran and got green — destroying the only evidence.

⚠ **Do not claim a throughput win.** This removes harness round-trips; it does not make the system
faster per message. `drain` may barely move and that is fine.
⚠ **The new busy metrics are server-side handler time, not interpreter time.** They bound how much of a
phase is *waiting*. They do not prove the remainder is interpretation — say so.

**Run everything heavy through `./scripts/capped.sh`** (`--limit 8g`). **Read the floor's Summary line,
never a piped exit code.**

**Write your SCORE** to `docs/excursus/2026/08/001-sns-sqs/one-sample-per-boundary/SCORE.md`, graded row
by row, everything uncommitted. Copy the shape of
`docs/excursus/2026/08/001-sns-sqs/the-inbox-visibility-stops-being-an-outlier/SCORE.md`.
