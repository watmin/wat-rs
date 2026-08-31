# EXPECTATIONS — excursus 001 stone 3: SQS in userland

**Written BEFORE the strike, 2026-08-30.** Blast radius derived from the BRIEF's own section.

## ★ The floor is FULLY GREEN — 5121, FLOOR=0

Every prior stone here ran against a known red and could point at it. **That cover is gone.**
Any failure in this stone's floor is this stone's.

## The scorecard

| # | what | expected |
|---|---|---|
| 1 | it is a DEMO, not stdlib | `wat-scripts/demos/sqs/`; `git diff -- wat/ src/ crates/` **empty** (the ⛔ decision) |
| 2 | send → receive | 3 sent, `receive` 2 returns 2 |
| 3 | received messages go invisible | a second immediate `receive` returns only the third |
| 4 | ack removes | the acked message never returns |
| 5 | ★ **redelivery** | the RECEIVED-but-UNACKED message **reappears** once its window passes |
| 6 | visibility is ONE put | no lock, no timer, no base-table read (STOP-2) |
| 7 | both backends agree | identical rendered summary, mem vs sqlite |
| 8 | ★ the `isk` boundary is demonstrated | a message visible at exactly `now` is returned — proven, not argued (STOP-3) |
| 9 | floor | `FLOOR=0`, 5121 + the new fixture's arms |
| 10 | zero substrate change | no `wat/`, no `src/`, no `crates/` |
| 11 | prior stones undisturbed | all `probe_ex001_*`, inst, write-opts arms PASS |

## Runtime prediction

**2–4 hours.** The surface is three ops, but the lifecycle fixture is the work — driving
send/receive/invisible/ack/redeliver across two backends and rendering a comparable summary.

## Trap-doors

1. **★ Row 5 is the row that can be faked.** A fixture testing only send/receive/ack passes
   without ever proving redelivery — and a queue that never redelivers silently loses every
   message a consumer failed to ack. It needs a *visibility window that actually elapses*,
   which means the fixture must control time rather than sleep. (`:wat::time::now` is real;
   `:wat::kernel::after` exists. If neither gives a deterministic way to advance past a
   window, **say so** — that is a finding about testability, not a reason to drop row 5.)
2. **Row 3 and row 5 are in tension.** Too short a visibility timeout and row 3 flakes; too
   long and row 5 cannot be observed. If the fixture needs a sleep to pass, it is wrong —
   `mora`: *"sleep is a guess; guesses race."* Prefer a queue whose timeout is a parameter the
   fixture sets to something it can step over deterministically.
3. **`scan-index` returns `IndexRow`, not `StoredRow`.** Re-putting means constructing a fresh
   `StoredRow` from the IndexRow's `pk sk data` plus NEW index-keys. If any field is missing,
   that is STOP-2 territory.
4. **The `bijection-anchor` wart** may reappear if the queue holds a peer — `sns-fanout.wat`'s
   header explains it. Not a new finding if it does; just do not re-derive the explanation.
5. **A stable `sk` needs a source.** The message id must be unique per send. Stone SORTKEY
   settled the analogous question for telemetry (`:wat::uuid::v4` at the producer). Follow that
   precedent unless there is a reason not to — and if you deviate, say why.

## Not in this stone

- Promotion to `wat/queue.wat` — the builder's, explicitly.
- Dead-letter queues, FIFO ordering guarantees, batch receive — not briefed, not assumed.
- Any change to `:wat::query::Store`.
