# DESIGN — service I/O budgets: per-op declared limits + fragmentation/pagination tooling (AWS-shaped)

> **Origin (builder, 2026-07-20):** the RICH Rules arena forced this out (`ALIVS ARGVIT` — the consumer
> surfaces the requirement). A shadowdancer got the arena green only by *routing around* a 512 KiB frame
> cap (hand-chunked 2×400 writes, inlined page-loops). The builder rejected the shortcuts: *"i do not accept
> the shortcuts the shadowdancer took to get us here — we finish this right."* The right shape is the one
> every AWS service already has: **explicit per-operation budgets + client tooling to fit them + server
> pagination** — and callers *love* those limits because the SDK fragments/pages within them, so nobody
> "freaks out on the budget." This doc is that contract for wat services. Its floor is the structural
> mute-kill (`DESIGN-no-hidden-failures.md` — over-budget must *speak* and must *not fault the wire*).

## STATUS (2026-07-21c) — #16.2 DONE (`023a15c4`): the enforcement CODEGEN + Stone 16.3 (`:max-request-bytes` MANDATORY on `:nature :Peer'` ops) + the corpus migration (`wat-scripts/fixes/declare-max-request-bytes.wat`, ~90 files); floor 4200/0. RESUME at item (c) — now DESIGNED as the telemetry-`span` UX (`log` enqueues into an INVISIBLE buffered sink; `with-log-sink` is NOT a user form — the `span`/ctx is; see `DESIGN-self-scheduling-defservices.md` + the 2026-07-21d CURARE CHECKPOINT in `DESIGN-no-hidden-failures.md`). The invisible buffering = a **self-scheduling defservice** substrate stone, whose strike STOPPED on the **poll'/timer gap** (the serve loop `poll'`s over the unified `Peer'`; a `Timer'` fuses only into `Thread'`/`Process'`). **RESUME: resolve the poll'/timer fork FIRST**, then the sink, then wire the span. [Historical scout of 16.2 preserved below.]

**Grounded against the disk, not asserted (`AD ORACVLVM` — a prior breadcrumb wrongly said "Stone 1 HELD"):**
- **Stone 1 (the per-service hard limit `FOO`): DONE.** `:max-frame-bytes` is DECLARED on the defservice (`wat/service.wat`; `journal`/`mem-store` declare 10 MiB), threaded via `pair_with_budget` to the accepted-connection receivers, and over-`FOO` → reasoned reject-close (`ServiceEvent::Lost{cause}`, never mute `Closed`). Gate `probe_arc278_service_max_frame_bytes` is GREEN. The masking LAW (Mechanism A / transport-twin `Failed→DecodeError` / RST) is in. **⇒ "Stone 1 (must land first)" in Scope+sequencing below is SATISFIED.**
- **16.0 (parse `:max-request-bytes` onto each op): DONE** (`src/types.rs:335`, "Stone 16.0").
- **16.1 (Ruling A — every op-Response an enum carrying `RequestTooLarge{bytes,cap}`, checker-locked): DONE** (`9ca2e88d`).
- **16.2 (the per-op ENFORCEMENT codegen): ← RESUME HERE.** Feasibility proven HAND-ROLLED (`probe_arc278_per_op_request_too_large`, the `:impls` body measures+returns). 16.2 MOVES the measure+construct into the serve-loop CODEGEN so no service hand-rolls it. **Scout (2026-07-21):**
  - **Injection:** the `serve-op-arms` foldl (`wat/service.wat:753`) — wrap each op's arm BEFORE `<outcome-match>` (the `:impls` dispatch): `((<S>::Op::<Op> req) (if (> (measure req) <op-cap>) (do (send' … (<S>::Reply::<Op> (<Op>Response::RequestTooLarge n <op-cap>))) (serve …)) <outcome-match>))`; keep the connection.
  - **Byte-source SETTLED = re-encode** `(string::length (edn::write req))` — the serve arm holds the DECODED `req`, not the raw frame bytes (consumed upstream); and it matches how the client tooling fragments (both measure `edn::write` length → they agree). The frame-length option is not reachable here.
  - **CRUX-1 is the GATING decision** (see Open cruxes): the `serve-op-arms` foldl walks `:impls` *bodies* (no budget); the budget's on the *surface* (`types.rs:335`) with no wat accessor yet. 16.2 must build the discovery accessor FIRST — **rec (i): a synthesized per-op constant `<Surface>::<op>/max-request-bytes`** (`synthesize_surface_protocol`, `types.rs:1720`), reachable by BOTH the codegen AND the future client fragment tooling.
  - **RED gate:** an op DECLARES `:max-request-bytes 200`, its `:impls` body does NOT hand-roll (returns `:Ok`); an over-200-byte request → `RequestTooLarge` (from the codegen) + a follow-up small request on the SAME connection → `:Ok`. RED now (the body's `:Ok` comes back), GREEN after.
- **Item (c) and item (b): BUILT, 2026-09-01.** Not as `with-log-sink` — that sketch proposed a
  BRACKET, which is a worker pool and the wrong shape; `span'` already WAS the sink. Item (c) is
  stones A–D (buffer + delta/reset contract + both duration emissions + size trigger; two clocks;
  the size-triggered flush speaks; bounded buffer with a drop counter). Item (b) is
  `write-{logs,metrics}-batched` — built over a Vector, NOT over a `Stream`, which does not exist
  and has no consumer. **Item (a) (`write-*-stream` over a lazy Stream) remains unbuilt by
  RULING, not omission**: it waits for a consumer that actually streams. See the SCOREs beside
  this file. Item (c) is where the `:wat::telemetry::log` widget threads into the buffered `with-log-sink` collector — unblocked once CRUX-1's accessor exists.

