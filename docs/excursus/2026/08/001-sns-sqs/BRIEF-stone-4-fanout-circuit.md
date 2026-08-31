# BRIEF — excursus 001 stone 4: the fan-out circuit — proving wat-topic and wat-queue compose

**The app that proves the pair is usable, not just present.** N messages in, N×M outcomes out,
processed in parallel across real processes.

> Builder: *"proof that our sns and sqs clones are actually usable in some app who needs
> parallel processing of messages… one message goes in, N outcomes come out from it."*

## It is a CIRCUIT, and that is this project's own word

`docs/CIRCUIT.md:3` — *"A wat program is a circuit. Programmer-built. Fixed topology. Signals
flow through wires once powered… **The shape of `:user::main` is the wiring diagram.**"* And its
rule: *"main constructs all pipes, plugs each into its consumer, and starts the input stream.
That's its entire job. No computation in main."*

**Hold to that.** All wiring in `main`; the work lives in the workers.

```
                          N input messages
                                 │
                                 ▼
                        ┌────────────────┐
                        │   1  TOPIC     │   wat-topic
                        └───┬────┬───┬───┘
                            │    │   │  fan-out
                    ┌───────┘    │   └──────┐
                    ▼            ▼          ▼
                 ┌─────┐      ┌─────┐    ┌─────┐
                 │ Q1  │      │ Q2  │    │ QM  │   M queues · wat-queue
                 └──┬──┘      └──┬──┘    └──┬──┘   ★ ONE service instance each
              ┌─────┼─────┐   ┌──┼──┐    ┌──┼──┐
              ▼     ▼     ▼   ▼  ▼  ▼    ▼  ▼  ▼
             w₁    w₂    w₃  w₄ w₅ w₆   w₇ w₈ w₉    J workers per queue, each a PROCESS
              └─────┴─────┴───┴──┴──┴────┴──┴──┘
                                 │
                                 ▼
                         N × M outcomes
```

## ★ The topology IS the safety argument — do not rearrange it

`receive` is `scan-index` **then** `put` — two Store calls. What makes that safe is **not** the
put's atomicity; it is that **a `defservice` is a serializing actor**: *"the actor's loop is
the ONE place mutation"*, *"`Outcome::Reply new-state reply` rebinds `state` for the next"*
(`wat/query/mem.wat:22-24`).

So: **ONE queue service instance per queue, J workers dialing it.** J *queue services* over one
store would each serialize internally and not against each other — that reintroduces the race.

Real SQS closes the same hole with storage-layer atomicity; ours closes it with the actor.

## The four properties — and only the last two are interesting

1. **Fan-out completeness** — exactly `N × M` outcomes. Not `N×M − 1`.
2. **No loss** — every message acked; a final `receive` on every queue returns empty.
3. **Parallelism actually happened** — ★ **not by wall-clock.** `mora`: *"sleep is a guess;
   guesses race."* Each worker is given an **id at spawn** and stamps it on every outcome; the
   assertion is that **all `M×J` ids appear**. Since `:locus (:wat::spawn::process)` already
   guarantees a distinct process per worker, ids-did-work *is* processes-did-work.
   ⚠ Do NOT reach for a self-pid. `:wat::kernel::peer-pid` answers *the pid at the other end*,
   not self; whether a process can name its own pid is **unverified** and this design does not
   need it.
4. **The actor serializes under load** — ★ **zero duplicate deliveries at `M×J`-way
   concurrency.** Today that safety is derived from reading a comment. This is the first thing
   that measures it.

## Weight — full standalone, scaled on the floor

The floor kills a test at **30s** by default (`.config/nextest.toml:10`), with per-target
overrides up to 600s. **Precedent for the split already exists**: `wat-scripts/perf/grid/` holds
heavy programs while `wat_scripts_grid_axes_live` drives a subset on the floor at ~33s.

- **Standalone, full weight** — host is **12 cores, 28 GB free** (measured). Target
  `N=2000, M=4, J=3` → 12 workers, ~18 processes, **8000 outcomes**. Runs as a program, prints
  its summary.
- **On the floor, scaled** — whatever fits comfortably inside the default budget
  (`N=100, M=3, J=2` or similar). **Same code, smaller numbers**, so the floor proves the
  circuit works and the standalone run proves it works *at weight*.

If the scaled version cannot fit the default budget, **say so and propose the override** rather
than adding one silently.

## Read in order

1. **`docs/CIRCUIT.md`** — the wiring axiom. `main` is the diagram; no computation in it.
2. **`wat-scripts/topic/sns-fanout.wat`** — wat-topic. Note its `bijection-anchor` wart and the
   grant-before-dial ordering its header explains; you need both at M queues.
3. **`wat-scripts/queue/sqs.wat`** — wat-queue. **The clock is an argument** (`now-ns` on send
   and receive) — the workers supply it, which is also what makes the visibility window
   testable without a sleep.
4. **`tests/services/probe_ex001_queue.rs`** — drives the shipped program via
   `startup_from_file`, `assert_eq!` on a whole summary. Copy that shape; do not make a second
   copy of the app to test.
5. **`wat-scripts/perf/grid/run-axis.sh`** + `wat_scripts_grid_axes_live` — the heavy-standalone
   / light-floor split.

## ⛔ Placement — propose, do not decide

`wat-scripts/fanout/` is the suggestion. It is neither wat-topic nor wat-queue; it *composes*
them. **If you think it belongs inside `topic/` or `queue/`, or under another name, say so in
the SCORE — do not move the existing two.**

## STOP triggers

1. **If you find yourself spawning more than one queue service per queue — STOP.** That is the
   race, and the topology is the safety argument.
2. **If a duplicate delivery is observed — STOP AND REPORT IT.** Do not "fix" it, do not retry,
   do not dedupe it away. **That is the finding this stone exists to produce.** Report the
   count, the concurrency it appeared at, and whether it reproduces.
3. **If the assertion needs a sleep or a wall-clock comparison — STOP.** `mora`. The clock is
   already an argument; if a property cannot be proven without timing, name it and leave it
   unproven rather than proving it badly.
4. **If the standalone run exhausts the host** (18 processes × 2000 messages) — scale it down
   and **report the number it broke at.** That is a real limit worth knowing, not a failure.
5. **If the floor reds at all — STOP**, capture whole, do NOT re-run. Floor is **5122, FULLY
   GREEN**. There is no known-red to hide behind.

## Blast radius

`wat-scripts/fanout/` (new) · one floor fixture (`.rs`, driving the shipped program) · this
excursus's SCORE. **Zero changes to `wat/`, `src/`, `crates/`, `wat-scripts/topic/`, or
`wat-scripts/queue/`.** If wat-topic or wat-queue needs a change to make this work, **that is a
finding** — say which and why, and stop.

## Verify — never through a pipe

```bash
./scripts/floor.sh; echo "FLOOR=$?"
```

`.contains(` on a deterministic string trips `no_loose_string_assert`; use `assert_eq!` on the
whole summary from the start.
