# BRIEF — the inbox visibility stops being an outlier

Make the topic-worker's inbox visibility a reachable parameter instead of a hardcoded `5000000000`,
then **sweep it and let the data choose the default.** One file, plus whatever the compiler forces.

**Read in order:**

1. `docs/excursus/2026/08/001-sns-sqs/the-inbox-visibility-stops-being-an-outlier/DESIGN.md` — the six
   precedent sites, the 67× measurement, and the hazard that makes this a sweep rather than an edit.
2. `wat-scripts/fanout/circuit.wat:2323` — `(:demo::mk-tw 5000000000 …)`. **The one site to change.**
3. `wat-scripts/topic/sns-fanout.wat:667` — `:demo::mk-tw`'s signature; param 1 is `vis-ns`.
4. `wat-scripts/topic/sns-fanout.wat:417` — where the worker spends it:
   `:visibility-ns vis :limit 10 :wait (UpTo (Milliseconds 250))`. **This is the hazard** — at 200 ms
   the visibility is shorter than the receive's own wait.
5. `wat-scripts/fanout/circuit.wat:2255-2260` — how the **subscriber** `vis` is already derived from
   `vis-ms`. Your new knob is symmetric with this one; copy its shape, do not merge with it.
6. `wat-scripts/fanout/circuit.wat`, `:user::main` — the CLI dispatch. `argv 8/9/10` are the chaos
   knobs added earlier today; yours is the next slot, optional with a default.

**Sketch:**

```
run-with … inbox-vis-ms                     ;; 16th param
inbox-vis (if (> inbox-vis-ms 0) (* inbox-vis-ms 1000000) <DEFAULT>)
:record (:demo::mk-tw inbox-vis (Handle/addr inbox-qh) qaddrs rate seed)
;; CLI: argv 11 -> inbox-vis-ms, optional, 0 = use DEFAULT
```

Choose `<DEFAULT>` **from your sweep**, and state the reasoning in the SCORE. The prior is
**200 ms** because six sites already use it, but the prior is not the answer.

**The sweep is the deliverable.** At minimum `inbox-vis-ms` ∈ {5000, 1000, 500, 200, 100}, run **both**
idle (×3) and 8-concurrent, on `50 2 2 32 false 0 <inbox-vis>` with `drop-ack-bp 1000 drop-seed 42`.
For each cell record: `dup`, inbox-tier `redeliveries`, `drain`, and whether the run hit slow mode.
Then confirm the chosen default on the write path: `2000 4 3 8192 true 1000`.

**Blast radius, as a property:** `wat-scripts/fanout/circuit.wat` is the only file that should need to
change — `mk-tw` already takes the value as a parameter, so `sns-fanout.wat` should not move. **The
compiler and the corpus gate are the census.** If either forces a change elsewhere, make it and name
it in the SCORE.

**STOP-1** — if `dup` is ever non-zero at any visibility value, **STOP the sweep there and report it
with the value and the run.** A duplicate delivery reaching a consumer is a correctness failure and it
outranks every timing number in this stone.

**STOP-2** — if the sweep shows the outlier was **right** — if short visibilities cost more in
`redeliveries` and duplicate fan-out than they buy in latency — **STOP and report that.** The stone
then lands as a comment at `:2323` explaining why 5 s is correct and the six other sites are the
anomaly. **That is a complete and valuable result**; it costs me a recommendation and buys a documented
constant.

**STOP-3** — if this cannot stay inside `circuit.wat`, **STOP and report what forced it** before
changing a second file.

**STOP-4** — on any red in the floor or corpus gate: do **not** re-run. Capture whole, name the exact
arm, surface it. ⛔ Concretely, from earlier today: I piped a chaos run through `grep`, lost the
failure text, then re-ran and got green — destroying the only evidence. Capture before you look.

⛔ **Do not change this constant in order to make anything pass.** The two chaos reds were fixed at
`18a86fe27` by progress-bounding the poller, without touching any rate, cap, or timeout, and they must
remain fixed *by that mechanism*. If a run reds, that is a finding, not a reason to reach for the knob.

**Run everything heavy through `./scripts/capped.sh`** (`--limit 8g`). CLI shape:
`circuit.wat n m j sub-cap fill-first? vis-ms drop-recv-bp drop-ack-bp drop-seed [inbox-vis-ms]`.
**Read the floor's Summary line, never a piped exit code** — and note a `--limit` kill shows as an
ABSENT Summary, which is itself a finding.

**Write your SCORE** to
`docs/excursus/2026/08/001-sns-sqs/the-inbox-visibility-stops-being-an-outlier/SCORE.md`, graded row by
row, everything uncommitted. Copy the shape of
`docs/excursus/2026/08/001-sns-sqs/the-drain-gives-up-on-a-stall-not-a-budget/SCORE.md`.