## The model (builder-ratified 2026-07-20)

Three pillars, exactly the AWS pattern:

1. **Explicit, per-op, contract-declared budgets** — like DDB (`Query` ≤1 MB response, `PutItem` ≤400 KB
   item, `BatchWriteItem` ≤25 items/16 MB), CloudWatch (`PutLogEvents` ≤1 MB/10k events), S3 (5 TB/object
   via multipart). A service does **not** have "a budget"; each **operation** carries its own. The limit is
   part of the published contract — **declared on the surface's `:features`, discoverable** (the client and
   the tooling read it off the op, exactly as they read the op's request/response types).
2. **Client tooling to fit the budget** — writers *fragment* an oversized batch into ≤budget submissions;
   readers *page* a cursor to exhaustion. Nobody hand-rolls chunking (the arena's crime).
3. **Server pagination that respects the budget** — responses are paged so each page fits, **including the
   output-side** (below): rules-querying is *inference*, so `|output|` is unbounded vs `|input|` — the
   response must stream, not assume it fits.

## Budget declaration — per-op, on the surface `:features` (the contract, discoverable)

Grounded: a `defsurface` declares method members under `:features`; a `defservice` `:satisfies` a surface
and `:impls` the bodies (`wat/service.wat:141-161`, `:ops` retired). The surface **is** the IDL (R31
`SATISFACTIO LIMEN TRANSIT`) — so the budget belongs there, as an annotation on each op, shipped with the
surface's `:messages`/protocol and discoverable by any holder.

```clojure
(:wat::core::defsurface :wat::telemetry::Journal
  :nature :wat::kernel::Peer'
  :messages [ …Request/Response records+enums… ]
  :features
  ;; each op annotates its OWN budget — request cap and (for paged ops) response-page cap.
  [(write-logs  [self <- … req <- …WriteLogsRequest]  -> …WriteLogsResponse
                  :max-request-bytes 10485760)                     ;; 10 MiB — the big write
   (query-logs  [self <- … req <- …QueryLogsRequest]  -> …QueryLogsResponse
                  :max-page-bytes    524288)                       ;;  512 KiB per response page
   (sift-rules  [self <- … req <- …SiftRulesRequest]  -> …SiftRulesResponse
                  :max-page-bytes    524288)                       ;; output paged (inference-explosion)
   (stats       [self <- …]                           -> …StatsResponse)])
                                                                   ;; no annotation → the default (512 KiB)
```

- **Default:** an op with no annotation inherits `DEFAULT_MAX_FRAME_BYTES` (512 KiB, `edn_shim.rs:1330`).
  Most ops want nothing more; only bulk/paged ops declare.
- **Discovery:** the budget is synthesized alongside the op's `::Op`/`::Reply` (`synthesize_surface_protocol`,
  `src/types.rs:1713`) so a holder of the surface can read op `X`'s limits without a round-trip. **CRUX-1
  (open):** the exact discovery accessor — a synthesized `<Surface>::<op>/max-request-bytes` constant vs a
  budget table on the surface value. Resolve when the fragment tooling is drawn (it's the first consumer).

## Two budget layers — the server hard limit and the per-op limit (both INDEPENDENT, builder-ruled 2026-07-19)

The wire is **newline-framed, no length header** (`comms/process.rs:300`), so the transport cap fires *during*
accumulation — **before** the op is known. There are **two independent limits**, and a request is **good or it
isn't — we never process a bad request** (no partial consumption, no "be nice, we gotcha… uh, wtf do we do when
we don't"):

| layer | the limit | when enforced | on violation |
|---|---|---|---|
| **transport** | the **service hard limit `FOO`** — **per-service, DECLARED** ("how many bytes I'll attempt per read"), op-agnostic | **during accumulation** (pre-decode) | **reject + CLOSE that connection** (reasoned, never mute); keep serving everyone else |
| **contract / app** | the **per-op limit** — the service's choice, **`≤ FOO`** | **post-decode**, in the serve loop | the op's **named `RequestTooLarge` response** (a matchable reply); connection **LIVES** |

- **The service hard limit `FOO` (the transport floor — the DoS backstop):** *"this service will never
  accumulate more than `FOO` bytes for any inbound frame, period."* It is **PER-SERVICE and DECLARED** — the
  service says *how many bytes it will attempt per read*, threaded into its socket **listener** →
  accepted-connection receivers. **Not** global, **not** derived from the ops. Read up to that service's `FOO`;
  the instant a frame exceeds it → **stop reading, reject, close that connection.** Accumulation is bounded by
  `FOO`, so there is **no endless drain, no reply-to-a-blocked-sender, no deadlock.** The client's op fails with
  a *reason*, never a mute clean-close (see the SPEAK mechanism below).
  - **The 512 KiB global default stays** (`DEFAULT_MAX_FRAME_BYTES`, `edn_shim.rs:1330`) — the conservative
    fallback for a service that declares nothing (a service with only small ops should NOT be made to hold
    10 MiB per connection — that is the 20× memory/DoS surface a global raise would inflict on everyone).
  - **A bulk service DECLARES more** — the telemetry `journal`/`mem-store` declare e.g. `10 MiB` (the arena's
    ~600 KiB write then arrives). Grounded that this is a real per-connection knob: socket receivers already
    carry a `max_frame_bytes` (`comms/process.rs`), settable via `pair_with_budget(n)` — Stone 1 threads a
    declared `FOO` from the defservice through `listener'`/`accept'`/`from_socket` to the accepted connections.
    (Different services, different limits — proven by the raise-experiment: PROCESS went mute→630 at 10 MiB.)
- **The per-op limit (the graceful, matchable tier):** the request already **fully arrived** (it was ≤ `FOO`)
  and decoded, so the client **is reading** — the serve loop checks it against *that op's* declared
  `:max-request-bytes` and, if over, replies the op's **named `RequestTooLarge{bytes, cap}` variant** (see
  "The response contract"), service keeps serving, connection lives. This is where the *matchable*, graceful
  "your request is too big for this op, here's the cap" belongs. A single request whose op is unknowable (over
  `FOO`) can only get the transport reject+close — which is why `FOO` is set well above any real op's budget, so
  a legitimate over-*op* request still arrives and gets the matchable answer. Paged ops keep each response page
  ≤ `:max-page-bytes`.

