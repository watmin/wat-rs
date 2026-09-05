# BRIEF — a ledger is a receipt, not a lock

Move `:fanout::seen`'s write from **claim time** to **after the outcome is emitted**. The ledger
stops being a lock and becomes a receipt, and the dead-owner loss has no form.

Read `DESIGN-a-ledger-is-a-receipt.md` first. `wat-scripts/scratch-pad/probe-a-ledger-is-a-receipt-not-a-lock.wat`
is the worked mechanism — `s1 emitted=0 LOST`, `s2 emitted=1 no-loss`, `s3 emitted=2
duplicate-not-loss`. **Copy its two-verb shape.**

⚠ Its client-side match is the shape that cost a compile last round: a generated client method
returns the **Response** directly (`CheckResponse::Absent`), *not* wrapped in `Reply::Check`.

## READ IN ORDER

| room | why you are there |
|---|---|
| `circuit.wat:45-66` | the `:fanout::Seen` surface. `claim` → **two** features: `check [queue seq] -> Recorded \| Absent` and `mark [queue seq] -> Ok`. `ClaimRequest`/`ClaimResponse` go |
| `circuit.wat:68-83` | the `defservice`. `claimed <- HashMap [String bool]` stays **this exact type** — the bool now means *reported*, not *claimed*. Counters `firsts`/`dups` become `recorded`/`skipped` |
| `circuit.wat:86-125` | the `claim` impl → two impls. `check` is **pure** (no state change). `mark` writes the receipt and bumps `recorded`; a `mark` on an existing key is idempotent and bumps nothing |
| `circuit.wat:410-470` | the worker's claim send + the T1 deadline `select`. It becomes the **`check`** call; the deadline/redial machinery is unchanged |
| `circuit.wat:475-500` | **the emit rule.** `first?` → `absent?`. On `Absent`: emit, then `mark`, then ack. On `Recorded`: skip the emit, still ack. ⚠ `:479`'s `_` arm asserts `"claim not First/Dup"` — it must learn the new arms |
| `circuit.wat:960-990` | the stats read that prints `seen-firsts` / `seen-dups` → `seen-recorded` / `seen-skipped` |
| `tests/services/probe_arc278_sane_circuit.rs:124` | parses `"seen-dups"`. **The one file outside circuit.wat you may touch**, and only to follow the rename |

## SKETCH

Worker, replacing the `first?` block:

```wat
absent? (:wat::core::match cresp
          ((:fanout::Seen::CheckResponse::Absent) true)
          ((:fanout::Seen::CheckResponse::Recorded) false)
          (_ (:wat::kernel::assertion-failed! "fanout worker: check not Absent/Recorded" …)))
outs1   (:wat::core::if absent? (:wat::vector::conj outs0 (:fanout::Outcome …)) outs0)
_mark   (:wat::core::if absent? (:fanout::Seen/mark seen0 (… :queue name :seq sq)) nil)
```

**Order is the stone: emit, then mark, then ack.** A `mark` before the emit is claim-before
again under a new name.

## BLAST RADIUS

`circuit.wat` + the one `.rs` line above. No `wat/`, no `sqs.wat`, no `src/`, no codemod, no
nextest config.

## STOP TRIGGERS

- **STOP-1** — if `mark` cannot be issued **after** the outcome is appended and **before** the
  ack without restructuring the worker's outcome fold, STOP and report the shape. Do not move
  `mark` earlier to make it fit; that silently restores the defect.
- **STOP-2** — if `check` cannot be made side-effect-free (a read that writes is a lock wearing
  a receipt's name), STOP.
- **STOP-3** — if the rename spills beyond `circuit.wat` and `probe_arc278_sane_circuit.rs:124`,
  STOP and report every other site instead of editing it.
- **STOP-4** — do not add a lease, a TTL, an owner field, or a worker-local emitted-set. All
  three were considered and cut in the DESIGN, with reasons.
- **STOP-5** — no perf work. Report the timings; change nothing for them.

## PRIOR RESULT TO COPY FOR SHAPE

`SCORE-a-claim-remembers-its-owner.md` — the immediately prior NOT-STRUCK. Its discipline is the
one to repeat: it held a STOP rather than patching, and it named exactly what the next stone
needed.
