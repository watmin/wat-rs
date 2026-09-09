# BRIEF — the cold counters ride one carrier

Move the six **rarely-changed** counters out of `:queue::queue::State` into one
`:queue::Counters` record. `State` goes 29 fields → 24, and 172 pass-through copy lines become 30.
Everything stays inside `wat-scripts/queue/sqs.wat`.

The six: `ticks`, `acks`, `sends-accepted`, `sends-refused`, `redeliveries`, `expired-waiters`.

**Read in order:**

1. `docs/excursus/2026/08/001-sns-sqs/the-cold-counters-ride-one-carrier/DESIGN.md` — the
   change-frequency table, and **why "one carrier for all 18 counters" is wrong**.
2. `wat-scripts/queue/sqs.wat:155` — the `:ephemeral` block. The 24 fields `State` will keep and the
   6 it will shed.
3. `wat-scripts/queue/sqs.wat:113` — `:queue::Stats`. **Read it to confirm you must NOT touch it:**
   its fields are read 32 times across 8 files, and it is built at one site, once per call.
4. `wat-scripts/queue/sqs.wat:1319` — the single `(:queue::Stats …)` construction. This is where the
   six get read back out of the carrier and passed into the unchanged flat record.
5. `wat-scripts/queue/sqs.wat:417` and `:434` — the `:init` zeroing. The carrier is initialised here.
6. `wat-scripts/queue/sqs.wat:470-495` — a worked rebuild site (the `Accepted 0` arm, which is one of
   the few that *changes* a cold counter: `sends-refused`). Copy its shape.
7. `wat-scripts/fanout/circuit.wat:1135` — the report line's format string. **Contract: it must come
   out byte-identical.**

**Sketch:**

```
(defrecord :queue::Counters
  [ticks acks sends-accepted sends-refused redeliveries expired-waiters])   ; all i64

State :ephemeral   … 24 fields …  counters <- :queue::Counters

;; a site that changes NO cold counter  (≥22 of 30)
:counters (:queue::queue::State/counters s)          ; one line replaces six

;; a site that changes one  (≤8 of 30)
:counters (:queue::Counters :ticks (:queue::Counters/ticks c) … :acks (+ (:queue::Counters/acks c) 1) …)
```

**Blast radius, as a property rather than a list:** `wat-scripts/queue/sqs.wat` is the only file
that should need to change, because `:queue::Stats` keeps its shape and the report line keeps its
format. **The compiler and the corpus gate are the census** — if either forces a change elsewhere,
make it and name it in the SCORE. Do not contort the design to stay inside a boundary I drew.

**Measure before you change anything.** Take a `drain` baseline on the current tree, three runs at
each of n=1000/2000/4000, `circuit.wat <n> 4 3 8192 true 1000`, box verified quiet. Then the same
after. Report medians **and** spreads — a claim of improvement needs non-overlapping spreads, and my
own single-run baseline (1194 / 2506 / 5305) is one sample, not a band.

**STOP-1** — if `drain` does not move, or moves the wrong way, **STOP and report that with the
numbers.** The whole stone rests on `the-store-reports-time-per-operation/SCORE.md:177` naming
**allocation** as the mechanism. If removing 5 net fields from a record rebuilt 30× per message
changes nothing measurable, **that mechanism is wrong**, and saying so is a complete and valuable
result — worth more than the 142 deleted lines.

**STOP-2** — if a seventh field turns out to be cold, or one of my six turns out to be changed at
more sites than the DESIGN's table says, **STOP and report the real counts.** The table came from
`grep -c` on a pass-through pattern; it is my measurement and it may be short.

**STOP-3** — if the change cannot stay inside `sqs.wat` — if `:queue::Stats` or the report line has
to move — **STOP and report what forced it** before changing a second file. A 32-site corpus
migration is a different stone and it needs the builder's ruling.

**STOP-4** — on any red in the floor or the corpus gate: do **not** re-run. Capture the whole log,
name the exact failing arm, surface it. A green re-run destroys the only evidence.

**Run everything heavy through `./scripts/capped.sh`** (`capped --limit 8g …` for a bounded run).
A codemod census took this box down today; the wrapper is why it cannot happen again. **Read the
floor's Summary line, never a piped exit code** — `cargo … | tail` returns `tail`'s status, and a
`--limit` kill shows up as an absent Summary, which is itself a finding.

**Write your SCORE** to
`docs/excursus/2026/08/001-sns-sqs/the-cold-counters-ride-one-carrier/SCORE.md`, graded row by row
against EXPECTATIONS.md, leaving everything uncommitted. Copy the shape of
`docs/excursus/2026/08/001-sns-sqs/the-inbox-holds-messages-not-pairs/SCORE.md`.