**The SPEAK mechanism — the client is TOLD, nobody dies (builder-ruled 2026-07-19; grounded):** a bad request
does not burn anything down — **the client is told its request is too large (a 400), the client survives, the
server survives.** On an over-`FOO` frame the serve loop, via a new **`ServiceEvent::Rejected{idx, cause}`** arm:
(1) **replies `Reply::Failed{cause}`** to that client — the client is READING when this fires (the diagnostic
proved its `send'` completes and it blocks in `recv'` awaiting the reply), so the reply LANDS; `Reply::Failed`
is a *catchable* reason via Mechanism A, so the client is told and handles it (does not die); (2) **evicts that
one connection** (`remove-at`) — discarding the un-read residual of the oversized frame, which would otherwise
desync the wire (this is why `Malformed`, which *keeps* the connection, is wrong here, and why no drain-realign
is needed — closing discards the residual); (3) **keeps serving everyone else** (recur).

> **★ Kicking is correct; the pipe is reaped by Linux (builder-ruled 2026-07-19).** The client is *told* (a
> catchable 400) then the connection resets — it just **reconnects** and retries. We do **NOT** spend a cycle
> draining the pipe to salvage a bad connection: evicting drops the `Peer` → `OwnedFd::drop` → **`libc::close(2)`**
> → the kernel reaps the pipe (discards the buffered residual) for free (grounded: `comms/process.rs` `Sender`/
> `Receiver` own `OwnedFd`s, `:310/:496`). *A bad request is bad; don't waste resources making it not-bad.*
> **Recoverable-*on-the-same-connection* lives one tier up — the per-op limit:** there the request *arrives* (it
> was ≤ `FOO`), the wire is in sync, so an over-op-budget request gets a matchable `RequestTooLarge` and the
> client fixes+retries on the same socket. The transport `FOO` kick is the **DoS backstop** for frames so large
> the wire desyncs — with per-service `FOO`, it only fires on genuinely-huge (abuse/bug) frames.

