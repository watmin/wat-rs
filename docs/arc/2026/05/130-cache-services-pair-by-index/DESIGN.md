# Arc 130 — Cache services adopt pair-by-index via HandlePool

**Status:** drafted 2026-05-01.

## TL;DR

The `:wat::lru::*` and `:wat::holon::lru::*` services currently
require callers to allocate per-call ack/reply channels and pass
both halves to helper verbs. Arc 126 surfaced this as the
channel-pair-deadlock anti-pattern; the substrate's helper-verb
signatures THEMSELVES require the deadlock pattern at every call
site. The 6 deadlock-class tests `:should-panic`'d to validate
arc 126's check; that's a test scaffold, not a resolution.

This arc adopts Console's existing pair-by-index discipline:
the service `spawn` pre-allocates N reply channels at startup,
paired by HandlePool index with the request channels. Clients
pop one Handle = `(ReqTx, ReplyRx)` — both ends of TWO distinct
channels, not both halves of one. The Request enum stops
carrying embedded reply channels; a unified `Reply<V>` enum
unifies Get's `Vec<Option<V>>` and Put's `unit` so both verbs
share one reply channel per slot.

After arc 130: arc 126's check stops firing on cache-service
consumers (no per-call channel allocation), the 6 `:should-panic`
annotations retire (the tests pass without panic), arc 119 closes
with full structural correctness, and arc 109's K.holon-lru +
K.thread-process slices unblock.

## Provenance

Surfaced via the failure-engineering chain that landed today
(2026-05-01):

| # | Arc | Surfaced | Resolution |
|---|---|---|---|
| 1 | arc 119 | HolonLRU's fire-and-forget Put | Arc 119 reshape to mini-TCP |
| 2 | arc 124 | proc-macro scanner gap | Discovered the 6 hanging tests |
| 3 | arc 126 | channel-pair-deadlock anti-pattern | Compile-time check (slice 1) |
| 4 | arc 128 | check walker descends into sandbox forms | Boundary guard |
| 5 | arc 129 | `:time-limit` swallows panic substrings | recv_timeout split |
| 6 | **arc 130 (this arc)** | **helper-verb signatures REQUIRE the anti-pattern** | **Pair-by-index via HandlePool** |

Arc 126's diagnostic message names the two canonical fix shapes
("Pair-by-index via HandlePool" / "Embedded reply-tx in payload").
The substrate's own Console service uses pair-by-index. The
cache services were the only substrate-shipped services NOT
following the pattern; arc 130 brings them in line.

## What's wrong today

`crates/wat-lru/wat/lru/CacheService.wat` (mirrors in HolonLRU):

```scheme
;; ReplyTx still per-verb-shape (Get returns Vec<Option<V>>):
(:wat::core::typealias :wat::lru::ReplyTx<V>
  :wat::kernel::Sender<wat::core::Vector<wat::core::Option<V>>>)

;; Put has its own ack channel (carries unit):
(:wat::core::typealias :wat::lru::PutAckTx
  :wat::kernel::Sender<wat::core::unit>)

;; Request enum embeds the per-call reply/ack-tx:
(:wat::core::enum :wat::lru::Request<K,V>
  (Get  (probes  :wat::core::Vector<K>)
        (reply-tx :wat::lru::ReplyTx<V>))
  (Put  (entries :wat::core::Vector<wat::lru::Entry<K,V>>)
        (ack-tx   :wat::lru::PutAckTx)))

;; Spawn returns HandlePool of just ReqTx — caller has to make
;; the reply/ack channel themselves:
(:wat::core::typealias :wat::lru::Spawn<K,V>
  :(wat::kernel::HandlePool<wat::lru::ReqTx<K,V>>,
    wat::kernel::Thread<wat::core::unit,wat::core::unit>))

;; Helper verbs take (req-tx, reply-tx, reply-rx, ...) — caller
;; must hold ALL of these in scope (the channel-pair-deadlock
;; anti-pattern):
(:wat::lru::get<K,V>
  (req-tx :wat::lru::ReqTx<K,V>)
  (reply-tx :wat::lru::ReplyTx<V>)
  (reply-rx :wat::lru::ReplyRx<V>)
  (probes :wat::core::Vector<K>)
  -> :wat::core::Vector<wat::core::Option<V>>)

(:wat::lru::put<K,V>
  (req-tx :wat::lru::ReqTx<K,V>)
  (ack-tx :wat::lru::PutAckTx)
  (ack-rx :wat::lru::PutAckRx)
  (entries :wat::core::Vector<wat::lru::Entry<K,V>>)
  -> :wat::core::unit)
```

