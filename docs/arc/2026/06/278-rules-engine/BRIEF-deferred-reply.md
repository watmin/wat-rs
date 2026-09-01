# BRIEF — the deferred reply (`Outcome::ReplyTo`)

Let a `defservice` arm reply to a client **other than the one that invoked it**, named by the
`conn-id` it already receives in `ctx`. This is the substrate half of long polling; the queue is a
separate stone.

Read `DESIGN-STONE-the-deferred-reply.md` beside this first — the contract decision (a vector,
addressed by conn-id, never a peer) and one assertion that must change or the stone is inert.

## Read in order, and why you are being sent there

1. **`wat/service.wat:1523`** — `:conn-id (:wat::core::first (:wat::core::nth selectables idx))`.
   **This is the whole feasibility argument**: ctx's `conn-id` IS the id stored beside the peer in
   the serve loop's tuple. The loop can resolve a name back to a peer because the id travels with it
   from birth.
2. **`wat/service.wat:59-63`** — the `Outcome` enum, and `Alarm` above it. `NoReplyAndArm`'s
   `arms <- (Vector :- [Alarm])` is the shape you are mirroring: an outcome carrying a vector of
   things for the loop to do.
3. **`wat/service.wat:1591-1607`** — the INTERNAL op arm, including the assertion that an internal op
   *"has no client to reply to"*. **That assertion must stop catching `ReplyTo`** — a fired timer has
   no invoking client, which is precisely why it must be able to name one.
4. **`wat/service.wat:1670-1700`** — the surface arm's `Outcome` match, and how each arm faces
   `SendOutcome` (*"client gone → keep serving"*). Your resolution+send follows that doctrine.

## The work

**1. `Directed {conn-id <- i64, reply <- :R}`** and **`Outcome::ReplyTo [state, sends <- (Vector :- [(Directed :- [R])])]`**.

**2. Resolution in the serve loop**: for each `Directed`, find the peer whose id is that `conn-id` in
`selectables` and send. A conn-id that is absent, or a send that fails, **keeps serving** — a fact
about one client, not the world.

**3. Legal from BOTH arm kinds.** Surface arms and internal arms. The internal arm's existing
assertion keeps catching `Reply`/`ReplyAndArm` and must let `ReplyTo` through.

**4. A gate service** that proves the whole shape: op `park` stores its `ctx`'s conn-id and returns
`NoReply`; op `wake` returns `ReplyTo` naming the parked conn-id. Two clients: one parks (blocks in
`recv'`), the other wakes it, and the first returns with the value the second sent.

## Blast radius

`wat/service.wat` (the `Outcome` enum, `Directed`, the two arm kinds, the resolution) and the gate.
**No runtime change. No change to any existing service.** Every current `Outcome` keeps its meaning.

## STOP triggers

**STOP-1 — an arm must never receive or hold a `Peer`.** Address by `conn-id`; the loop resolves.
Handing a caller's peer to user code is the lifetime hazard excursus 002 spent three stones walling
off, and `:durable`/`:ephemeral` have no honest home for one.

**STOP-2 — the internal-op assertion must let `ReplyTo` through.** If a fired timer cannot
`ReplyTo`, the stone is inert and **every other gate still passes**. This is the one that fails
quietly.

**STOP-3 — a vanished waiter is not an error.** Absent conn-id, or a failed send: keep serving.
Do not raise, and do not silently desync the service's own state — the arm removes the waiter as it
returns the outcome, so a failed send leaves a forgotten waiter and a gone client, which is true.

**STOP-4 — every existing `Outcome` keeps its exact meaning.** This adds a variant; it does not
adjust `Reply`, `NoReply`, or the arm/timer machinery. If an existing service's behaviour moves,
STOP.

## The gates to write

- **★ a parked client is woken by another client's call** — the parked `recv'` returns the value the
  waker supplied. **RED today: there is no way to express it.**
- **★ a parked client is woken by a TIMER** — an internal arm returning `ReplyTo`. This is the one
  STOP-2 protects, and long polling is impossible without it.
- **several waiters woken by one call** — the vector, doing what a single-reply variant could not.
- **a vanished waiter** — park, drop the client, then wake: the service keeps serving.
- **nothing else moved** — the full floor, and no existing service touched.

## Prior comparable result

`SCORE-impls-completeness-guard.md` — a no-delta stone, and its note on why: the load-bearing
decisions were made against measurements taken first, and where a measurement could not be trusted
the brief said so.