The poll' process
client-read arm (`runtime.rs:27658`) currently collapses `RecvError::FrameTooLarge` into a reason-free `Closed`
(the mute) → Stone 1 routes it to `Rejected{cause}`. The reply `send'` must be **non-blocking** (an extreme
over-`FOO` frame could leave a client blocked mid-`send` and not reading — a blocking reply would deadlock the
serve loop; non-blocking → skip the reply, evict anyway, the client gets an honest EPIPE on its own `send`).

> **★ NOT `Lost` / NOT `eprintln` (a grounded finding, 2026-07-19):** `eprintln` **is wat's panic** —
> `panic_any` → structured exit, never returns (it is the "I'm dying, here's why" primitive). The serve loop's
> existing `Lost` arm uses `eprintln` *intending* "log the reason + keep serving," but because `eprintln`
> terminates, its `(recur)` is **dead code** — so routing a **client-triggerable** over-`FOO` frame to `Lost`
> would let any client **crash the whole shared service** (a DoS the two-sided contract forbids; proven: a
> `Lost`-routed survival probe fails, the child dies). A client-reachable path must **never** be able to fire the
> terminal `eprintln`. Over-`FOO` is a **400-class client error**, handled by `Reply::Failed` to the client
> (above), no `eprintln`. **Separately: the `Lost` arm's `eprintln` is a latent burn-it-down bug** (a genuine
> peer break crashes the whole service instead of evict-and-keep-serving) — its own stone: the substrate needs a
> **non-terminal warn/log sink** ("bad thing happened, I'm continuing"), distinct from `eprintln` ("I'm dying").

**RETIRED — the drain-realign "be nice" model (`24ac73e7`, `DESIGN-no-hidden-failures.md` facet 1):** the
earlier design *drained part of an over-budget frame, re-aligned the wire, and kept the connection alive for its
next request*. A request/reply transaction is **atomic** — there is no half-request to process — and keeping
the connection alive forces re-syncing the wire off a sender that may be blocked mid-write → **deadlock** (this
is the deadlock a shadowdancer chased, 2026-07-19). **Retired.** Too big is too big: reject + close; there is
nothing left to re-align.

## The response contract — a NAMED variant per failure kind (builder-ruled 2026-07-20)

**Never overload a `cause` bucket; name every failure kind so the caller cannot guess what they got.**
An op's response enum carries one variant per *distinct* thing that can happen — success, each caller-fixable
(400-class) error, and the server-fault (500-class) error — each obvious, each forced. Because wat `match` on
an enum is **exhaustivity-checked**, the caller **cannot compile without an arm for each failure kind** — this
is `conformare`: diagnostic completeness by *structure*, not hand-discipline; incomplete handling is
uncompilable. A single `::Rejected{cause}` (my first draft) is REJECTED — it makes the caller string-parse a
cause to learn whether it was too-large vs impure-params vs unknown-type. Example (sift):

```clojure
(:wat::core::defenum :…::SiftRulesResponse :wat::enum::Pure
  :Deductions         [items <- :PV<Value>  cursor <- (:Option :Cursor)]  ;; 200 — paged
  :RequestTooLarge    [bytes <- :i64  cap <- :i64]                        ;; 400 — fragment + retry
  :ImpureRules        [cause <- :wat::kernel::Failure]                    ;; 400 — rules/predicate not pure (safety, policed)
  :UnknownMessageType [type-name <- :wat::core::String]                   ;; 400 — a Log's class isn't in :defs
  :Fatal              [cause <- :wat::kernel::Failure])                   ;; 500 — server-side, not the caller's fault
```

