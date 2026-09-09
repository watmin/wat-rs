# BRIEF — find the 934 milliseconds

Find what the send arm's restructure costs, name the mechanism, and recover the time **without
giving up the retry**. Read `DESIGN.md` first — it carries the A/B
control and the two hypotheses already killed.

## THE HARNESS — reproduce the A/B before you change anything

This is the measurement everything else is graded against. Same box, quiet, `sqs.wat` swapped in
place:

```bash
cd ~/work/holon/wat-rs
ps -eo args | grep -E 'cargo|nextest|release/wat' | grep -v grep   # must be quiet

cp wat-scripts/queue/sqs.wat /tmp/sqs-with-retry.wat
git show 853704d24~1:wat-scripts/queue/sqs.wat > wat-scripts/queue/sqs.wat
for i in 1 2 3; do ./target/release/wat wat-scripts/fanout/circuit.wat \
  | grep -oE 'setup=[0-9]+;publish=[0-9]+;drain=[0-9]+;stop=[0-9]+'; done

cp /tmp/sqs-with-retry.wat wat-scripts/queue/sqs.wat
for i in 1 2 3; do ./target/release/wat wat-scripts/fanout/circuit.wat \
  | grep -oE 'setup=[0-9]+;publish=[0-9]+;drain=[0-9]+;stop=[0-9]+'; done
```

Expect ≈ `publish` 23723 vs 24657. **If your two arms do not differ by roughly 900 ms, stop and
report that first** — the box, not the code, is then the variable.

## READ IN ORDER

| room | why |
|---|---|
| `sqs.wat:379-440` | the send arm as it stands — `nap`, `once-put`, `_ok`, then `s'` and the body |
| `git show 853704d24~1:wat-scripts/queue/sqs.wat` | the same arm before, with the body **inside** `((PutResponse::Success) …)` |
| `sqs.wat:771-833` | the ack arm, the same shape on the drain path — **and it costs nothing**, which is the control that already exists inside the file |
| `wat-scripts/scratch-pad/probe-what-a-closure-costs.wat` | the narrow closure measurement, and the shape to copy for a new micro-probe |
| `wat-scripts/scratch-pad/probe-what-a-closure-costs-in-a-wide-scope.wat` | the wide one — and an example of catching a flaw in your own probe before using its number |

★ **The ack arm is the strongest clue in the file.** It received the *same* restructure and sits on
the drain path, where the measured cost is zero. Whatever the send arm is paying, the ack arm is
not — find what differs between them.

## THE WORK

1. **Reproduce the A/B.** Numbers on disk before any edit.
2. **Bisect the restructure.** Build variants of the send arm and measure each with the harness —
   e.g. the closures hoisted or removed entirely; `_ok` collapsed back so the body sits inside the
   `Success` arm again with the retry expressed some other way; the arm count varied. **One
   variable per variant, three runs each, `publish` median reported.**
3. **Name the mechanism.** State what the interpreter is actually doing, with the measurement that
   shows it — not a plausible story.
4. **Recover the time**, keeping the retry's behaviour identical: `:Transient` retried on the
   a1/a2/a3 budget, everything else dying by its own name.

## MICRO-PROBE FIRST, CIRCUIT SECOND

A 40-second circuit run is a poor bisect instrument. Where a hypothesis can be put to a
`foldl`-over-`range` micro-probe like the two committed ones, **do that first** — it isolates the
mechanism and runs in a second. Use the circuit to confirm, not to search.

## BLAST RADIUS

`wat-scripts/queue/sqs.wat` and `wat-scripts/scratch-pad/`. **No `wat/`, no `src/`, no surface
change, no behaviour change.** If the mechanism turns out to live in `src/` — the interpreter
itself — that is STOP-3, not a licence to edit it.

## STOP TRIGGERS

- **STOP-1** — the two A/B arms do not differ by ~900 ms. Report the numbers; the box is the
  variable and every conclusion after that is noise.
- **STOP-2** — the cost cannot be recovered without weakening the retry. **Do not trade the retry
  for the milliseconds.** Report the measurement and hand it back.
- **STOP-3** — the mechanism is in the interpreter (`src/`), not in `sqs.wat`. **That is a
  finding worth more than this stone** — name it with its measurement and STOP. Do not edit `src/`
  under this brief.
- **STOP-4** — a variant that is faster but changes what the queue does. Behaviour is fixed;
  only the shape moves.
- **STOP-5** — anything outside the blast radius.

## GRADE AGAINST

`docs/excursus/2026/08/001-sns-sqs/what-does-publish-actually-cost/SCORE.md` — the arc's prior measurement stone, same shape.

Write `SCORE.md`, then `pulsare_yield kind=scored`.
