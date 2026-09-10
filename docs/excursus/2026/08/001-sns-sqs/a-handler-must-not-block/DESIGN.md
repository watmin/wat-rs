# DESIGN — a handler must not block

## Why

Confirmed at `c833f4a6e`, both directions, with the prediction written before the run:

```
:fanout::worker's receive wait   collect (12 workers, 3 runs)     per worker
        50 ms                     952 / 1052 /  881                80 ms
       250 ms  (shipped)         3584 / 4315 / 4259               338 ms
       500 ms                    6147 / 8711 / 8227               641 ms
```

Linear both ways. `circuit.wat:471` — inside `:fanout::worker`'s tick handler —
`Queue/receive … :wait (Wait::UpTo (Milliseconds 250))`. **While that client call is outstanding the
handler has not returned, so the service loop cannot dispatch anything else.** For up to 250 ms the
worker is deaf to `disrupts`, to `stop`, and to every other request.

One mechanism explains every cost measured today: `:queue::queue` is request-driven and never parked
(**2.8 ms**); `:probe::ctr` is idle with no poll (**0 ms**); `:fanout::worker` is parked
(**~150 ms per request**). Tick rate is irrelevant — the tick *starts* a poll, the poll blocks. Payload
is irrelevant — it is wait time, not data.

⛔ **And this is not a `collect` defect.** It is the cost of asking anything of a service parked in a
long poll. `collect` is merely where the harness asks 12 workers two questions each.

★★ **The discipline was already written down here.** The `mora` ward: *"every wait must arrive via the
wire, not via mechanism… time is I/O; it arrives as an fd-event or it doesn't arrive honestly."* The
IPC campaign multiplexed every wait in `src/comms` for exactly this reason. A `Queue::Wait::UpTo` held
inside a handler is the same defect one level up — in wat rather than Rust.

## What is already in the substrate — half of it

Read this session, so the stone does not rebuild what exists:

- **The serve loop already multiplexes.** `wat/service.wat:1564` types a selectable as
  `(Peer :- [proto::Reply service::Op])`, and `:1542` says *"client op is RE-TAGGED into its
  `<service>::Op` counterpart at the Message arm"*. **A peer's reply already arrives as one of the
  service's own ops.**
- **Mixing real peers with timers in one select set is proven feasible in-tree** —
  `wat-scripts/scratch-pad/probe-self-scheduling-loop.wat` is the disconfirming probe for exactly that
  shape and cites `arc-292` for the mix.
- **Alarms already let a handler schedule its own future wake** without blocking.

## ⛔ What is missing — the outbound half

`:wat::service::Directed` is `[conn-id, reply]` — a **reply to a parked requester**. `Outcome::Continue`
carries `state`, an optional `reply`, `sends` (Directed), and `arms` (Alarm). **Nothing in that set
issues an outbound REQUEST to a peer.** A handler's only way to ask a peer anything is a synchronous
client call, which is precisely what blocks.

★ So B′ reduces to one question: **can a handler issue a request and return, with the reply arriving as
an op — using what exists, or does the outcome need a fourth affordance?**

## What this stone delivers

**`:fanout::worker` stops blocking inside its handler** — by whatever the substrate already affords.

⛔ **And it is deliberately scoped as a FEASIBILITY strike, not a committed substrate change.** Two
outcomes are both complete:

1. **The affordance exists** (via `selectables`, or a form of `Directed`, or a peer registered so its
   replies arrive as ops) → the worker is rewritten to use it, and `collect` should fall toward the
   50 ms row's ~80 ms/worker **without** converting the long poll into busy-polling.
2. **The affordance does not exist** → STOP and report *precisely what is missing from the outcome
   surface*. That is the arc-shaped finding, and opening it is the builder's ruling. **Reporting the
   gap is the deliverable; inventing the substrate is not.**

## The one contract decision

⛔ **The long poll must not become a busy poll.** `Wait::Immediate` measured acceptably on this
workload (correctness held, `store-calls` +1.5 %) but it is a **workaround that fails the Honest
question**: it makes one worker responsive, leaves the defect standing everywhere else, and its cost
depends on a workload we happen to run. **A service that idles long would busy-poll.** So the target is
*non-blocking with push*, not *non-blocking by polling harder*.

Measured guard: `store-calls` must not rise materially, and `drain` must not regress.

## Out of scope = rejected

- **Generalising to every `defservice`.** This stone changes one service and reports what the change
  needed. Rolling it out is the follow-on.
- **`:fanout::held-worker`'s 50 ms** — a sibling with an undocumented 5× different wait (`:943`). Named,
  not touched; the third such constant found today.
- `fill`'s 2046 ms of non-work and the never-swept `p`; the poll loop's 95 % share of stats traffic;
  tier-1 fault injection; the seven chaos tests' assertions.
- `setup`. Cold boot, ruled out.

## ⚠ What must not be claimed

⚠ The `collect` improvement is a **consequence**, not the point. The point is that a service stops
being deaf. A SCORE that reports only the milliseconds has measured the smaller half.
⚠ And this does not fix any other service. `:fanout::worker` is one site; the finding is that the
*shape* is available to every service that calls a peer from a handler.
