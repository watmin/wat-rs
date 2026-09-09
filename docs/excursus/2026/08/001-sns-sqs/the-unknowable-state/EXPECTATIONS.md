# EXPECTATIONS — the unknowable state

Written **before** the strike. Re-run by me on a quiet box.

⚠ Re-baseline the circuit on the grading box before any throughput comparison (S29).

| # | what | expected |
|---|---|---|
| 1 | ★★ **`seen-dups` moves** | with the drop on, **`seen-dups > 0`**. Any non-zero. The number 3c could not move |
| 2 | ★★ **the placement discriminates** | drop-**before**-write → `seen-dups = 0`; drop-**after**-write → `seen-dups > 0`. **Same rate, same seed, one variable.** If both cells agree, the placement was never the variable |
| 3 | ★★ **`distinct`** | **I predict `< 8000` — see below.** Whatever it is, report it |
| 4 | the seed replays | two runs, same seed → same `seen-dups` |
| 5 | rate 0 unchanged | `seen-dups=0; distinct=8000; dup=0`, floor untouched |
| 6 | the worker untouched | `git diff` shows no change at `:353-360` or `:402-419` |
| 7 | scope | `circuit.wat` only |
| 8 | the floor | `5213/5213` at the default |

## ⚠ ROW 3 — I PREDICT LOSS, AND I AM NAMING THE MECHANISM

- A claims → `First` → **ledger written** → reply dropped → A does not ack and emits `outs0`.
- Visibility expires. B receives the same message → claims → `Dup` → `first? = false` → **B emits
  nothing either.**
- **No outcome is ever emitted for that message.**

**`distinct < 8000`. Not a duplicate — a stranding.** The consumer claims *before* it emits, so a
lost claim-reply converts at-least-once delivery into at-most-once processing.

⛔ **If that happens it is the finding, not a failure.** It is a real defect in the idempotent
consumer, surfaced by the first fault that can reach it, and it is exactly why this fault domain
exists. **Report it. Do not repair it here** — a fix whose failure was never observed is the thing
this arc has spent nine stones learning not to ship.

If `distinct = 8000` instead, **something recovers the message and I cannot see what from here.**
Say what it was; that is a more interesting result than the prediction holding.

★ I state the mechanism because **every prediction this campaign that named only a number has died,
and all three that named a mechanism have held** — the store going hot under durability, the chain
amortising by K, and 3c's alarms firing between arms.

## RUNTIME PREDICTION

**60–90 minutes.** The drop is a few lines on a branch that already exists; the second placement and
the paired measurement are most of the work.

## TRAP-DOOR RISKS

1. **The two placements must share rate and seed.** Different seeds make the cells incomparable and
   row 2 proves nothing.
2. **`Seen`'s `:max-frame-bytes` is 256.** Adding `:durable` fields grows the hibernation payload,
   not the request; but if a `stats` response grows past the cap, reading the counters severs the
   connection that reads them.
3. **A dropped reply on `claim` means the worker redials `seen`** (`:406`). That path is live since
   3c-pre; if it starts asserting, the redial is failing for a different reason and that is a finding.
4. **`disrupts=24` must stay 24** if 3c's chaos is left on — if it moves, the seed threading broke.
5. **Do not confuse `dup` and `seen-dups`.** `dup` counts duplicate *outcomes* in the ledger vector;
   `seen-dups` counts absorbed *claims*. This stone should move the second, not the first.

## WHAT WOULD MAKE ME REJECT A GREEN REPORT

- Row 2 with only one placement built.
- Row 2 with the two cells run at different seeds.
- Row 3 reported as a pass because `distinct=8000` without saying what recovered the message.
- `distinct < 8000` repaired in this stone rather than reported.
- The rate or seed tuned to produce a particular `seen-dups`.
- A run that hangs.