- **400-class = caller-fixable, connection LIVES** (retry-smaller / fix-rules / fix-`:defs`); **500-class =
  server fault.** The distinction is a *type*, so the caller's `match` handles "bad request, retry" separately
  from "server died" — your "a 400 is something a client can just deal with without faulting hard," made
  structural. (This RETIRES the sift form's current lumping of unknown-type + everything into one `::Fatal`.)
- Shared kinds (`RequestTooLarge`, `ImpureRules`) likely draw from a common failure-record vocabulary in
  `:wat::query`; the exact sharing is a build detail, the *naming-per-kind* is the law.

### ★ Ruling A (builder-ruled 2026-07-20) — the contract is UNIVERSAL and CHECKER-LOCKED; records-as-Responses are retired for services

Four-questioned and ruled **A** over the alternatives (enum-only-records-exempt / stdlib-only): **every serviceable op-Response is an outcome enum carrying `RequestTooLarge{bytes,cap}`.** The reasoning is the distributed target ("AWS on a single computer" first, then networking as a *transport swap* — [[project_aws_on_a_single_computer_then_networking]]): every op is **wire-capable**, so every op *can* face a too-large request over some transport, so every op-Response must be able to *signal* it. A bare `defrecord` Response cannot carry a variant, so **records-as-Responses are retired for services** (an op-Response is minimally `:Ok | :RequestTooLarge`).

- **Universal, not opt-in.** RequestTooLarge is not "only where a sub-`FOO` budget is declared" — reachability depends on the satisfying service's `FOO` (a default-budget op under a larger-`FOO` service can hit it), which the surface cannot know. So RequestTooLarge rides *every* op-Response, and every arm is a **live** shield, not dead ceremony — the AWS-SDK-uniform contract (every operation carries its error variants; every caller handles them). Materializing the user-forms was decisive: a record-Response op that later crosses a wire would need a *redesign* (record→enum) to signal a breach — the exact "redesign, not a transport swap" the roadmap forbids.
- **Checker-locked (Stone 16.1c).** `synthesize_surface_protocol` (`src/types.rs`) requires each op's `<Op>Response` to resolve to an enum carrying `RequestTooLarge{bytes <- :i64, cap <- :i64}`; a record Response, or an enum omitting it, is a **located compile error**. The wrong shape is uncompilable (`conformare` — the shield is structural, not hand-discipline).
- **Wire-only firing.** The variant rides the *loci-agnostic* Response but only *fires* across a wire (a serialized transport); shared memory (thread) delivers no byte frame, so it is never constructed there. 16.2's enforcement is keyed on the serialized frame — **transport-general, never `if process`** — so UDS / localhost-TCP / mTLS / remote inherit it for free.
- **Migration (Stones 16.1a/b, done):** production stdlib Responses were already enums (RequestTooLarge added + 7 s2s-propagate arms); all 22 test-fixture record-Responses across 31 files migrated to enums (`:Ok` + RequestTooLarge); service-to-service consumers **propagate** the breach as their own op's `RequestTooLarge`, terminal test callers surface it via `assertion-failed!` (positional today — a kwargs sweep of that primitive is an arc-109 note).

## The two-sided contract (builder-ruled 2026-07-20) — defense at the gate + ergonomic tooling

A defservice is a **network service facing untrusted clients who will do dumb shit.** So the contract has two
sides, and both are mandatory:
- **Defense at the gate (server):** the API **MUST enforce** both limits, and neither is ever mute — one dumb
  client cannot mute-crash or DoS the shared service:
  - a frame over the **server hard limit `FOO`** → the transport rejects it and **closes that connection** with
    a *reason* (`Lost{cause}`, never mute); the service keeps serving everyone else.
  - a request that arrived (≤ `FOO`) but is over its **per-op limit** → the serve loop replies the op's named
    `RequestTooLarge` variant and **keeps the connection alive**.
- **Ergonomic tooling (good client):** the tooling has **perfect byte-knowledge** (it encodes + measures), so
  it **never emits an over-budget frame** — the enforcement path is *unreachable for a well-behaved caller.*
  Good clients are just good. The enforcement is real and mandatory; it is simply the path our own tooling
  never walks.

**Per-item ceiling (the one thing even good tooling must reject up front):** a *single* item whose encoded size
— alone in a request, envelope included — exceeds the op budget **cannot be chunked** and is **rejected, not
enqueued**: `::ItemTooLarge{item, bytes, cap}`, returned *before* any write or buffer. So effectively a single
log line must be ≤ `:max-request-bytes` − envelope-overhead (~9.99 MiB for a 10 MiB cap); the tooling does the
*exact* per-item check (encode-alone-in-a-request, measure), not a fixed number. In `with-log-sink`, a `push`
of an over-max item returns `::ItemTooLarge` immediately and never buffers it.

## Writer tooling — the write loop (symmetric with read: consume a Stream / push with backpressure)

Streams are the bulk-I/O interface **both** ways: read *produces* a lazy `Stream` the client pulls; write
*consumes* a `Stream` the client feeds (or pushes into a buffered sink). The client never sees a batch boundary
in either direction — the mandatory budgets are invisible. Three forms, one contract:

### (a) `write-*-stream` — consume a `Stream<item>`, batch to fit → `WriteResult`
The write dual of `<op>-stream`. Takes a `Stream<Log>` (materialized OR lazy/produced-over-time), batches to
the op's `:max-request-bytes` (discovered from the contract), writes each batch, returns a **`WriteResult`**.
```clojure
(:wat::telemetry::write-logs-stream journal log-stream)   ;; -> :wat::query::WriteResult
```
- **Sizing (CRUX-2, lean: exact):** fold accumulating each item's encoded byte-length; cut a batch when the
  next item would cross the budget. Exact `edn::write` length beats an estimate (the encode is needed anyway).
- **Guaranteed flush (no lost tail):** the final partial batch flushes when the input stream ends — structural.

### (b) `write-*-batched` — the have-it-all-in-hand convenience
`(write-logs-batched journal items)` ≡ `(write-logs-stream journal (stream-from items))`. Replaces the arena's
hand-rolled 2×400.

### (c) `with-log-sink` — the buffered producer sink (flush on time-OR-size; the decade-proven Ruby idiom)
For a producer emitting over time (span.wat): push single items, batching hidden, flush on **size OR latency**
(Kafka `linger.ms`+`batch.size` / Kinesis KPL / CloudWatch agent). A **bracket** (ephemeral lifetime, RAII):
```clojure
(:wat::telemetry::with-log-sink journal :max-bytes 1048576 :max-latency-ms 1000
  (:wat::core::fn [sink] -> :wat::query::WriteResult
    (:wat::telemetry::push sink log-a)      ;; backpressured enqueue-ack (below)
    (:wat::telemetry::push sink log-b)))     ;; scope exit (RAII): flush remainder, reap → WriteResult
```
- **The sink is a `defservice` actor, bracket-managed.** Its serve loop `select'`s over `{bounded input channel,
  flush-timer}` — the timer is a real io-selectable `timerfd`/`Timer` (grounded, both loci; `mora`-honest, not a
  sleep). On item → buffer + reset-timer-if-first; on `bytes ≥ :max-bytes` → flush + reset timer; on timer →
  flush if non-empty. `:max-latency-ms 0` collapses to synchronous-per-push (durable ack each).
- **The buffer is `:ephemeral` State** — a `Vec<Log>` + a running byte-count (O(1) size-trigger), threaded
  (rebound, never mutated) through serve: `push` → `(conj buf item)`/`(+ bytes n)`; `flush` → `(Vector)`/`0`.
  Pure data, but `:ephemeral` **by choice** (transient in-flight work, not the soul — a buffer resurrected
  across a hibernation gap is wrong). **Flush at EVERY exit** (`:hibernate`, `:stop`, RAII close) — because
  `:ephemeral` is dropped on the gap, un-flushed items would vanish otherwise; buffer reconstructed empty on
  resume/`:init`. *(Build-time verify: is a PURE field allowed in `:ephemeral`? 293.W mandates it for impure;
  pure-by-choice is believed OK — disconfirming-probe item for the sink strike.)*

### The ack model — backpressured enqueue-ack + separately-surfaced durable outcome (NOT fire-and-forget)
Two honest handshakes at two levels (builder-ruled 2026-07-20):
- **`push` = the enqueue handshake, backpressured** (the arc-214 reactor / bounded-channel model): buffer has
  room → enqueue + **ack immediately** (caller unblocks); sink busy/full → **block** (backpressure) until room
  frees, then enqueue + ack. Every push handshakes — the producer can't outrun the sink. A *completed `send`* IS
  the enqueue-ack; the bounded channel IS the backpressure.
- **The durable outcome is separately surfaced** — the enqueue-ack means "accepted into the buffer," NOT
  "durably written." What landed / failed comes via the `WriteResult` at close **and** a **prompt fail-signal on
  the next `push` after a fatal flush** (the producer learns fast, can stop). No outcome discarded.
- **Not fire-and-forget on either count:** every push handshakes (not flung blindly) AND every outcome surfaces
  (not discarded) — distinct from the outlawed fire-and-forget, which does neither.

### `WriteResult` — named per outcome (same law as the response contract)
```clojure
(:wat::core::defenum :wat::query::WriteResult :wat::enum::Pure
  :Done         [written <- :i64]                                    ;; all landed
  :ItemTooLarge [item <- :wat::core::Value  bytes <- :i64  cap <- :i64] ;; 400 — one un-chunkable item; fix/drop it
  :Failed       [written <- :i64  cause <- :wat::kernel::Failure  remainder <- :wat::stream::Stream<wat::core::Value>])
```
- **No silent loss:** `:Failed` carries `written` + the named `cause` + the **unprocessed remainder** (a
  `Stream` to retry) — the AWS `UnprocessedItems` shape. A mid-bulk `::Fatal` leaves written batches written and
  hands back the rest.
- **Failure-mode fork (open, your ruling):** stop-on-first-failure + remainder (simplest; lean) vs
  skip-bad-items-and-continue-collecting (AWS `BatchWriteItem` style). Lean: **stop + remainder** primary,
  skip-mode an opt-in variant.
- Loci-agnostic (thread ≡ process). Symmetry with read: read puts failure **in-band in the stream**; write puts
  it **in the returned `WriteResult`** — both named, both forced by `match`.

## Reader tooling — `<op>-stream` returns a lazy `Stream<Value>` (the builder's `Enumerator` idiom)

Grounded: wat has **Streams** (arc 118/119; `wat/seq.wat`, `crate::stream::NativeLazyCell`) —
`:wat::stream::lazy <thunk>` / `:wat::stream::cons head tail` / `:wat::stream::empty`, and `map`/`filter`/
`take`/`drop` are lazy over them. This is Ruby's `Enumerator.new do |yielder| … end` (the builder's years-long
paginate idiom): the cursor loop is the *engine*; a flat item-stream is the *surface*.

