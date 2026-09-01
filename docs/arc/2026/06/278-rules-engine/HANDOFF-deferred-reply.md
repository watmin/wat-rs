# HANDOFF — the deferred reply (`Outcome::ReplyTo`)

You are letting a `defservice` arm reply to a client **other than the one that invoked it**, named by
the `conn-id` it already gets in `ctx`. This is the substrate half of long polling.

Start here, in order:

1. `DESIGN-STONE-the-deferred-reply.md` — why now (measured: the circuit's cost is its round-trip
   count, and a service call is already at a realistic network price), and the contract decision.
2. `BRIEF-deferred-reply.md` — the rooms as exact `file:line`, four STOP triggers.
3. `wat/service.wat:1523` — the feasibility argument in one line: ctx's `conn-id` IS the id stored
   beside the peer in the serve loop's tuple. The loop can already resolve a name to a peer.

Three things to hold:

**Address by `conn-id`, never by peer.** An arm must not receive or hold a live `Peer` — `:durable`
crosses the wire and `:ephemeral` is the body; a caller's peer belongs to neither, and handing one to
user code is the lifetime hazard excursus 002 spent three stones walling off. The arm names; the loop
resolves and sends.

**★ The internal-op assertion must let `ReplyTo` through.** Today an internal (`-`) arm returning
`Reply` hits a located assertion saying it *"has no client to reply to"*. That is true of `Reply` and
**false of `ReplyTo`**: a fired deadline timer has no *invoking* client, which is exactly why it must
be able to name one. If you leave that assertion catching `ReplyTo`, the stone compiles, the floor is
green, every other gate passes — and long polling is still impossible. That is the failure mode here.

**A vanished waiter is not an error.** A long-polling client may give up. An absent conn-id or a
failed send keeps the loop serving, following the doctrine already in the reply arms ("client gone →
keep serving"). Have the arm remove the waiter as it returns the outcome, so a failed send leaves a
forgotten waiter and a gone client — which is the truth.

Do not use a sleep in the gate. Park/wake is ordered by the wire: A blocks in `recv'` until B's call
causes the send. If a gate needs a sleep, the mechanism is not doing what it claims.

The floor is `./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a
red — do not re-run, name the arm, surface it.

Write `SCORE-deferred-reply.md` when done. It will be graded by re-running.
