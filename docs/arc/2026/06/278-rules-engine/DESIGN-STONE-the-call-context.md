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

## ⛔⛔ SCOPE CUT, builder-ruled 2026-08-09 — READ THIS BEFORE THE FLOOR BELOW

> *"we stopped working on telemetry to resolve all the 170 IPC issues completely… i do not care that
> we don't have reads written for them - we need the data written such that a read can be built
> trivially. do we need query-by-uuid now to thread a context record into a service handler who bears
> the caller's identity such that it can destroy a rete world on disconnect?"*

**No — query-by-uuid does not block this.** But the "one field" cut I drew from that is **WRONG and
was overruled the same hour.** The builder:

> *"i want the fucking uuid, namespace and time on the context — just because we do not use them
> doesn't mean we will not use them… can we not build the plumbing for a system not yet in use?… the
> realization that we needed a context blob to bear the caller's identity IS the exact need to ship a
> request-id, request-start-time and so on (the operation IS the request handler's name; the namespace
> is literally the service's name… they're just keywords?)"*

**He is right, and the last clause is the proof.** GROUNDED:

- **`namespace` = the service's fqdn** — the macro already holds it: `fqdn` / `fqdn-str` / `fqdn-kw`
  (`wat/service.wat:85-93`).
- **`operation` = the op arm's name** — the macro already computes it: `op-str`
  (`wat/service.wat:883`, `:942`, `:954`, `:988`, `:1417`).

**Both are COMPILE-TIME LITERALS the macro already has in hand for other purposes.** Splicing them
into a ctx constructor is two keywords. There is no design work in it, no correlation core, no
relocation.

**MY ERROR, named precisely:** I conflated *"ctx carries correlation fields"* with *"ctx splices the
telemetry `Scope` surface."* The second needs the relocation and the naming ruling; the first is five
scalars. I let the MECHANISM question contaminate the CONTENT question, then deferred the content on
the mechanism's grounds — and nearly manufactured a second migration across every op arm to add
fields we already know we want.

**⇒ THE ctx FLOOR IS FIVE FIELDS, and none of them wait on telemetry:**

| field | where it comes from | cost |
|---|---|---|
| **caller identity** | the serve loop's connection table | the stable id this stone exists for |
| **namespace** | the service fqdn — `fqdn-kw`, compile-time | a spliced literal |
| **operation** | the op name — `op-str`, compile-time | a spliced literal |
| **request-id** | minted per call in the serve loop | one `Uuid/v4` |
| **start-ns** | stamped per call in the serve loop | one clock read |