So the reader is **one** synthesized helper per paged op — no generic layer, no `-all`/`-each` zoo:

```clojure
;; Ruby: loop { resp = client.query(params.merge(next_token:)); resp.items.each { yielder << _ }; break if next_token.nil? }
(:arena::my-sift/sift-rules-stream svc base-req)   ;; => :wat::stream::Stream<wat::core::Value>
;; the consumer uses NORMAL lazy ops — never sees a page or a cursor:
(:wat::core::foldl f acc (:arena::my-sift/sift-rules-stream svc base-req))
(:wat::core::into (:wat::core::Vector …) (:arena::my-sift/sift-rules-stream svc base-req))   ;; = the old "-all"
(:wat::core::take 10 (:arena::my-sift/sift-rules-stream svc base-req))                       ;; constant-memory early stop
```

- **The generator** (synthesized per op — the macro knows the response shape): `stream::lazy` a thunk that
  drains the current page's buffered items via `stream::cons`, and when the buffer empties, fetches the next
  page (one `sift-rules` call) and continues; `stream::empty` when the cursor is dry. Per-page is the engine
  that makes the read **exhaustive**; laziness makes it **constant-memory** (`take` stops early).
- **`-all`/`-each` are NOT separate tools** — `(into [] stream)` / `(foldl … stream)` are the standard lazy
  ops. One `<op>-stream`; everything else is the stream library. (The earlier generic `page-all` is YAGNI —
  the per-op synthesized stream knows its own response destructure; a generic layer only earns its place if a
  dynamic-op consumer ever demands it — `ALIVS ARGVIT`.)