The caller's let* must bind `(reply-pair, reply-tx, reply-rx)` and/or
`(ack-pair, ack-tx, ack-rx)`. Both halves of the same channel
sit in the same scope. Arc 126's check fires.

## The fix

Adopt Console's pair-by-index pattern. The service's
`spawn` pre-allocates N reply channels paired with the N
request channels. The HandlePool entry IS the (req-tx,
reply-rx) pair; the matching (req-rx, reply-tx) lives in the
driver's selection vector.

### New typealiases

```scheme
;; Unified Reply enum — replaces per-verb reply types.
;; Get's Vec<Option<V>> and Put's unit ack flow on ONE channel:
(:wat::core::enum :wat::lru::Reply<V>
  (GetResult (results :wat::core::Vector<wat::core::Option<V>>))
  (PutAck))

;; Reply channel typealiases:
(:wat::core::typealias :wat::lru::ReplyTx<V>
  :wat::kernel::Sender<wat::lru::Reply<V>>)
(:wat::core::typealias :wat::lru::ReplyRx<V>
  :wat::kernel::Receiver<wat::lru::Reply<V>>)
(:wat::core::typealias :wat::lru::ReplyChannel<V>
  :(wat::lru::ReplyTx<V>,wat::lru::ReplyRx<V>))

;; Handle = client's view of one slot = paired (ReqTx, ReplyRx).
;; Mirrors Console::Handle = (Tx, AckRx).
(:wat::core::typealias :wat::lru::Handle<K,V>
  :(wat::lru::ReqTx<K,V>,wat::lru::ReplyRx<V>))

;; Driver's view of one slot = paired (ReqRx, ReplyTx).
;; Mirrors Console::DriverPair = (Rx, AckTx).
(:wat::core::typealias :wat::lru::DriverPair<K,V>
  :(wat::lru::ReqRx<K,V>,wat::lru::ReplyTx<V>))

;; Request enum — no longer carries embedded channels:
(:wat::core::enum :wat::lru::Request<K,V>
  (Get  (probes  :wat::core::Vector<K>))
  (Put  (entries :wat::core::Vector<wat::lru::Entry<K,V>>)))

;; Spawn return: HandlePool of paired Handle.
;; Was: HandlePool<ReqTx<K,V>>
;; Now: HandlePool<Handle<K,V>>
(:wat::core::typealias :wat::lru::Spawn<K,V>
  :(wat::kernel::HandlePool<wat::lru::Handle<K,V>>,
    wat::kernel::Thread<wat::core::unit,wat::core::unit>))
```

### Retired typealiases

The old per-verb reply/ack channel typealiases retire:

- `:wat::lru::PutAckTx`, `:PutAckRx`, `:PutAckChannel`
- The bare-Sender-shape `:ReplyTx<V>` (replaced; same name,
  different body — now wraps `Reply<V>` enum)

The new `:Handle<K,V>` and `:DriverPair<K,V>` are the substrate's
opaque carriers; users never construct them by hand.

### New helper-verb signatures

```scheme
;; Get takes ONLY the Handle. Internally projects ReqTx + ReplyRx
;; from the tuple, sends Request::Get, recvs Reply, matches on the
;; GetResult variant.
(:wat::lru::get<K,V>
  (handle :wat::lru::Handle<K,V>)
  (probes :wat::core::Vector<K>)
  -> :wat::core::Vector<wat::core::Option<V>>)

;; Put takes ONLY the Handle. Internally similar; matches Reply
;; on PutAck variant; returns unit.
(:wat::lru::put<K,V>
  (handle :wat::lru::Handle<K,V>)
  (entries :wat::core::Vector<wat::lru::Entry<K,V>>)
  -> :wat::core::unit)
```

### Helper-verb body

