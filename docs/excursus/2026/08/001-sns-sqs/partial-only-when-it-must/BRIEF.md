# BRIEF — partial only when it must

Take a prefix only when the request cannot fit in `cap` at all; otherwise all-or-nothing. Read
`DESIGN.md` first — it carries five dead models so you do not re-run any
of them.

## READ IN ORDER

| room | why |
|---|---|
| `wat-scripts/queue/sqs.wat:344-362` | the whole admission `let`. `n0`, `cap`, `depth`, `room`, `take0`, `take`, and the `take == 0` reply |
| `sqs.wat:352-354` | the three lines that change |
| `wat-scripts/scratch-pad/probe-the-server-manages-its-own-capacity.wat` | the existing probe — its cases still have to hold, with **one expected change** (see below) |

## SKETCH

Replace `take0`/`take`:

```wat
room  (:wat::i64::- cap depth)
room  (:wat::core::if (:wat::i64::< room 0) 0 room)
take  (:wat::core::if (:wat::i64::> n0 cap)
        room                                              ;; cannot ever fit whole → prefix
        (:wat::core::if (:wat::i64::<= n0 room) n0 0))    ;; fits in principle → all or nothing
```

Everything below is unchanged: `take == 0` → `Accepted 0`, no write; otherwise the first `take`
bodies, one `Store/put`, `Accepted take`.

## ⛔ AN EXISTING PROBE CASE CHANGES, AND IT IS CORRECT THAT IT DOES

`probe-the-server-manages-its-own-capacity.wat` has `cap 10`, depth 6, send 8 → today `Accepted 4`.
Under the new rule `8 <= cap 10`, so it becomes **`Accepted 0`** — it does not fit now, and it will
fit when the queue drains. **Update the probe and say so in the SCORE**; do not preserve the old
number. Its `drain5-send8` case (room 9, n0 8) becomes `Accepted 8`.

The case that must **not** change: `nsubs 7` publish 10 → 70 bodies > `cap 64` → prefix → a count
comes back and the service is alive.

## BLAST RADIUS

`wat-scripts/queue/sqs.wat` and scratch probes. **No `sns-fanout.wat`, no `circuit.wat`, no
`wat/`, no `src/`.** No `:cap` change.

## STOP TRIGGERS

- **STOP-1** — `n0 > cap` cannot be distinguished from `n0 > room` at this point. Report it; the
  whole rule is that boundary.
- **STOP-2** — the nsubs-7 case stops returning a count (assertion, hang, or `Accepted 0` forever).
  That is the livelock returning and the stone has failed.
- **STOP-3** — `queue-receive-calls` does **not** fall. **Report it as a finding, do not chase it**
  — it refutes the operation-count model and is worth more than this stone.
- **STOP-4** — anything outside the blast radius, or any `:cap` change.

## THE MEASUREMENT

`circuit.wat` ×5 reporting **`queue-receive-calls`, `dup`, `full-retries`, `publish`, and the
`outbox` histogram** together. Priors: 11859 / 2759 / 7900 / 70244 / 10759 items. Pre-regression:
5396 / 0 / 3770 / 22583 / 8000 items.

## GRADE AGAINST

`docs/excursus/2026/08/001-sns-sqs/the-server-manages-its-own-capacity/SCORE.md` — the stone this narrows.

Write `SCORE.md`, then `pulsare_yield kind=scored`.