- **Sub-detail 1 — start-vs-done state (fetch-first):** the generator carries a small `Start | More(cursor) |
  Done` state so the FIRST page always fetches and a post-last-page `None` terminates — conflating "not yet
  fetched" with "no more pages" (both `cursor = None`) is the trap; the Ruby loop avoids it by fetching then
  checking `next_token.nil?`. Model it explicitly.
- **Sub-detail 2 — failure is an IN-BAND stream element, case-matched (builder-ruled 2026-07-20; NOT a raise).**
  The stream element is an **enum**: `<op>Item = :Item[value]` plus the op's **named failure variants**
  (`:RequestTooLarge` / `:ImpureRules` / `:Fatal` — synthesized per op, so the named-error granularity
  survives into the stream). A mid-stream page failure lands as a **terminal `<op>Item` failure element** at the
  page boundary where it happens, then the stream ends — NEVER a silent `stream::empty`, and NEVER a raise. A
  raise would be an out-of-band failure channel — either uncatchable (`panic_any`, defeating "a 400 the client
  just deals with") or forgotten (no wrapper → hard fault) — both the exact masking/hard-fault this arc kills.
  In-band makes the failure a **value the type system forces the consumer to match** (`conformare`, exhaustive;
  it cannot be swallowed). Happy-path ergonomics are preserved by composition: `(map unwrap-or-raise (op-stream
  …))` → a `Stream<Value>` that raises on a failure element — **in-band enum is the honest primitive; raise is
  opt-in sugar on top** (YAGNI until wanted). So `<op>-stream : … -> Stream<<op>Item>`, and the `into`/`foldl`/
  `take` examples above `match` each element (`:Item` vs the named failures).

## Output-side streaming — the two-level cursor (the inference-explosion crux)

The load-bearing insight (builder): **rules-querying produces potentially MORE output than input.** The sift
Rules form today pages the **input** (journal rows at `:limit`, cursor = the journal's next-key) and returns
**all deductions for that page in one response frame**. If `:limit`=100 rows each fan out to 40 deductions =
4000 records in one reply → that reply itself can blow the page budget. So **input paging is not enough** —
the **output** needs its own budget + resume point.

**The composite cursor (builder-accepted, riff if awkward):**

```clojure
(:wat::query::Cursor
  [row-cursor  <- (:wat::core::Option :wat::core::String)   ;; the journal input position (as today)
   ded-offset  <- :wat::core::i64])                         ;; deductions already emitted from the CURRENT input page
```

- Serve loop: fire rules over the input page, collect deductions, emit up to `:max-page-bytes` worth, set
  `ded-offset` to how many were emitted, and return `Cursor{row-cursor(unchanged), ded-offset}` if more
  deductions remain for this input page; else advance `row-cursor` and reset `ded-offset` to 0. Done when
  both are exhausted.
- The client's `page-all`/`page-each` treats the composite cursor opaquely — loops until done.
- **Determinism (required):** deductions must be emitted in a **stable order** across fires so `ded-offset`
  resumes correctly — pure replay (R5/R18 `RENASCOR NON RETRACTO`) gives this for free (same facts+rules →
  same deductions, same order), but the collection order must be deterministic (sort by a stable key, or
  preserve fire order). **CRUX-3 (open):** the stable ordering key.
- **CRUX-4 (open) — the resume cost on the STATELESS sift form.** The sift op is stateless (each call
  re-fires from the journal). To resume at `ded-offset` within an input page, it must **re-fire that input
  page and skip the first `ded-offset` deductions** — pure replay makes this *correct* (R5/R18) but it is
  *re-work*: an input page whose output spans K output-pages is re-fired ~K times (≈O(K²) in the pathological
  fan-out). Options: (a) accept it — output-explosion within a single input page is rare, and re-firing a
  ≤`:limit`-row page is cheap; (b) shrink the input `:limit` when fan-out is high so output rarely splits;
  (c) note that the **stateful streaming R0** (task #7, `MACHINA CHAOS DOMAT` — a live `Session` across
  messages) has NO re-fire cost (it holds the fired state), so heavy output-streaming is naturally R0's
  domain, and the stateless sift form can accept (a)/(b). Lean: **(a)+(c)** — accept the bounded re-work on
  the paged form; heavy streaming belongs to R0. Resolve at the output-streaming strike.

## Scope + sequencing

- **Stone 1 (must land first) — the per-service hard limit `FOO`:** let a defservice **declare** its `FOO`
  (bytes-per-read) and **thread it** from the service through `listener'`/`accept'`/`from_socket` to its
  accepted-connection receivers (the 512 KiB `DEFAULT_MAX_FRAME_BYTES` stays the fallback; the journal/mem-store
  declare ~10 MiB); and make an over-`FOO` frame **reject + close with a reason** (`FrameTooLarge` →
  `ServiceEvent::Lost{cause}`, not the mute `Closed`) while the service **keeps serving everyone else**. This
  one stone kills the mute floor AND unblocks the arena (the ~600 KiB legit write now arrives — proven by the
  raise-experiment). The drain-realign (`24ac73e7`) becomes moot (reject+close closes the connection) — retire
  it as cleanup. RED probe (both loci): (a) a service declaring a large `FOO` accepts a legit ~600 KiB request
  → **succeeds**; (b) a service declaring a *small* `FOO` gets a frame > its `FOO` → the caller's op fails with
  a *reason* (owner sees the cause via `Lost`, not the mute "peer closed") **and** the service stays alive (a
  follow-up in-budget request on a fresh connection succeeds). At HEAD both fail (mute + 512 KiB cap).
- **Then, in order:** (1) **per-op limits** — declare `:max-request-bytes` (`≤ FOO`) on `:features` + discovery,
  and **post-decode enforcement** in the serve loop returning the op's named `RequestTooLarge` response (the
  matchable, graceful tier — the request already arrived, so no transport work here); (2) writer fragmentation +
  reader pagination tooling; (3) output-side streaming (the composite cursor). Each: DESIGN → RED disconfirming
  probe → brief → shadowdancer → weigh by own re-run.
- **Then re-do the arena on the fixed substrate, shortcuts DELETED:** a single `write-logs` within `FOO` (no
  hand-chunking — or `write-logs-batched` if genuinely over an op limit); real `page-all` tooling; and a RED
  assertion that an over-`FOO` request rejects-with-a-reason-and-closes (service alive) on both loci. The arena
  commit stays **HELD** until at least Stone 1 lands (never ship green on the masked teardown).
- **OUT (rejected):** a length-prefixed rewrite of the wire (newline framing + a bounded read-to-`FOO` is
  sufficient); silent chunking inside the transport (fragmentation is the *client's* tooling, explicit,
  AWS-shaped); the **drain-realign / keep-the-connection-alive** model (retired — a transactional request is
  atomic, "too big is too big", reject + close); and **deriving** the transport ceiling from the op budgets
  (rejected in favor of the independent server hard limit `FOO` — the ops choose limits `≤ FOO`, they do not
  set it).

## Open cruxes (tracked)

- **CRUX-1** — the discovery accessor shape (synthesized constant vs surface budget-table). Resolve at the
  fragment-tooling strike (its first consumer).
- **CRUX-2** — request-fragment sizing (exact encode-length vs estimate). Lean: exact.
- **CRUX-3** — the stable deduction-ordering key for output-cursor resume. Resolve at the output-streaming
  strike.

*Realization-shaped (the AWS I/O contract, forced out by the arena) — the song is the builder's to hand.*
