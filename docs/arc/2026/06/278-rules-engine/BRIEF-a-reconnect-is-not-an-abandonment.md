# BRIEF — a reconnect is not an abandonment

Make `Lost` and `Closed` redial **and retry** instead of redialing and discarding the work, in all
three retry ladders. Add `ack-exhausted` so the ack path counts abandonments like its siblings.
`wat-scripts/fanout/circuit.wat` only.

A **correctness fix**.

## Read in order

1. **`DESIGN-a-reconnect-is-not-an-abandonment.md`** — the evidence: 40 stranded at n=2000 with
   `ack-limit-ms = 1 000 000`, so deadline exhaustion is impossible and `Lost`/`Closed` is the only
   path that abandons.
2. **`circuit.wat:709-712`** — the **ack** ladder's `Lost` / `Closed` arms:
   `(Tuple (QueueRetry::Exhausted 0) (redial-q) 0)`.
3. **`circuit.wat:532-534`** — `seen-until`'s **check/mark** arms, identical shape with
   `SeenRetry::Exhausted 0` and `(redial)`.
4. **`circuit.wat:713-725`** — the `DeadlineFired` arm. **This is the retry shape to reuse**: check
   the elapsed bound, draw a backoff, `await-ms`, recurse with the redialed peer.
5. **`circuit.wat:250-251`** — `check-exhausted` / `mark-exhausted` / `ack-retries` on
   `worker::Record`. **`ack-exhausted` joins them.**
6. **`circuit.wat:930`** — `"redial failed — peer is dead, not a broken pipe"`. The substrate already
   separates the two cases; a successful redial means the peer is alive.

## The work

**1. `Lost` and `Closed` retry.** In all three ladders, those arms redial and then take the **same
path the `DeadlineFired` arm takes** — bound check, backoff, recurse — rather than returning
`Exhausted`. The retry counter increments, as it does for a deadline retry.

**2. `Exhausted` becomes reachable only from the time bound.** After this, no arm produces
`Exhausted` for a transport reason.

**3. `ack-exhausted`** on `worker::Record`, threaded exactly as `check-exhausted` and
`mark-exhausted` are, and surfaced in the summary, the phases line, **and the failure-path string**.

## Blast radius

`wat-scripts/fanout/circuit.wat` **only** — the `:fanout::worker` service. No `sqs.wat`, no `wat/`,
no `StatsResponse` — **no ripple.**

⚠ The `PersistentMap` change from the previous stone is **in the tree, uncommitted, and correct**
(n=500/1000 green, `distinct`/`dup` exact). **Build on it; do not revert it.**

## STOP triggers

- **STOP-1** — if a `Lost`/`Closed` retry can loop without bound, **STOP.** It must ride the same
  elapsed bound as `DeadlineFired`; a reconnect loop with no bound is worse than the bug.
- **STOP-2** — if `distinct` or `dup` changes at any depth, **STOP.** This is a correctness fix and
  must not alter delivery counts.
- **STOP-3** — floor red on any arm: **STOP; do not re-run.** Name the exact arm.
- **STOP-4** — **do not change `vis`.** It is separately illogical at 1000 s on a 40 s run and is its
  own stone; changing it here would **mask** this fix by letting stranded messages reappear.
- **STOP-5** — **do not touch `:2075` or `wat/`.**
- **STOP-6** — **leave the tree parsing.**
- **STOP-7** — if n=2000 still strands, **STOP and report `ack-exhausted`, `check-exhausted`,
  `mark-exhausted` and the terminal sweep.** With the new counter the failure can finally name
  itself; that report is the result.
