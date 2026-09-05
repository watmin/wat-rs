# BRIEF — the window gets a test

Add the third redelivery case: a redelivery arriving **mid-processing**, which under record-after
duplicates rather than absorbing. Gate that it never *loses*.

Read `DESIGN-the-window-gets-a-test.md` first — especially the contract decision on what is
gated versus reported.

## READ IN ORDER

| room | why you are there |
|---|---|
| `circuit.wat:812-820` | `mk-worker` — `delay-ms` becomes `ack-delay-ms`; add `work-delay-ms <- :wat::core::i64`. **6 call sites, all in this file** |
| `circuit.wat:520` | `_nap` — today it sits after `mark`, before `ack`. That is the **ack** delay and it stays |
| `circuit.wat:486-500` | the emit path — the **work** nap goes between the `check` result and the `conj` of the `Outcome` |
| `circuit.wat:1770-1800` | `:user::redelivery-is-absorbed` — the fixture shape to copy: two workers, `vis=200000000`, one queue, one message |
| `tests/services/probe_arc278_sane_circuit.rs:111-131` | `redelivery_is_absorbed_by_the_consumer` — the test shape to copy |

## SKETCH

Fixture: two workers, `vis = 200000000`, `ack-delay-ms = 0`, `work-delay-ms = 350`.

Test:

```rust
// distinct is the invariant and is GATED. total/dup are observations and are REPORTED.
assert_eq!(field(&stored, "distinct"), "1",
    "a redelivery mid-processing must not LOSE the message; got {stored}");
eprintln!("MID-PROCESSING {stored}");   // total is 2 today; 1 would be an improvement
```

## BLAST RADIUS

`circuit.wat` and `probe_arc278_sane_circuit.rs`. No `wat/`, no `sqs.wat`, no `src/`, no codemod.

## STOP TRIGGERS

- **STOP-1** — if the work nap cannot be placed between the `check` result and the emit without
  restructuring the fold, STOP and report the shape.
- **STOP-2** — **do not gate `total`.** `distinct=1` is the assertion; `total` and `dup` are
  `eprintln!`. A gate on `total=2` reds on a future improvement and pins today's behaviour by
  accident. This is the contract decision.
- **STOP-3** — if the new test is not deterministic (`distinct` varies across 6 runs), STOP and
  report the spread. A flaky floor test is worse than no test.
- **STOP-4** — do not change what the window does, do not touch the two existing redelivery
  tests, no perf work.

## PRIOR RESULT TO COPY FOR SHAPE

`SCORE-the-queue-can-drop-too.md` — its grading is where the gate-versus-report rule was written
down, and this stone is its first application.
