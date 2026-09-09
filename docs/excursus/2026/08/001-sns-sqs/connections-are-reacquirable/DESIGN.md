# DESIGN — connections are re-acquirable

Drawn 2026-09-03. **Not struck.** First stone of the chaos work — and a **precondition** for it.

## Why

Every service takes an `Address` at `:init`, dials it once, keeps the `Peer`, and **discards the
address**:

```wat
:durable   [nsubs]      / [vis-ns] / []
:ephemeral [inbox <- (Peer :- [Queue::Op Queue::Reply])]
```

So there is nothing to re-dial. And accordingly, **all 20 `RecvOutcome::Lost` arms across
`circuit.wat`, `sns-fanout.wat` and `sqs.wat` are `assertion-failed!`** — a lost connection kills
the service.

★ **This inverts the sequencing.** Injecting packet loss today would teach us nothing about
at-least-once; every service would simply die on the first dropped frame. **Recovery must exist
before chaos can measure anything.**

## The fix is the substrate's own doctrine, currently violated

`:durable` is *"the soul: EDN, crosses the wire, survives hibernation."* `:ephemeral` is *"the body:
resources and peer clients, never crosses."*

**An `Address` is a soul. A `Peer` is a body.** We keep the body and throw the soul away — backwards
for anything that must outlive its own connection.

## PROVEN, not assumed — `probe-redial-from-durable-addr.wat`

No exemplar existed: **nothing in `wat/` or `wat-scripts/` holds an `Address` in `:durable`.** Three
things were unproven and all three were probed before this was drawn:

```
durable-addr=ok ; before=yes ; redial=yes ; after=yes
```

1. `:durable` accepts an `(Address :- [Op Reply])`.
2. An **arm** can re-dial from it and store the fresh `Peer` in returned state — the excursus-002
   handle-lifetime wall does **not** reject a `Peer` created outside `:init`.
3. The re-dialed peer works.

**No substrate change is needed.** The design was expressible all along; nobody wrote it.

## The one contract decision: recovery is reconnect + DO NOT ACK

On `Lost`, an arm does **not** re-issue the in-flight request itself. It reaps the dead peer,
re-dials from the durable address, and **declines to ack**. The row's visibility then expires and the
work returns through the path that already exists.

This matters because the in-flight request is **unknowable** — it may or may not have arrived. The
only honest response is to let it be retried and have the receiver absorb a possible duplicate,
**which S13 built yesterday**. So the pieces compose with nothing new downstream:

```
pipe breaks -> reap -> re-dial from durable address -> do not ack
            -> visibility expiry returns the row -> :fanout::Seen absorbs it if it landed
```

## Scope boundary: a broken pipe, NOT a dead peer

This stone covers **the connection failing while the peer lives** — which is exactly what packet
loss produces, and exactly what the chaos arc needs.

It does **not** cover the peer being gone. A stopped service's address is dead; re-dialing it will
fail forever. Recovering from that needs supervision and restart, and a restarted service has a
**new** address — so it needs address *rediscovery*, not a stored one. **Different fault, different
stone, named here so it is not conflated.**

## Out of scope = REJECTED

- **Supervision / restart / address rediscovery.** Above.
- **In-arm retry of the in-flight request.** The contract decision; visibility expiry is the retry.
- **A reconnect attempt bound or backoff.** Reconnect to a live peer succeeds or the peer is dead,
  which is the other fault. Do not invent a counter — S13's stone rejected one for the same reason.
- **`wat/`, `src/`.** The probe proves neither is needed.
