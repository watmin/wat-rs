# Arc 301 — SNS and SQS

**Status: DRAWN 2026-08-30.** Stone 1 (SNS) is STRUCK in userland and needs no substrate
change. Stone 2 (SQS) is blocked on ONE substrate decision, stated below and not taken.

> Builder: *"we have been wanting to build something like an sns and sqs… let's build sns in
> userland… then we build what we must for sqs"*

## What already exists — the reason this arc is small

| AWS | wat, today | evidence |
|---|---|---|
| DynamoDB | `:wat::query::Store` — `pk`/`sk`, GSIs, atomic batch `put` | `wat/query.wat:497` |
| — two backends | `wat/query/mem.wat` (169 lines), `wat/query/sqlite-store.wat` (336) | both `:satisfies :wat::query::Store` |
| CloudWatch | `:wat::telemetry::journal` — a service HOLDING a Store peer | `wat/telemetry/journal.wat:62` |
| the wire | `defsurface :nature :wat::kernel::Peer` + `defservice`, thread or process | `wat-scripts/probes/arc-278/s2s-{thread,process}-probe.wat` |
| timers | `:wat::kernel::after` | `wat-tests/timer-*.wat` |

So neither stone builds a transport, a store, or a serializer. Both are compositions.

## Stone 1 — SNS, in userland. STRUCK.

`wat-scripts/topic/sns-fanout.wat` — one topic, three subscribers, **one file that runs
BOTH loci and prints both counts**. Prints `"3 3"`. The locus is a parameter, so the
differential is the artifact rather than a thing a reader must remember to run twice.

Established by bisecting UP from the green `s2s-process-probe.wat`, one variable at a time
(seven steps, each green): a `(Vector :- [Peer])` ephemeral field · dialing through it · a
`let`-wrapped grant · a `(Vector :- [Address])` as a `/start` kwarg across a fork · two child
services · `:init` folding and dialing all of them · `publish` fanning out to all of them.

Two facts the bisect pinned that are NOT written down elsewhere:

1. **A subscriber's birth-seed allow-set holds only `getppid()`.** The topic is a *stranger*
   to a subscriber it did not spawn, and is bounced until granted. On the process locus every
   subscriber must `grant` the topic's pid, via `process/post-spawn` — which fires owner-side
   with the child's `ProcessLaunch{pid}` after the fork and before `:init` ships. This is the
   grant-before-dial ordering. See `tests/services/probe_arc170_m1_teeth_admitted.wat`
   ("served ONLY because we granted it") and `probe_arc209_c0b3bb_bounced.wat`.
2. **A forked child's bundle does not carry the program's other `defn`s.** An `:init` that
   calls a top-level helper dies at StartupError with `UnresolvedReference`. The dial must be
   written inline in `:init`. Measured 2026-08-30.

### The one wart — `bijection-anchor`, and why it is NOT drawn as a stone

`defservice` enforces a bijection between `:peers` and ROOT ephemeral peer fields
(`wat/service.wat:857–891`), and the derivation that finds those fields reads only the
**top-level type head** (`wat/service.wat:824`) — so a `(Vector :- [Peer])` is invisible to it.
Declaring `:peers [:demo::Sub]` with only the vector is REFUSED; omitting `:peers` passes the
check but stops shipping the subscriber's `surface-forms` into the forked child
(`wat/service.wat:792`, `:2523`), which kills the child on the process locus.

The demo therefore carries a root scalar peer field held only to satisfy the bijection. It is
never published to.

**This was initially reported as a blocker. It is not.** The anchor works on both loci, so
making the derivation look inside container types would DELETE AN UGLINESS, not unblock
anything. It is recorded here as a candidate, deliberately not drawn, so the case for it stays
honest: the fix is small and the motivation is cosmetic.

## Stone 2 — SQS. BLOCKED ON ONE RULING.

### The storage model, which is why this is small

Journal already proves the trick at `:wat::telemetry::time-sk`: a **constant-width** `#inst`
sorts lexicographically *and* chronologically, so `sk` can be a time and `scan`'s range order
is time order.

- `pk` = the queue name
- `sk` = the message's **visible-at** timestamp
- `send` → `put` a row at `sk = now`
- `receive` → `scan` pk where `sk <= now`, take N, then `put` those rows back at
  `sk = now + visibility-timeout`

**The visibility timeout is a re-put that moves the sort key into the future.** No lock, no
timer, no side state; redelivery is simply what happens when nobody moved it again. `put` is
already atomic-batch, so a receive of N is one transaction.

Then `ack` must remove the message. And:

### `:wat::query::Store` has no `delete`

Its `:features` block (`wat/query.wat:551–569`) is exactly four: `ensure-schema`, `put`,
`scan`, `scan-index`. Append and read. **SQS's ack is not expressible on it.**

### The ruling — taken by the builder, not here

**(a) Add `delete` to the Store surface.** It mirrors `put` exactly:
`DeleteRequest [keys <- (Vector :- [Key])]` where `Key` is `(pk, sk)`; `DeleteResponse` reuses
the shared `:Success/:Constraint/:Transient/:Fatal/:RequestTooLarge/:RequestMalformed`
vocabulary; atomic batch, one transaction, same as `put`. Two satisfiers to update, both short
and symmetric — `mem-store` filters the `PersistentVector` (mirroring its `put` foldl);
`sqlite-store` is `begin → delete-rows → commit` with a `:wat::query::delete-rows` helper
mirroring `put-rows`.

**(b) Tombstones.** Ack writes a delete-marker row; `scan` filters. Zero substrate change, and
expressible today. But every scan pays forever and the queue never shrinks.

**Recommendation: (a).** The gap is not SQS-shaped. A keyed store that cannot delete is
incomplete for anything that is not a log, and the next consumer hits it too. (b) buys nothing
except deferral, and it makes the *queue* — the one structure whose whole job is to drain —
the thing that cannot shrink.

## Out of scope, affirmatively cut

- **Networking.** Threads and processes are the two loci; unix sockets / TCP are not this arc.
- **`:peers` seeing through containers.** Named above, deliberately not drawn.
- **Dynamic subscribe** (a subscriber `attach`ing at runtime via an `Address` in a message
  payload, rather than at `:init`). UNTESTED — two unknowns: whether an `Address` survives as
  a message *field*, and whether a service can grow `:ephemeral` peer state mid-life. The demo
  wires subscribers at `:init`, which is static-but-N-ary, and this doc does not claim
  otherwise.
