# DESIGN — service I/O budgets: per-op declared limits + fragmentation/pagination tooling (AWS-shaped)

> **Origin (builder, 2026-07-20):** the RICH Rules arena forced this out (`ALIVS ARGVIT` — the consumer
> surfaces the requirement). A shadowdancer got the arena green only by *routing around* a 512 KiB frame
> cap (hand-chunked 2×400 writes, inlined page-loops). The builder rejected the shortcuts: *"i do not accept
> the shortcuts the shadowdancer took to get us here — we finish this right."* The right shape is the one
> every AWS service already has: **explicit per-operation budgets + client tooling to fit them + server
> pagination** — and callers *love* those limits because the SDK fragments/pages within them, so nobody
> "freaks out on the budget." This doc is that contract for wat services. Its floor is the structural
> mute-kill (`DESIGN-no-hidden-failures.md` — over-budget must *speak* and must *not fault the wire*).

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

## Two budget layers (per-op is primary; the transport ceiling is derived)

The wire is **newline-framed, no length header** (`comms/process.rs:300`), so the transport frame cap fires
*during* accumulation — **before** the op is known. Therefore:

| layer | granularity | when enforced | who uses it |
|---|---|---|---|
| **contract / app** | **per-op** (declared) | **post-decode**, in the serve loop | fragment/page tooling + serve-layer 400 |
| **transport** | **per-connection** (`max_frame_bytes`) | **during accumulation** (pre-decode) | DoS backstop only |

- **Per-op (primary):** the serve loop, after decoding a request, checks it against *that op's* declared
  `:max-request-bytes` → over → the op's **named 400 variant** (`::RequestTooLarge{bytes, cap}`, see "The
  response contract" below), service keeps serving. Paged ops keep each response page ≤ `:max-page-bytes`.
- **Transport ceiling (derived):** the per-connection `max_frame_bytes` = **max over the service's declared
  op request-budgets** (auto-derived at spawn from the surface's annotations — no hand-set number). A frame
  bigger than *any* op could accept is rejected at the transport, but — per the mute-kill — it **speaks**
  ("frame exceeds the N-byte ceiling") and **does not fault the wire** (drain-realign → 400 → keep serving;
  see `DESIGN-no-hidden-failures.md`). Wiring: `spawn_process_peer(max_frame_bytes)` already threads a budget
  (`spawn.rs:756/775`) — derive it from the surface instead of the `ProcessOpts` default, **and** apply it to
  the **input** channel too (`spawn.rs:765` currently hardcodes `pair()` = 512 KiB — the gap that let the
  arena's *write* die; the output channel at `:775` already takes the budget).

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

## The two-sided contract (builder-ruled 2026-07-20) — defense at the gate + ergonomic tooling

A defservice is a **network service facing untrusted clients who will do dumb shit.** So the contract has two
sides, and both are mandatory:
- **Defense at the gate (server):** the API **MUST enforce** every budget — a bad/hostile/buggy client's
  over-budget frame gets a *reasoned* rejection (the named `::RequestTooLarge`, never mute — #15) and the
  service **keeps serving everyone else** (one dumb client cannot mute-crash or DoS the shared service). The
  transport `FrameTooLarge` guard + the serve-loop per-op enforcement are the gate.
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

- **Floor (must land first):** the structural mute-kill — over-budget *speaks* + 400-and-continue + the wall
  (`DESIGN-no-hidden-failures.md`). Without it, every budget is a mute cliff. RED probe: send >cap → client
  gets a *reasoned* 400 **and** a follow-up request on the same service succeeds (connection + service alive),
  both loci.
- **Then, in order:** (1) per-op budget declaration on `:features` + discovery + derived transport ceiling +
  the `spawn.rs:765` input-channel fix; (2) writer fragmentation + reader pagination tooling; (3) output-side
  streaming (the composite cursor). Each: DESIGN → RED disconfirming probe → brief → shadowdancer → weigh by
  own re-run.
- **Then re-do the arena on the fixed substrate, shortcuts DELETED:** a single `write-logs` within the raised
  cap (no hand-chunking — or `write-logs-batched` if genuinely over 10 MB); real `page-all` tooling; and a
  RED assertion that an over-budget request 400s-without-faulting on both loci. The arena commit stays
  **HELD** until at least the floor lands (never ship green on the masked teardown).
- **OUT (rejected):** raising the *global* `DEFAULT_MAX_FRAME_BYTES` (the per-op knob is the point — 512 KiB
  stays the default for most things); a length-prefixed rewrite of the wire (the newline framing + drain-
  realign is sufficient); silent chunking inside the transport (fragmentation is the *client's* tooling,
  explicit, AWS-shaped).

## Open cruxes (tracked)

- **CRUX-1** — the discovery accessor shape (synthesized constant vs surface budget-table). Resolve at the
  fragment-tooling strike (its first consumer).
- **CRUX-2** — request-fragment sizing (exact encode-length vs estimate). Lean: exact.
- **CRUX-3** — the stable deduction-ordering key for output-cursor resume. Resolve at the output-streaming
  strike.

*Realization-shaped (the AWS I/O contract, forced out by the arena) — the song is the builder's to hand.*