```scheme
(:wat::core::define
  (:wat::lru::get<K,V>
    (handle :wat::lru::Handle<K,V>)
    (probes :wat::core::Vector<K>)
    -> :wat::core::Vector<wat::core::Option<V>>)
  (:wat::core::let*
    (((req-tx :wat::lru::ReqTx<K,V>)
      (:wat::core::first handle))
     ((reply-rx :wat::lru::ReplyRx<V>)
      (:wat::core::second handle))
     ;; Send Request::Get; option::expect the disconnect path.
     ((_ :wat::core::unit)
      (:wat::core::Result/expect -> :wat::core::unit
        (:wat::kernel::send req-tx (:wat::lru::Request::Get probes))
        "lru/get: req-tx disconnected"))
     ;; Recv Reply; option::expect the disconnect path.
     ((reply :wat::lru::Reply<V>)
      (:wat::core::Option/expect -> :wat::lru::Reply<V>
        (:wat::core::Result/expect -> :wat::core::Option<wat::lru::Reply<V>>
          (:wat::kernel::recv reply-rx)
          "lru/get: reply-rx disconnected")
        "lru/get: reply-rx returned None")))
    ;; Match on the GetResult variant. PutAck shouldn't appear in
    ;; response to a Get; defensive Result/expect via match-with-fallback
    ;; or a panic.
    (:wat::core::match reply -> :wat::core::Vector<wat::core::Option<V>>
      ((:wat::lru::Reply::GetResult results) results)
      (:wat::lru::Reply::PutAck
        ;; Should not happen — driver replied with PutAck on a Get.
        ;; Panic with a clear message; the discipline is broken if so.
        (:wat::core::panic! "lru/get: driver sent PutAck on Get reply channel")))))
```

Arc 126's check at the call site:

- `(first handle)` traces to `handle`. `handle`'s RHS is
  `(HandlePool::pop pool)` — NOT a `make-bounded-channel`.
  Trace gives up cleanly (returns None).
- Same for `(second handle)`.
- Two args (`req-tx`, `reply-rx`) classified as Sender + Receiver
  kinds, but neither has a pair-anchor → no two-args-share-anchor
  → no fire.

### Driver loop

The driver's `Vec<DriverPair<K,V>>` holds the matching (ReqRx,
ReplyTx) pairs by index. Select fires; index → DriverPair → recv
Request → match → handle work → send Reply on the matching
ReplyTx.

```scheme
(:wat::core::define
  (:wat::lru::handle<K,V,G>
    (req :wat::lru::Request<K,V>)
    (reply-tx :wat::lru::ReplyTx<V>)
    (state :wat::lru::State<K,V,G>)
    -> :wat::lru::State<K,V,G>)
  (:wat::core::match req -> :wat::lru::State<K,V,G>
    ((:wat::lru::Request::Get probes)
      ;; ... process probes; build Vec<Option<V>> ...
      ;; send Reply::GetResult on reply-tx
      ;; return updated state
      ...)
    ((:wat::lru::Request::Put entries)
      ;; ... process entries; mutate cache state ...
      ;; send Reply::PutAck on reply-tx
      ;; return updated state
      ...)))
```

### Caller surface

```scheme
;; Pre-arc-130:
(:wat::core::let*
  (((spawn :wat::lru::Spawn<K,V>) (:wat::lru::spawn ...))
   ((pool ...) (:wat::core::first spawn))
   ((req-tx :wat::lru::ReqTx<K,V>) (:wat::kernel::HandlePool::pop pool))
   ((reply-pair :wat::lru::ReplyChannel<V>)
    (:wat::kernel::make-bounded-channel ... 1))    ;; ARC 126 FIRES
   ((reply-tx :wat::lru::ReplyTx<V>) (:wat::core::first reply-pair))
   ((reply-rx :wat::lru::ReplyRx<V>) (:wat::core::second reply-pair))
   ((results ...) (:wat::lru::get req-tx reply-tx reply-rx probes)))
  ...)

;; Post-arc-130:
(:wat::core::let*
  (((spawn :wat::lru::Spawn<K,V>) (:wat::lru::spawn ...))
   ((pool ...) (:wat::core::first spawn))
   ((handle :wat::lru::Handle<K,V>) (:wat::kernel::HandlePool::pop pool))
   ((results ...) (:wat::lru::get handle probes)))    ;; clean
  ...)
```

The caller never allocates `make-bounded-channel`. The channels
are owned by the service spawn; HandlePool::pop returns an
opaque tuple-typed Handle.

