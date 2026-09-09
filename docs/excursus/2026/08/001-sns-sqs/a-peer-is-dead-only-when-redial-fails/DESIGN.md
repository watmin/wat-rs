# DESIGN — a peer is dead only when redial fails

**Stone 3c-pre.** The gate in front of chaos. Small, uniform, and the probe is already green.

## WHY

`probe-frame-cap-severs-one-conn.wat` established the mechanism on 2026-09-03:

```
a-small=ok ; a-big=lost ; b-other=ok ; a-again=closed
```

**The FIRST failure on a severed connection is `Lost`. Every SUBSEQUENT touch is `Closed`.** And
B's separate connection kept working throughout — **the service was alive the whole time.**

The corpus reads that `Closed` as death:

```wat
(:wat::kernel::RecvOutcome::Closed
  (:wat::kernel::assertion-failed! "fanout worker: ack closed" …))
```

Measured across the three circuit files, **by arm, not by token**:

| file | `Closed` fatal | `Lost` fatal |
|---|---|---|
| `sns-fanout.wat` | **6/7** | **1/7** |
| `circuit.wat` | **6/7** | **1/7** |
| `sqs.wat` | **5/5** | **1/6** |

**17 of 19 `Closed` arms die; 3 of 20 `Lost` arms die.** The same fault, one step later, gets the
opposite disposition — and nobody chose that. It happened because nothing in the tree had ever
severed a connection, so only the first-touch path was ever designed.

★ **This is the gate in front of chaos.** A client-side drop severs a connection. The peer's next
touch returns `Closed`. Today that kills the worker. **Injecting drops now would measure the absence
of our error handling rather than its behaviour** — 58 written failure arms, and the chaos would
never reach past the first one.

## THE PROBE IS ALREADY GREEN

`wat-scripts/scratch-pad/probe-closed-is-recoverable.wat`, 3/3, committed:

```
a-small=ok ; a-big=lost ; a-again=closed ; a-REDIAL=ok ; b-still=ok
verdict = CLOSED-IS-RECOVERABLE
```

After the sever, and after the stale handle has returned `Closed`, **re-dialing the same address
produces a working connection.** The recovery path is reachable from `Closed`, with no substrate
change. The executor is not discovering whether this works.

## ⛔ THE ONE CONTRACT DECISION

**A peer is dead only when redial fails.**

Not when an outcome variant says `Closed`. The probe proves why: the service answered B while A's
handle was returning `Closed`. **A connection is not a service**, and no connection-level outcome can
report on a service's liveness. The only instrument that can is a redial — which is exactly what the
`Lost` arms already do.

That is a *derived* cannot, not a convention: it follows from what the two things are.

## WHAT IT DELIVERS

`Closed` joins `Lost` on the recovery path. The exemplar is already in the tree —
**`circuit.wat:224-232`**, verbatim:

```wat
((:wat::kernel::RecvOutcome::Lost _cause)
  ;; Do not ack. If the claim landed, vis + Dup absorb.
  (:wat::core::Tuple q0
    (:wat::core::match (:wat::kernel::connect (:fanout::worker::Record/seen-addr rec))
      ((:wat::kernel::ConnectOutcome::Connected p) p)
      (_ (:wat::kernel::assertion-failed!
           "fanout worker: redial seen failed — peer is dead, not a broken pipe" …)))
    outs0))
```

Every fatal `Closed` arm in the three files becomes that shape, against its own address. The
assertion survives — **one rung down**, where it is now true: *redial failed, therefore dead.*

### `Stopped` does NOT change

`Stopped` means *the substrate was asked to stop* — the peer was alive and the channel open, and we
are shutting down. Redialing into a shutdown is wrong. It stays as it is.

★ It is *also* true that 16 `Stopped` arms assert where a clean exit would be more honest. **That is
a different reason to change** — shutdown hygiene, not fault recovery — and mixing it in would braid
two stones. **S26.**

## FILES

`wat-scripts/topic/sns-fanout.wat` · `wat-scripts/fanout/circuit.wat` · `wat-scripts/queue/sqs.wat`

**17 fatal `Closed` arms.** Each needs its own address in scope to redial — that is the per-site
work, and it is why this is not a codemod.

★ **This census is mine, taken by arm rather than by bare token after five census errors this
campaign — the last of which would have swept a latency histogram into a queue rename. Treat it as a
hypothesis; the count you find is the fact.**

## OUT OF SCOPE = REJECTED

- **`wat/service.wat`'s own arms** (4/7 `Closed` fatal). That is the reactor's handling of *its*
  peers — the serve loop's job, a different layer, and stdlib (BOOTSTRAP). Chaos needs the
  **clients** to survive. **S27** if the reactor turns out to need it too.
- **`Stopped` → clean exit.** **S26**, above.
- **The drop injection itself.** That is 3c. This is the gate that makes 3c legible.
- **Retry limits / backoff.** A redial that keeps failing already asserts. Bounding *how many times*
  we redial is a policy question chaos will inform — not a guess to make first.

## THE PROOF

1. **★ A severed worker recovers and the invariant holds.** Sever a live worker's connection
   mid-run; the worker must reconnect and the run must still finish `distinct=8000; dup=0`. This is
   the row that matters — a unit probe proves redial works, this proves *the system* survives it.
2. **The assertion still fires when it should.** Point a redial at a genuinely dead service and show
   `"peer is dead, not a broken pipe"` still fires. **A wall that never fires is a deleted wall** —
   if `Closed` can no longer kill anything, the recovery is unfalsifiable.
3. **No `Stopped` arm moved.** `git diff` shows `Stopped` untouched.
4. **The circuit**, five runs, `dup=0`, publish inside band.
5. **The floor**, Summary line, `5213/5213`.
