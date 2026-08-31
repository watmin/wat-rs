# BRIEF — excursus 001 stone 7: the fan-out proof, re-attempted

**The stone this whole detour was for.** Stone 4 was sent the wrong way by a guard that could
not see a parametric field type. That guard now sees it (stone 5), the queue is fixed (stone 6),
and the circuit's own instance of the same defect has surfaced. Fix it, drop the workaround it
forced, and run the proof.

## Three jobs

**1. `Outcome` into `:fanout::Worker`'s `:messages`.** `:fanout::Outcome` is declared at
`circuit.wat:30`, **beside** the surface at `:69`, and `Worker::DrainResponse::Ok` references it
as `(Vector :- [:fanout::Outcome])`. **`Envelope`'s exact twin, and the same lift for the third
time.** Measured at stone 6: a message may keep a non-prefixed name, so this is a pure move —
no rename, no call-site churn. The floor's one red goes green here.

**2. Remove the foreign-read workaround** (`circuit.wat:~127-140`). It currently does:

```wat
fr    (:wat::edn::read-foreign (:wat::edn::write e))    ;; round-trip the Envelope through TEXT
eid   (match (:wat::edn::ForeignRecord/get fr :id)   ((Some v) (format "{v}" :v v)) (None ""))
ebody (match (:wat::edn::ForeignRecord/get fr :body) ((Some v) (format "{v}" :v v)) (None ""))
```

Written when `Envelope` could not cross the fork. **It now can.** Replace with
`(:queue::Envelope/id e)` and `(:queue::Envelope/body e)`.

⚠ **Note the silent-zero path while you delete it:** `(:wat::core::None "")` means a missed key
yields `eid = ""`, and `ack` with `:id ""` acks nothing — no error, no signal. Whether that is
behind stone 4's `total=0` is **unmeasured**; the executor flagged it as a candidate. **Do not
assume it was the cause** — remove the workaround because it is obsolete, then measure what the
circuit actually does.

**3. Run the proof.** Stone 4's four properties, unchanged:

1. **fan-out completeness** — exactly `N × M` outcomes
2. **no loss** — a final `receive` on every queue returns empty
3. **parallelism proven by ids, not a clock** — all `M×J` worker ids appear
4. **duplicate count reported** — zero is a result; non-zero is a finding (STOP-2)

## The control — RE-DERIVED this session, not carried forward

An earlier stone's control was copied across a change that invalidated it and told the executor
to void a good run. This one is measured **now**:

- **Today:** `--check wat-scripts/fanout/circuit.wat` → **1**,
  *"references `:fanout::Outcome` which is not declared"*.
- **After:** `--check` → **0**, and the summary is **non-zero**.

Stone 4's summary was `n=12;m=2;j=2;total=0;distinct=0;dup=0;workers=0;empty=0`. Reading the
field names, a correct small run should give `total = n×m = 24`, `distinct = 24`, `workers = 4`,
`empty = 2`. ⚠ **That reading is mine, from the names — if the field semantics differ, say so
and report the shape you actually get.** The load-bearing claim is *non-zero and internally
consistent*, not a literal string I guessed.

## Weight — small on the floor, full standalone

Stone 4 ran `n=12, m=2, j=2`. Keep that (or near it) as the **floor** fixture — the floor kills
at 30s by default and there is precedent for the split (`wat-scripts/perf/grid/` heavy +
`wat_scripts_grid_axes_live` light).

Then run it **standalone at weight**: host measured at **12 cores, 28 GB free** — target
`N=2000, M=4, J=3` → 12 workers, ~18 processes, **8000 outcomes**. Report the numbers, or the
number it broke at.

## Read in order

1. **`wat-scripts/queue/sqs.wat`** — see how stone 6 lifted `Envelope`; job 1 is the same edit.
2. **`wat-scripts/fanout/circuit.wat:30`, `:69`, `~:127-140`** — the twin defect and the workaround.
3. **`SCORE-stone-4-fanout-circuit.md`** — the four properties and why `dup=0` was called vacuous.
4. **`docs/CIRCUIT.md`** — `main` is wiring only. Still binding.
5. **`tests/services/probe_ex001_queue.rs`** — drives the shipped program via `startup_from_file`.
   Copy that shape; do not make a second copy of the app to test.

## STOP triggers

1. **If more than one queue service per queue appears — STOP.** The actor's serialization is the
   safety argument; J *services* over one store reintroduces the race.
2. **If a duplicate delivery is observed — STOP AND REPORT.** Do not dedupe it away. That is the
   finding this proof exists to produce, and it is now finally reachable.
3. **If a property needs a sleep or a wall-clock — STOP.** `mora`. The clock is an argument.
4. **If a THIRD instance of the beside-the-surface defect appears — report it and fix it**, and
   say so plainly. Two have been found one stone apart because the first masked the second; a
   type-check halts at the first error, so **do not assume the current red is the last one.**
5. **If the standalone run exhausts the host — scale down and report the number it broke at.**

## Blast radius

`wat-scripts/fanout/circuit.wat` · one floor fixture (`.rs`) · this excursus's SCORE.
**Zero changes to `wat/`, `src/`, `crates/`, `wat-scripts/topic/`, `wat-scripts/queue/`.**

## Verify — never through a pipe

```bash
./target/release/wat --check wat-scripts/fanout/circuit.wat; echo "CHECK=$?"
./target/release/wat wat-scripts/fanout/circuit.wat
./scripts/floor.sh; echo "FLOOR=$?"
```

Floor is **5126 with 1 red** (the loader gate, on `:fanout::Outcome`). This stone must take it
to **FLOOR=0**.