## Why a unified Reply enum

Two reply types (`Vec<Option<V>>` for Get, `unit` for Put)
could in principle stay as separate channels — but that would
require two HandlePool entries per slot (one for ReplyRx<Vec<...>>,
one for AckRx<unit>), doubling the pool size. The unified
Reply<V> enum is simpler:

- ONE HandlePool<Handle<K,V>> per spawn
- ONE reply channel per slot, carrying any reply variant
- Driver matches on Request, sends matching Reply variant
- Helper verb matches on Reply variant, returns the relevant data

The ReplyTx<V>'s body widens from `Sender<Vec<Option<V>>>` to
`Sender<Reply<V>>` — the type still parametrizes on V (Get's
result type), but the channel carries the enum.

A future arc could split per-verb channels for performance reasons
(no enum dispatch); for now, the simplicity wins.

## What this arc closes

- The 6 `:should-panic` annotations on the deadlock-class
  tests retire. Tests rewrite to use the new helper-verb shape;
  they PASS without panic; arc 126's check doesn't fire on
  the new shape.
- Arc 119's pending closure question: helper-verb signatures
  redesign happens (option b); arc 119 closes with full
  structural correctness.
- Arc 109's blocked K-slices unblock: K.holon-lru's namespace
  flatten can proceed once arc 130 ships.
- The "substrate-shipped services obey the pair-by-index
  discipline" claim becomes universally true (Console + LRU +
  HolonLRU all follow the same pattern; Telemetry is a separate
  shape since it doesn't have a per-verb reply pattern).

## Limitations / non-goals

- **Telemetry stays as-is.** `:wat::telemetry::*` is fire-and-go
  (per arc 089/091/096); it doesn't have per-call replies. Arc
  130 only touches the two cache crates.
- **Lab consumers are not touched.** Per memory's lab-archive
  state, substrate work doesn't wait for lab. The reconstruction
  inherits the new shape.
- **No new compile-time check.** Arc 130 is a substrate reshape
  to AVOID arc 126's existing check; it doesn't add a new
  rule.
- **No migration shim.** The pre-arc-130 helper-verb signatures
  retire; consumers must update. Per "no v1/v2 for proposals"
  + "no broken commits" — workspace stays green via simultaneous
  reshape of substrate + tests.

## Implementation plan

### Slice 1 — `:wat::lru::*` substrate + tests

1. **Substrate (`crates/wat-lru/wat/lru/CacheService.wat`):**
   - Add `Reply<V>` enum, `ReplyTx<V>`, `ReplyRx<V>`,
     `ReplyChannel<V>`, `Handle<K,V>`, `DriverPair<K,V>`
     typealiases.
   - Reshape `Request<K,V>` enum (drop embedded channels).
   - Reshape `Spawn<K,V>` to return
     `HandlePool<Handle<K,V>>`.
   - Reshape `:wat::lru::spawn` body to allocate N reply
     channels, pair with request channels, populate
     HandlePool with paired Handle entries, hand
     DriverPair<...> vector to the driver.
   - Reshape `:wat::lru::handle` (driver's per-request
     dispatcher): now takes `(req, reply-tx, state)`; matches
     Request variant; sends matching Reply variant on
     reply-tx.
   - Reshape `:wat::lru::loop`-step / `:wat::lru::run` to
     thread the DriverPair vector.
   - Reshape `:wat::lru::get` and `:wat::lru::put` helper-verb
     signatures: take `Handle<K,V>` instead of channel triples.
     Bodies project Handle → send Request → recv Reply → match.
   - Retire: `PutAckTx`, `PutAckRx`, `PutAckChannel`.

2. **Tests (`crates/wat-lru/wat-tests/lru/CacheService.wat`):**
   - Rewrite `test-cache-service-put-then-get-round-trip` to
     use the new shape. The test pops a Handle, calls /put with
     handle + entries, then /get with handle + probes.
   - Retire the `:should-panic("channel-pair-deadlock")` and
     `:time-limit "200ms"` annotations on this test — the test
     PASSES without panic in the new shape.

3. **Verification:** `cargo test --release -p wat-lru`. Single
   test reports `... ok`, NOT `... ok (should panic)`.

### Slice 2 — `:wat::holon::lru::*` substrate + tests

