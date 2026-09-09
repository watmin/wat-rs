# DESIGN STONE — the deferred reply (`Outcome::ReplyTo`)

**Commissioned 2026-09-01.** The substrate half of long polling. Drawn because the measurement said
the circuit's cost is its **round-trip count**, not its speed — and long polling is how a queue stops
paying a network hop to be told "nothing yet".

## Why this, and why now

At process locus a service call costs **154 µs** — measured, and at the fast end of a realistic
network hop (an intra-datacenter RPC is 0.1–1 ms). We are modelling a distributed system with IPC
standing in for the wire, so that is the right price and it will only rise when the transport becomes
TCP. **The program does not change; the transport does.**

So the circuit's 88 s is a round-trip *budget*, not a speed problem. `circuit.wat`'s worker asks
`:limit 1` and its fold runs `cap` times regardless, so **every empty poll costs a full network
round-trip**. Real SQS has `WaitTimeSeconds` (up to 20 s) for exactly this reason.

## Two of the three pieces already exist

- **Waiting** — `poll'` is a real multiplexer over the selectable set, and arc-292 made a timer a
  peer, so a deadline is just another selectable. Process tier is timerfd-backed. `mora`-honest.
- **The deadline** — `Alarm` + `NoReplyAndArm`. Green at both loci as of this morning; the queue
  could not have had long polling before today.
- **Addressing the waiter** — `service.wat:1523`:
  `:conn-id (:wat::core::first (:wat::core::nth selectables idx))`. ctx's `conn-id` IS the id stored
  beside the peer in the serve loop's tuple, and the doc says why it exists (`:93`, `:99`): *"the
  stable monotonic i64 minted in the serve loop (**never reused**) … the name that **outlives the
  round**."*

## The gap

Every `Outcome` replies to **the client that invoked this arm**. There is no way to say *"reply to
conn-id 7."* A name that outlives the round has nothing that can use it after the round.

## ★ THE CONTRACT DECISION: a vector of directed replies, addressed by conn-id — never a peer

```
Directed {conn-id <- i64, reply <- :R}
Outcome::ReplyTo [state <- :S, sends <- (Vector :- [(Directed :- [R])])]
```

**A vector, not one**, mirroring `NoReplyAndArm`'s `arms <- (Vector :- [Alarm])`: one arriving batch
may satisfy several waiters, and an outcome that could only wake one would force a queue to wake them
one message at a time — the round-trip tax this whole stone exists to remove.

**By `conn-id`, never by peer.** An arm must not receive or hold a live `Peer`: `:durable` is the
soul and crosses the wire, `:ephemeral` is the body — a caller's peer belongs to neither, and handing
one to user code is the lifetime hazard excursus 002 spent three stones walling off. The serve loop
holds the `(conn-id, peer)` pairs; it does the resolution. The arm names, the loop sends.

## ★ The internal-op assertion must change, and this is the subtle part

Today an internal (`-`) arm that returns `Reply` hits a located assertion:

> *"defservice: an internal (-) op returned Outcome::Reply, but an internal op has no client to reply
> to (return NoReply / NoReplyAndArm)"*

That is **true of `Reply` and false of `ReplyTo`.** A fired deadline timer has no *invoking* client —
which is exactly why it must be able to name one. `ReplyTo` from an internal arm is the whole point:
the timer wakes the waiter it was armed for.

So the assertion stays for `Reply`/`ReplyAndArm` and must **not** catch `ReplyTo`. Getting this wrong
makes the stone useless while every other gate still passes.

## A waiter that vanished is not an error

A long-polling client can give up. When a `conn-id` is no longer in the set, or its send fails, the
serve loop **keeps serving** — the same doctrine the existing reply arms already apply (*"client gone
→ keep serving"*). It is a fact about one client, not about the world.

The service's own bookkeeping stays consistent because the arm removes the waiter from its state as
it returns the `ReplyTo`: the send is the last step, so a failed send leaves a forgotten waiter and a
gone client, which is the truth.

## The consumer that justifies it (its own stone, after this one)

```
receive, none available, wait > 0  → store the waiter, NoReplyAndArm(deadline)
send arrives                       → ReplyTo the waiting conn-ids
deadline fires                     → ReplyTo empty
```

`ReceiveRequest` gains a wait duration; **`wait = 0` must be byte-identical to today**, so every
existing queue gate passes untouched. `limit` already exists and is already honoured — including
`limit 1`, which stays valid for a client that wants exactly one. The circuit then asks for more than
one, and its round-trip count falls by that factor.

## Out of scope = REJECTED

- Long polling itself, and the circuit's batching. This is the substrate; they are the consumer.
- Handing an arm a `Peer`. See the contract decision.
- Fairness policy among waiters (FIFO vs other). Pick the obvious one, document it, and let a second
  consumer argue for more.
