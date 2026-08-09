# DESIGN-STONE — THE CALL CONTEXT: a handler is told WHO is calling, and it is pure data

> **DERIVED with the builder 2026-08-09**, out of the connection-scoped-world crawl. His words:
>
> *"whoa - we just derived the need to modify the service handlers…… we need it to be [self ctx req]
> or similar…. ctx is service defined and passed to all calls along with the user's input?"*
>
> *"ctx is a minimally defined record that users can extend…. maybe… like Ruby's rack…. the env call
> pattern…. but like… ctx can have at minimum a request-id set to a uuid, the caller's identity….
> maybe something like…. start nanoseconds of request."*
>
> *"uuid-v4's value is pure (its just a large int) - the func to generate a v4 is not. and a timestamp
> is pure (its just a large int) - the func to generate a time is not."*

## Why this exists — the gap that forced it

Building the connection-scoped world (`DESIGN-STONE-the-connection-scoped-world.md`) stopped dead on a
grounded fact: **a `defservice` op handler cannot know which client is calling.** The generated arm
binds `[s-binder state]` and nothing else (`wat/service.wat:1031`). `idx` exists in the generated
frame — it is used for the reply `send` — but is never passed in. `Outcome<S,R,O>`'s five variants all
thread `S`; there is no per-caller concept anywhere in the handler contract.

So a multi-tenant service cannot select the caller's tenant state **even if that state exists**. No
map, no cursor, no per-connection anything is reachable from a handler. This stone is the missing
argument.

## ★ WE ALREADY TOOK HALF OF THIS MODEL — the other half is what we dropped

`wat/service.wat:44` says it in its own words: `Outcome` *"is OTP gen_server's
`{reply,R,S} | {noreply,S} | {stop,…}` re-derived as a wat tagged sum."* OTP's signature is:

```erlang
handle_call(Request, From, State) -> {reply, Reply, State} | …
                    ^^^^ the caller — WE DROPPED THIS
```

We adopted the **return type** and omitted the **caller argument**. This is not a new idea being
imported; it is a half-adopted model being completed. (R19 `RATIONE NON MIRACVLO` again — the builder
reasoned to `From` without holding the name.)

## The Rack analogy — which half transfers, and where it breaks

**Transfers:** one context, built per call, handed to every handler, uniform shape. That is the
ergonomic intuition and it is right.

**Does NOT transfer — and this is the load-bearing caution:** Rack's `env` is an open `Hash`, and its
extensibility works *because of the middleware stack* — `Rack::Session` puts `rack.session` in, auth
puts the user in, each layer adding keys on the way down. **We have no middleware chain.** So "users
can extend ctx" runs straight into *extended by whom, at what moment?* A field the substrate cannot
fill and no layer adds is dead weight.

An open string-keyed bag is also precisely what arcs 293/296 spent themselves removing.

## The floor — and every field of it is PURE