Mirrors slice 1 for HologramCacheService. The HolonAST-typed
specialization (`K = V = HolonAST`) follows the same pattern.

1. **Substrate (`crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`):**
   - Same set of typealias additions / retirements.
   - Same Spawn / handle / loop / run reshapes.
   - Same helper-verb signature changes for
     `HologramCacheService/get` and `/put`.

2. **Tests
   (`crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`):**
   - Rewrite step3 through step6 to use the new shape.
   - Retire the 4 `:should-panic` + `:time-limit` annotations.

3. **Tests (`crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat`):**
   - Rewrite step-B to use the new shape.
   - Retire its `:should-panic` + `:time-limit` annotations.

4. **Verification:** `cargo test --release -p wat-holon-lru`.
   All 14 tests report `... ok`, no `should panic` markers.

### Slice 3 — closure

INSCRIPTION (this arc) + cross-references from:

- Arc 119's DESIGN/INSCRIPTION: closure noting arc 130
  delivered the helper-verb redesign that arc 119's
  "Realization" surfaced the need for.
- Arc 126's INSCRIPTION queued-follow-ups: marked complete.
- WAT-CHEATSHEET §11 (channel-pair-deadlock rule): note that
  the substrate's cache services exemplify the canonical fix
  (pair-by-index via HandlePool).
- ZERO-MUTEX § "Routing acks": cache-service example updated
  to match new shape.
- 058 changelog row (lab repo, optional given lab's
  archive state).

## Verification — workspace green

After all 3 slices:

- `cargo test --release --workspace` exit=0
- 6 deadlock-class tests pass cleanly (no `should panic`
  marker)
- 1 ignored test stays ignored (wat-sqlite arc-122 mechanism
  test)
- No new substrate failures
- Arc 126's check still passes its 5 unit tests (the rule
  itself is unchanged; it's just no consumer trips it now)

## The four questions

**Obvious?** Yes. Console already does pair-by-index. The cache
services were the only substrate-shipped services NOT following
the pattern. Arc 130 brings them in line with their sibling.

**Simple?** Medium. Real reshape of two crates' substrate +
tests. ~300-500 LOC across both. Not trivial but well-bounded;
mirrors a known-working substrate-shipped pattern (Console).

**Honest?** Yes. The current shape is broken (arc 126 fires on
every legitimate use). The redesign matches the substrate's own
working pattern. The Reply<V> enum unification is a small
honest tax for the simplicity gain.

**Good UX?** Phenomenal. Caller surface collapses from 6 lines
of channel allocation to 1 line of `HandlePool::pop`. Arc 126's
check stops firing. The substrate's three services compose
identically (pop a handle, call helper verbs, no per-call channel
allocation).

## Cross-references

- `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels" + §
  "Routing acks" — the doctrine.
- `wat/console.wat` (or similar) — Console's pair-by-index
  reference implementation.
- `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md` — the arc
  whose "Realization" surfaced the need for this redesign.
- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/INSCRIPTION.md`
  — the structural enforcement that flagged the helper-verb
  signatures as broken; queued-follow-ups names this arc's
  redesign.
- `crates/wat-lru/wat/lru/CacheService.wat` — substrate file
  for slice 1.
- `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`
  — substrate file for slice 2.

## Failure-engineering record

Arc 130 follows the chain that landed today:

| # | Arc | Sweep | Hard rows | Substrate gap |
|---|---|---|---|---|
| 1 | arc 126 | sweep 1 (slice 1) | 5/6 | arc 128 (boundary guard) |
| 2 | arc 126 | sweep 2 (slice 1 reland) | 14/14 | none (clean) |
| 3 | arc 126 | sweep 3 (slice 2) | 6/8 | arc 129 (Timeout vs Disconnected) |
| 4 | arc 129 | sweep 4 (slice 1) | 14/14 | none (clean) |
| 5 | arc 130 | sweep TBD | TBD | arc 119's helper-verb signature gap |

Pattern: each arc ships through DESIGN → BRIEF → EXPECTATIONS →
sonnet sweep → SCORE → INSCRIPTION. The artifacts-as-teaching
discipline propagates across structural-rule arcs (arc 126),
substrate-fix arcs (arc 128, arc 129), AND now across
service-redesign arcs (arc 130). Different layer, same
discipline.
