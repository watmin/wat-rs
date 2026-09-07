# BRIEF — fill deep, then drain

Make the circuit able to fill its subscriber queues to a known depth with nothing consuming, then
arm the workers and time the drain alone. Drive it from `argv`, one point per invocation.
`wat-scripts/fanout/circuit.wat` only.

## Read in order

1. **`circuit.wat:1802-1808`** — `run-with`'s twelve parameters. Two more are added at the end.
2. **`circuit.wat:1837`** — the subscriber queues' `:cap 32`, hardcoded. Becomes `sub-cap`.
   ⚠ `:1847` is the **inbox** `:cap 64` — leave it alone.
3. **`circuit.wat:1883-1890`** — `qclients` built by `conj` over `(range 0 m)`. They exist before
   `t-pub0`, so a sweep is available at the fill/drain boundary.
4. **`circuit.wat:1910`** — `_twgo`, arms the j topic-workers. **Stays where it is**: the topic must
   keep fanning out during the fill.
5. **`circuit.wat:1969`** — `_go`, arms the m×j subscriber workers. **This is the line that moves.**
6. **`circuit.wat:1974`** — `t-pub0`. **`:1990`** — `join-publishers` returns; the fill is over.
   **`:1995`** — `t-drain0`. **`:1996`** — `poll-until-drained … 4000`.
7. **`circuit.wat:930-1010`** — `sweep-of` and `poll-until-drained*` as the previous stone left them.
   `sweep-of` is what reports the fill depth; `snapshot-str` renders it.
8. **`circuit.wat:2040-2062`** — the `phases` format. Gains `fill-depth`, `arm`, and the pairs.
9. **`circuit.wat:2176-2185`** — `:user::main`. Gains the argv path.
10. **`wat-scripts/scratch-pad/255-stone-o-iv-c-1-span-still-points-at-caller.wat:18-22`** — the
    **worked argv idiom to copy**: `(:wat::runtime::argv)` then
    `(:wat::core::Option/expect (:wat::core::get argv 2) "usage: …")`. Index 2 is the first user
    argument; verified by running that file with and without an argument.
11. **`sqs.wat:111-121`** — `:cap` is `:durable` per-instance state, not a wire limit. **`sqs.wat:102,104`**
    — `:max-entries` / `:max-page` are the wire contract and **do not move**.

## The work

**1. `sub-cap` and `fill-first?`.** Append both to `run-with`. `:1837` uses `sub-cap`. Existing
wrappers (`run*`, `run-p*`, `run-chaos*`, `run-drop*`) pass **`32 false`** so every current caller
and every floor test is byte-identical in behaviour.

**2. Move the arm.** When `fill-first?`, the m×j subscriber workers are armed **after**
`join-publishers` (`:1990`), not at `:1969`. Two guarded bindings — one early, one late — each a
no-op in the other mode. Do not introduce ambient state to carry the choice.

**3. Split the phase.** Take `t-arm0` after the fill and `t-drain0` after the arming, so:

```
setup = t-setup0→t-pub0    fill = t-pub0→t-arm0
arm   = t-arm0→t-drain0    drain = t-drain0→t-collect0
```

**4. Report the fill depth.** Between `join-publishers` and the arm, one `sweep-of qclients` and
`snapshot-str` it into `phases` as `fill-depth=`. This is the row that proves the fill filled.

**5. Derive the liveness bound.** `poll-until-drained`'s `attempts` becomes a function of `n × m`
rather than the literal `4000`, so it scales with the work and still catches a hang. Say in a
comment how it is derived.

**6. `main` reads argv.** No args → exactly today's `run* 2000 4 3`. With args →
`<n> <m> <j> <sub-cap> <fill-first?>`, printing the same three lines.

## Sketch

```wat
;; in run-with's let, replacing the single `_go` at :1969
[_go-early (:wat::core::if fill-first? nil (:fanout::arm-workers! wpeers))
 t-pub0    …
 pub-pair  (:fanout::join-publishers ppeers)
 fill-sweep (:fanout::sweep-of qclients)          ;; the boundary observation
 t-arm0    (:wat::time::epoch-nanos (:wat::time::now))
 _go-late  (:wat::core::if fill-first? (:fanout::arm-workers! wpeers) nil)
 t-drain0  (:wat::time::epoch-nanos (:wat::time::now))
 …]
```

Lift the existing `_go` fold into `:fanout::arm-workers!` so both sites call one thing.

## Blast radius

`wat-scripts/fanout/circuit.wat` **only**. No `wat/`, no `sqs.wat`, no `sns-fanout.wat`. The inbox
`:cap 64` at `:1847` is untouched. New helpers are `:fanout::`-namespaced.

## STOP triggers

- **STOP-1** — if the fill deadlocks because the subscriber queues hit `sub-cap` before the
  publishers finish, **STOP and report the depth reached and the cap.** Do not raise a cap inside
  the code to make a run complete; the caller passes `sub-cap`.
- **STOP-2** — if the no-args path of `main` differs from today's behaviour in **any** field of the
  summary, **STOP.** Every existing caller must be byte-identical.
- **STOP-3** — if the liveness bound cannot be derived from `n × m` and would need a hand-picked
  constant, **STOP and say so.** The bound's ruling forbids raising it to fit a slow run.
- **STOP-4** — if `fill-depth` does not read `n` per queue on a `fill-first?` run, **STOP and report
  what it read.** A rate measured from a fill that did not fill is void, and this is the whole point.
- **STOP-5** — floor red on any arm: **STOP; do not re-run.** Capture whole, name the exact arm.

## Shape to copy

`DESIGN/BRIEF/EXPECTATIONS/SCORE-the-poller-sweeps-once.md` in this directory — same file, same
`phases`-line reporting discipline, and its SCORE carries the grading standard (gate on what the
stone controls; the wall clock is an observation).