All five are **pure scalars** (`Uuid` is in `is_pure_type`'s pure-scalar list). The record stays pure,
wire-crossable, `:durable`-legal. **Build the plumbing once, with the fields we know we want.**

**What genuinely IS deferred** (and only these): whether ctx SPLICES a shared correlation surface
rather than declaring its own fields; the `Scope` relocation + namespace/type naming; `tags`; and the
by-uuid READ path. Those are mechanism and telemetry. **None of them gate the five fields above** —
a later splice can replace hand-declared fields without touching a single call site.

**⇒ BUILD ORDER:**

1. **ctx with the five-field floor** — arity-dispatched arm shape (`[s]` / `[s req]` / `[s ctx req]`),
   a stable caller id minted in the generated serve loop, and lifecycle hooks threading state on
   `Connection`/`Closed`/`Lost`. **Blocked by nothing.**
2. **the connection-scoped world** — needs only (1).
3. **telemetry** — the correlation surface, the relocation, `tags`, the by-uuid reader. Refines (1)
   without disturbing it.

**The telemetry state, for whoever picks it up:** the *write* side is complete and correct — every
`Metric` and every `Log` writes the `by-uuid` correlation GSI (`journal.wat:12`, `:46`, `:59`, schema
ensured at `:99`), so the data is already on disk in the shape a read needs. What is missing is only a
*reader*: `query-metrics`/`query-logs` call `Store/scan` on the base table and **never call
`Store/scan-index`**, so the uuid pivot cannot be performed today. `Store/scan-index` itself is built
and proven (`query.wat:527`; `tests/services/probe_arc278_tagged_keys_store.wat` Test B round-trips a
`#uuid` GSI scan). **The gap is one op, not a mechanism** — which is exactly the builder's bar: *the
data is written such that a read can be built trivially.*

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

> ⚠ **THE FORK AS FIRST POSED WAS ONE-SIDED — builder-corrected 2026-08-09:** *"so the option is we
> have correlation or we don't?…… feels kinda like a one sided question?"* Correct. If a correlation
> id only earns its name when the logs carry the request's id, then "two independent ids" is not an
> option, it is the failure mode. Posing it as a choice manufactured a non-option
> (`[[feedback_ground_the_decision_before_you_pose_it]]`).
>
> **The real question is mechanical, and the disk has already answered most of it.**
>
> **LOAD ORDER FORCES THE DIRECTION.** `wat/service.wat` is registered at `src/stdlib.rs:324`;
> `wat/telemetry.wat` at `:422` — **service loads ~100 entries FIRST** (confirmed by the neighbours'
> own comments: telemetry's journal *"Loads LAST — after telemetry.wat … and service.wat"*). So:
>
> - **ctx CANNOT splice `:wat::telemetry::Scope`** — `Scope` does not exist yet when `service.wat` loads.
> - **`Scope` CAN splice a type defined in `service.wat`** — it loads after.
>
> **AND NEITHER IS A SUPERSET OF THE OTHER**, which kills the "one of them just splices the other"
> shape:
>
> | | `Scope` (telemetry) | ctx (service) |
> |---|---|---|
> | correlation id | `uuid` | request-id |
> | a timestamp | `time-ns` | start-ns |
> | **only here** | `namespace`, `tags` | **caller identity** |
>
> A log line has no caller; a request has no facility/tags. Splicing either into the other drags a
> field that does not belong — a `caller` on every `Metric` is exactly the kind of lie 293 exists to
> prevent.
>
> **⇒ THE SHAPE THAT SURVIVES: both splice a SHARED CORRELATION CORE** — smaller than both, holding
> only `{correlation id, timestamp}` — and that core must live **at or below `service.wat`'s load
> position** for ctx to reach it. The record already gestures at this: the 2026-07-04 curare lists
> *"the shared correlation-core surface"* as an OPEN ITEM, and `Scope` is described in its own header
> as *"the correlation core."* It was designed as a core; it just lives too late in the load order to
> be one for `service.wat`.
>
> **⛔ WHAT IS ACTUALLY OWED (narrow, and a real question):** **where does the shared core live, and
> what exactly is in it?** Candidates: a new low-loading home; `wat/core.wat`; or `service.wat` itself
> defining it and `telemetry.wat` splicing it. Do NOT hand-copy the id from ctx into `Scope` at the
> producer — that is derive-is-the-wall violated, and it drifts on the first refactor.

## ★ THE SHAPE, RULED 2026-08-09 — three levels, and the metric half is ALREADY BUILT

The builder specified the observability shape from his AWS practice. Grounding it against the disk
found most of it already shipped in `wat/telemetry/span.wat`.

**Ruling 1 — CloudWatch is a VOCABULARY REFERENCE, not a constraint.** *"we own the db layer here -
we don't have to get parity with cloudwatch."* So the fan-out/dimension-identity questions the
CloudWatch model forces do not bind us.

**Ruling 2 — `namespace : operation` is a TWO-LEVEL NAMESPACE.** *"i view the metric-name as a
secondary namespace"* — `namespace: "my-app"`, `operations: ["my-first-request-handler", …]`.

**Ruling 3 — TAGS LEAVE THE METRIC.** *"tags are just a thing we put on the logs records as just
extra metadata about this specific invocation that all log lines share in addition to uuid… i've
never really reached for dimensions… we can just have an empty dimension set until we know we need
it."* Tags are LOG metadata. (Independently confirmed by the CloudWatch doc: dimensions are part of a
metric's *identity*, so a varying tag-set forks the metric — and a per-request uuid as a dimension is
a cardinality bomb. We avoid the whole class by not putting them there.)

**Ruling 4 — RAW SAMPLES ARE KEPT; the derived stats come at emit.** `{:timers {"some-timed-thing"
[25000 25000]}}` → summed to a total and a count. R5 at the telemetry layer: store the samples, derive
the answer. (Also the only way to keep percentiles — the CloudWatch doc says a pre-aggregated
StatisticSet forfeits them.)

**Ruling 5 — the REQUIREMENT, stated:** *"we need to be able to see where our largest time sinks are."*
Ranking by total time is the acceptance criterion, not a nice-to-have.

### ⇒ THE THREE LEVELS

| level | fields | lifetime |
|---|---|---|
| **unit of work** | `uuid`, `namespace` | the whole request — THE CORRELATION CORE |
| **operation** | `operation`, counters, timers, start/end | one `with-op` scope; emits N metrics at close |
| **event** | instant, level, message, **tags** | one log line |

**`time-ns` is NOT in the correlation core** — it varies by role (`time-ns` per log event,
`start-time-ns` per operation), which is precisely why `span::Record` had to hand-redeclare three of
`Scope`'s four fields instead of splicing it. **`tags` is NOT in it either** (ruling 3). The core is
`{uuid, namespace}`.

### ★ ALREADY ON THE DISK — do not rebuild

`wat/telemetry/span.wat`'s own header: *"On `close` it emits the accumulated state as Metrics to the
sink (**each counter → 1 Metric; each duration name → a `<name>/count` + a `<name>/duration`
Metric**)"* — the builder's exact fan-out. And the vocabulary exists:

```clojure
(defenum   :wat::telemetry::Numeric :I64 [val] :F64 [val])                 ;; the :value
(defenum   :wat::telemetry::Unit    :Count :Nanos :Millis :Bytes :Percent) ;; the :type
(typealias :wat::telemetry::Samples (Vector i64))                          ;; the raw [25000 25000]
(defrecord :wat::telemetry::Metric  [~@Scope start-time-ns name value unit])
(defservice :wat::telemetry::span   :durable [namespace uuid tags start-time-ns counters durations])
```

`Counter` = `Unit::Count`; `Duration` = `Unit::Nanos`. (R61 `PAR NON ARGVIT, NOSTRA ARGVVNT` — our own
prior art, consulted this time.)

### THE DELTA — what is actually missing

1. **`operation`** — carried by NOTHING today. Not in `Scope`, not in `span::Record`. It is the
   builder's secondary namespace and the thing that makes a metric legible; without it every metric
   from one service is distinguishable only by its timer names.
2. **`tags` moves** — off `Scope`/`Metric`, onto the log record (ruling 3).
3. **the correlation cut** — `Scope` loses `time-ns` and `tags`, becoming `{uuid, namespace}`; each
   consumer stamps its own time with its own meaning.
4. **ctx plumbing** — the per-call context that makes any of this reachable from a handler; this stone.

### ⛔ ONE OPEN, and it is small

**Does a Metric carry the `uuid`?** Ruling 3 puts tags on logs "in addition to uuid", which says logs
carry it; it does not say metrics lose it. Recommendation: **keep it on metrics.** We own the DB, so
there is no cardinality penalty, and it buys the single most valuable debugging move — *this operation
was slow → here are that exact invocation's log lines.* Drop it and metrics and logs can only be
joined by namespace+time, which is a guess.

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
> **⚠ MY RECOMMENDATION OF (b) IS WITHDRAWN — the four questions, run at the builder's direction,
> flip it to (a). The reasoning that produced (b) is kept below, wrong, because it was wrong in an
> instructive way.**
>
> *The withdrawn argument:* "(b) matches everything this arc has ruled — a closed set is an enum, name
> every variant, no wildcard arm, no Option-as-skip — and it is honest." Right about closed sets;
> **wrong about which situation this is.** The honest move is not to make 120 handlers face an
> impossible case; it is to make the case impossible. That is the extirpare ladder, and (a) sits a
> rung higher.

### THE FOUR QUESTIONS — (a) vs (b), flat, each YES/NO

**(a)** = arming a public op is REFUSED at the definition site; caller-fulness is carried by ARM SHAPE
(`[s]` / `[s req]` / `[s ctx req]`, arity-dispatched); the identity is a plain, TOTAL `ConnId`.
**(b)** = the identity is a `Client[…] | Timer` enum, faced at runtime in every caller-ful handler.

| | **(a) compile-time refusal** | **(b) runtime enum** |
|---|---|---|
| **Obvious?** | **YES** — `[s req]` says "I don't need to know who"; `[s ctx req]` says "I do". The rule is learned once, at a located error. | **NO** — a reader meets `Caller::Timer` in a query handler and asks *when does a query fire from a timer?* For every service in the corpus the answer is **never**. A variant that cannot occur is not obvious; it is puzzling. |
| **Simple?** | **YES** — ONE mechanism, and it EXTENDS one that already exists (the macro already dispatches a 1-param `[s]` arm for internal ops). No new type, no new match. | **NO** — adds a second type (`Caller`) **and** a mandatory match in every caller-ful arm. Two things where (a) has zero. |
| **Honest?** | **YES** — the identity is TOTAL: it can never name a caller that does not exist, because the case cannot arise. | **NO** — and this is the surprise. It is honest about the *value* and dishonest about the *situation*: it models as live a case the design intends to be impossible, and mints **~120 arms nothing can ever contradict** (`[[feedback_an_unreachable_arm_accumulates_lies]]`). Worse, each such arm hand-reimplements the `assertion-failed!` the macro **already performs** for callerless ops (`service.wat:1063-1073`). |
| **Good UX?** | **YES** — **0 existing arms change**; the third param is opt-in; no dead arms. | *not reached* — Obvious ∧ Simple ∧ Honest must hold first, and (b) fails all three. |

**(a) = 4 × YES. (b) fails at the first question and does not reach UX.**

### Why the compile-time bonus is decisive here, not merely nice

The builder: *"compile time gets a pretty significant bonus."* It is the extirpare ladder stated as a
preference, and this fork is a clean instance of it:

```
convention   "don't arm a public op"                    — nobody remembers
check        (b) every handler inspects Caller at run   — 120 sites, 120 dead arms
no form      (a) the definition is REFUSED              — the mistake cannot be written
```

(b) forces every handler to *look at* a value; it never forces any of them to *do* anything different
— which is the must-use gate's own failure mode (`[[feedback_a_match_with_identical_arms_is_a_discard]]`).
(a) removes the situation instead of documenting it 120 times.

### What (a) costs, stated plainly

One capability: **no public op may be alarm-armed.** Nobody does it today — the only live consumer
arms a `-tick` (`tests/services/probe_arc278_self_scheduling.wat:44`) — and internal ops exist
precisely to be the armable ones. The workaround is also the better factoring: a public op and an
internal `-tick` that both call one shared helper. So (a) does not lose the capability; it forces the
decomposition that should have been written anyway.

### ✅ RUN 2026-08-09 — (a) IS NOT FREE, AND THE HOLE IS LIVE, REACHABLE AND SILENT

The hypothesis was *"maybe the checker already refuses arming a public op, so (a) is half-built."*
**Refuted by run.** Three results, in order, each on `target/release/wat`:

| # | form | verdict |
|---|---|---|
| 1 | `:op :poll` — bare keyword, mirroring the working `:op :-tick` | **refused**, exit 1 — `TypeMismatch: expects Alarm<keyword>; got Alarm<probe::tick2::Op>` |
| 2 | `:op (:probe::tick2::Op::Bump (…Request…))` — the explicit ctor | **ACCEPTED**, `--check` exit 0 |
| 3 | the same, RUN with a state-mutating witness | **the handler FIRED** — durable count `7 → 8`, process exit 0 |

**Result 1 is an ACCIDENT, not a wall.** The macro's keyword→`Op` resolution covers *internal* (`-`)
ops only (`service.wat:992` — *"rewrite each internal-op keyword (`:-tick`) … to its `<service>::Op`
variant ctor"*), so a bare `:poll` never becomes an `Op` and the `Alarm<keyword>`/`Alarm<Op>` mismatch
trips. Nothing is checking intent. **Result 2 is the route around it, and it is the natural one if you
follow the types.**

**Result 3 is the finding, and it needed the witness.** The first run armed the read-only `poll` and
printed `7` — the service survived, but that could not distinguish *"fired and was harmless"* from
*"never fired at all."* Re-run with a **mutating** public op (`bump`, `count + 1`) as a non-vacuity
control: the client's later read returned **8**. So the armed PUBLIC handler **executed from the timer
fire**, with a timer in the `idx` slot, **mutated durable state**, and returned `Outcome::Reply` — and
**no error surfaced anywhere**; the service went on to serve a real client normally.

**So the defect exists TODAY, independent of ctx: a handler can run believing it is serving a client,
have no client, and have its reply go nowhere with nothing reported.** That is a silent discard of
exactly the kind this arc has spent itself annihilating (R55/R57) — reached by writing one ordinary
constructor.

**Consequences for the fork, stated:**
- **(a) must be BUILT** — a real checker rule refusing a public op inside an `Alarm`. It is not free.
- **(a) gains a second and stronger justification:** it does not merely keep ctx's identity honest, it
  **closes a live silent-discard that exists right now**, before ctx is built at all.
- **(b) would surface** the case (the handler must face `Caller::Timer`) — but at 120 dead arms, and
  it leaves the wrong form *writable*, merely observable.
- A **third, lower rung** is now visible: keep arming legal and make the callerless reply LOUD, reusing
  the `assertion-failed!` the internal-op arm already carries. Cheaper than (b), no dead arms — but a
  runtime error where a compile-time refusal is available.

**Reproduction, kept so the numbers outlive the session.** A `defsurface` with two public ops
(`bump` mutating, `poll` reading) and a `defservice` whose `start` arms the public op:

```clojure
(start [s req]
  (:wat::service::Outcome::ReplyAndArm s (:probe::Tick2::StartResponse::Ok)
    ;; ⛔ a PUBLIC op, armed. --check exit 0. Fires. Mutates. Reply vanishes.
    [(:wat::service::Alarm :after (:wat::time::Millisecond 5)
       :op (:probe::tick2::Op::Bump (:probe::Tick2::BumpRequest)))]))
```

Driver: `start` the service at `:count 7`, connect, call `start` (arming it), wait 60ms via a
`select'`-on-`after` nap (not a sleep-guess), then `poll`. **Observed `8`.** Without the mutating
witness the run prints `7` and proves nothing — that control is the difference between a finding and
a vacuous green.

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