The builder's correction is the design's spine and it is grounded: **a value does not inherit its
generator's classification.** `is_pure_type`'s well-known-pure-scalar arm lists `wat::core::Uuid`
beside `i64` and `String` (`src/check.rs`). A `Uuid` is a large int; a `time-ns` is an `i64`.
`Uuid/v4` is *entropic* (pure ∧ non-deterministic — arc 299's third axis); the **value it returns has
no such property**.

Proposed floor, minimal, each with a named consumer:

| field | why it is here |
|---|---|
| **request id** (`Uuid`) | correlation — see the `Scope` convergence below |
| **caller identity** | THE reason this stone exists: selects the tenant's per-connection entry |
| **start nanos** (`i64`) | request-duration measurement without a second clock read at entry |

**Nothing else until something asks.** A context that accretes fields nobody reads becomes the next
hand-list.

### ⇒ CONSEQUENCE: ctx is a PURE RECORD, and that buys three things

Because the floor is pure data, the whole record is EDN — which the impure reading would have
forbidden:

1. **ctx crosses the wire** → a service calling a service can forward its caller's request id
   downstream. That is distributed tracing, falling out rather than needing a parallel mechanism.
2. **ctx may live in `:durable`** → connection-scoped facts survive hibernate/resume.
3. **ctx can BE a key** → comparable, hashable, round-trippable, so the request id can be the
   correlation key the journal indexes on, not a copy of one.

**The one constraint that survives** is a rule about the *minting site*, not a taint on the type:
**ctx is produced at an impure boundary (the serve loop) and consumed by a pure handler.** A handler
cannot mint one, because it cannot call the generators — which handler purity already enforces. Same
shape as every `Metric` and `Log` we emit today.

## ★★ THE `Scope` CONVERGENCE — and it needs a RULING

`wat/telemetry.wat:73` already defines `:wat::telemetry::Scope`, spliced into `Metric`/`Log` via
`~@:wat::telemetry::Scope`, carrying: **namespace (facility), uuid (correlation id), tags
(dimensions), time-ns (event time)** — described in its own design note as *"a UNIT-OF-WORK's
CORRELATED records"*.

That is the builder's ctx list, already minted and shipped.

**And a correlation id only earns its name if the logs emitted during a request carry the request's
id.** If ctx mints one uuid and `Scope` mints another, we hold two ids for one unit of work and
correlate nothing.

> **⛔ RULING OWED (the builder's):** is ctx **the same object as `Scope`**, does ctx **splice**
> `Scope`, or does `Scope` splice **ctx**? They are not independent and must not be designed apart.
> This is the fork most likely to produce two ids for one request if left unruled.

## Extension — surface-splice, not a bag

The wat-native answer exists and is shipped: `defsurface` + `~@Surface` splice. `Scope` → `Metric`/
`Log` is the worked exemplar (`wat/telemetry.wat:84`, `:93`): spliced fields inline first, then the
record's own, with accessors minted free by the unified aggregate constructor. Structural
satisfaction, typed, checked, no open hash, no re-listing (derive-is-the-wall).

**So the TYPE side of "users can extend it" is solved and proven.** The open side is population:

- the **substrate** fills the floor (it knows the caller, the clock, and how to mint a uuid);
- anything richer — tenant, auth principal, per-tenant limits — is established **once, at connect**,
  not per call.

**Which is the per-connection map.** The connection's spec holds the tenant-level facts; the serve
loop merges them into each call's ctx beside the per-call ones. **That is the wat-native Rack:
connection-scoped facts + per-call facts, merged by generated code — no middleware, because the
connection IS the layer that accumulated the context.** ctx and the connection-scoped world are one
design; neither is complete alone.

## ⚠ THE CALLERLESS CALL — a handler can fire with no client, and the identity would LIE

Two grounded facts:

1. **Internal (`-`) ops are already callerless and the substrate models it STRUCTURALLY** — their arm
   is 1-param `[s]` (no `req`), and returning `Reply`/`Stop`/`ReplyAndArm` from one is a *located
   assertion*: *"an internal (-) op has no client to reply to"* (`service.wat:1063-1073`). The
   caller-ful/caller-less split is a difference in ARM SHAPE, not a nullable field. **Internal ops
   must therefore receive NO ctx at all** — handing them one with a `None` identity is exactly the
   "none means skip" conflation ruled out on 2026-08-08.

2. **But `Alarm<O>` is `[after <- Duration, op <- :O]` (`service.wat:56`) — `O` is the FULL `<service>::Op`
   type, NOT restricted to internal ops.** So a *public* op can be alarm-armed, fire with a timer in
   the `idx` slot, and land in a caller-ful `[s ctx req]` arm **with no client behind it**. Today's
   only live consumer arms a `-tick` (`tests/services/probe_arc278_self_scheduling.wat:44`), but
   nothing forbids the public case.

> **⛔ RULING OWED (the builder's), and it is a real fork:**
> **(a)** restrict arming to internal ops (a checker rule) — then a public op *always* has a client
> and ctx's identity is total; or
> **(b)** make the identity a closed enum — `Client[…] | Timer` — so the handler must face the
> callerless case exhaustively.
>
> **My recommendation: (b).** It matches everything this arc has ruled — a closed set is an enum, name
> every variant, no wildcard arm, no Option-as-skip — and it is *honest*: an alarm-fired call genuinely
> has no client, and the handler should have to say what it does about that. (a) is simpler but bans a
> capability nobody has yet asked to lose.

## The four questions

| | |
|---|---|
| **Obvious?** | **YES** — a handler is told who called it. Every server framework in this lineage has this argument; ours is the outlier for lacking it. |
| **Simple?** | **YES** — ONE record, produced in one place, threaded to every caller-ful arm. It adds no second mechanism: the extension path is the `~@Surface` splice that already exists. |
| **Honest?** | **YES, and this is the point** — a handler today cannot distinguish two tenants, so any per-caller behaviour it claims is a lie it has no way to check. The callerless ruling above is what keeps the *new* field from telling its own lie. |
| **Good UX?** | **YES** — `[s ctx req]` reads as state / who / what, and the floor gives request-id + timing for free at every service, which today every service would hand-roll. |

**4 × YES**, conditional on the two rulings owed.

## Cost — measured

**120 arms** match the plain `[s req]` binder across **65** files declaring a `defservice`; the true
set is whatever the checker enumerates once the arity changes (R52 — impose the change, the fire IS
the worklist; do not grep for it). One recorded `wat-fix` codemod, mechanical. R65 `SCVTVM IDEM INDEX`
is the precedent and the reassurance: this substrate turns a shape change into a finite located list.

## What is NOT in scope

- **Middleware.** There is no chain and this stone does not invent one.
- **Automatic downstream propagation** (service→service tracing). The *wire-crossing property* makes
  it possible later; threading it is not this stone.
- **Deadlines / timeouts in ctx.** 24y's `NO TIMEOUT` ruling stands.
- **Any field beyond the floor** without a named consumer.

## ⛔ STOPs

1. **STOP-1 — ctx MUST STAY PURE. Do not put a `Peer` in it.** The obvious "improvement" is to hand
   the handler its caller's peer so it can reply directly. That makes ctx impure, and it instantly
   forfeits all three properties above: no wire, no `:durable`, no key. A registered opaque in a pure
   record is now a load-time error (2026-08-08), so this fails loudly — but design it out, don't rely
   on the wall to catch it.
2. **STOP-2 — internal (`-`) ops get NO ctx.** They have no caller; their 1-param arm already says so.
   Do not give them a ctx with an empty identity.
3. **STOP-3 — the alarm hole must be ruled BEFORE the identity field is built.** If a public op can be
   alarm-armed and ctx claims an identity, ctx lies. See the fork above.
4. **STOP-4 — do NOT mint a second correlation id.** Until the `Scope` ruling lands, do not add a
   request-id field that is independent of `Scope`'s uuid.
5. **STOP-5 — the type name is an intueri CAST, owed, not narrated.** `ctx` is the builder's word for
   the *parameter*; the TYPE's name has not been cast. Materialize the candidates and spawn the ward
   (`INCANTO NON NARRO`) — do not let this document's placeholder become the name by default.
6. **STOP-6 — a per-call uuid makes any reply that echoes it non-reproducible.** Decide whether the
   request id crosses into replies, or is injectable for tests, before a golden asserts on one.

## The dependency, stated plainly

**This stone unblocks the connection-scoped world and is unblocked by nothing.** Order: rule the two
forks → cast the name → build ctx (floor only) → then the per-connection map, which needs ctx's
identity to select an entry and needs the lifecycle hooks that same macro change should carry.
