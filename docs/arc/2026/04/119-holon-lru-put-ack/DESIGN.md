# Arc 119 — Pattern B cache services: batch-oriented protocol fix

**Status:** locked 2026-05-01 (post-three-gaze-passes). Both
service crates revise to a symmetric batch protocol. Substrate
work proceeding.

**Promoted to substrate-wide convention 2026-05-01.** What arc
119 fixes is not a one-arc bug; the pattern is now codified in
`docs/CONVENTIONS.md` § "Batch convention":

> Every wat-rs-shipped service exposes only batch-oriented
> `get` / `put` interfaces. Console is the single exception.

Five substrate services exist; three (Telemetry, Telemetry-
Sqlite, Console) already obey or are exempt; arc 119 brings
the remaining two (LRU + HolonLRU) in line. After arc 119
ships, the convention is **substrate-uniform** — every
non-Console service takes batches.

See `docs/ZERO-MUTEX.md` § "Batch granularity = lock
granularity" for why the convention falls out of mini-TCP's
geometry, and `docs/SERVICE-PROGRAMS.md` § "Reply shapes"
for the two-reply-shape table that supersedes the
fire-and-forget `Push` row.

## Provenance

Surfaced 2026-05-01 mid-arc-109 K.holon-lru anchor work as a
small "Put needs an ack-tx" finding. Through three rounds of
gaze + user direction the scope grew into something arc 109
was meant to surface — a substrate-wide protocol asymmetry
between two cache services that should be identical.

The path:

1. Original gaze (during K.holon-lru first-pass) called
   HologramCacheService's "fire-and-forget" Put acceptable.
2. I drafted a comment codifying that as intentional.
3. User caught the lie pointing at `docs/ZERO-MUTEX.md`:

   > the client cannot continue until the server confirms...
   > both directions are lock step

4. ZERO-MUTEX § "When this is the right shape" makes the
   discipline universal: durability boundaries (commit acked;
   bytes written to fd; transaction sealed) need both
   directions. A cache Put IS a durability boundary.
5. Second-pass gaze locked variant-scoped `PutAckTx` family
   (Pattern A unit-ack) over a unified Reply enum.
6. User then noticed: **LRU CacheService and HologramCacheService
   should be basically identical in surface** — they're both
   "Pattern B services" doing the same job with different
   backends. Today they have divergent shapes (LRU has a tagged-
   tuple Body with unified Reply; HolonLRU has an enum Request
   with no Put ack at all).
7. User clarified the desired protocol:

   > if i want to get something from the cache i submit a
   > get-batch (could be a vec of 1 thing) and i block until
   > the server returns the batch lookup (each item could be
   > missing, all None)
   >
   > if i want to put something into the cache i submit a
   > put-match (could be a vec of 1 thing) and i block until
   > the server returns that the put was successful - empty
   > response..
   >
   > so get N items -> Vec<Option<Item>>
   > and put N items -> Unit

8. User framed the substrate intent:

   > the cache services are implementing a mutex by protecting
   > their shared mutable state through an io select loop who
   > acts like an rpc server

9. Third-pass gaze locked the names against the batch protocol
   covering both services.

## What's wrong today

**Two services that should be identical aren't.**

`:wat::holon::lru::HologramCacheService` (today, single-item):
```scheme
(enum Request
  (Get  (probe :HolonAST) (reply-tx :GetReplyTx))
  (Put  (key :HolonAST) (val :HolonAST)))   ;; NO reply path
```

`:wat::lru::*` (today, post-K.lru, single-item):
```scheme
(typealias Body<K,V> :(i64, K, Option<V>))      ;; tagged tuple
(typealias ReplyTx<V> :Sender<Option<V>>)       ;; UNIFIED return
(typealias Request<K,V> :(Body<K,V>, ReplyTx<V>))
;; tag 0 = GET, tag 1 = PUT (put-val carries Some); reply is Option<V>
```

Two distinct violations:

1. **HolonLRU's Put violates ZERO-MUTEX § Mini-TCP** — fire-and-
   forget; client returns before durability boundary. Cache
   subsequently racing the Put may miss.
2. **The two services diverge in surface shape** — different
   request typealias families, different reply mechanics. Same
   protocol concept (Pattern B cache), two different surfaces.
   A user who's read one cache crate has to relearn the other.

## The fix — symmetric batch protocol

Both services adopt the same shape. Singletons become
batches-of-one. Get is data-bearing; Put is unit-ack release.

### Locked typealiases

```scheme
;; Variant-scoped Pattern B (Get) + Pattern A (Put)
:wat::lru::GetReplyTx<V>          = Sender<Vec<Option<V>>>
:wat::lru::GetReplyRx<V>
:wat::lru::GetReplyChannel<V>

:wat::lru::PutAckTx               = Sender<unit>
:wat::lru::PutAckRx
:wat::lru::PutAckChannel

;; Entry — the batch-element name (gaze-picked)
:wat::lru::Entry<K,V>             = (K, V)

;; Request — enum-based; Body<K,V> retires
(enum Request<K,V>
  (Get  (probes  :Vec<K>)
        (reply-tx :GetReplyTx<V>))
  (Put  (entries :Vec<Entry<K,V>>)
        (ack-tx   :PutAckTx)))

;; Per-client client/server channel halves (already in place)
:wat::lru::ReqTx<K,V>
:wat::lru::ReqRx<K,V>
:wat::lru::ReqChannel<K,V>
```

HolonLRU mirrors with concrete HolonAST types (`K = V = HolonAST`):

```scheme
:wat::holon::lru::GetReplyTx        = Sender<Vec<Option<HolonAST>>>
:wat::holon::lru::PutAckTx          = Sender<unit>
:wat::holon::lru::Entry             = (HolonAST, HolonAST)
;; enum Request {Get(Vec<HolonAST>, GetReplyTx) | Put(Vec<Entry>, PutAckTx)}
```

### Verbs

```
get N items -> Vec<Option<Item>>          ;; (get probes)
put N items -> unit                        ;; (put entries)
```

Bare `get` / `put` (gaze-locked Q1). Argument type carries the
plurality; matches the substrate's unmarked-verb convention
(`:wat::core::map`, `:wat::core::if`). A singleton lookup is
`(get [probe])` — a batch-of-one.

### Why this matches the substrate's mutex-via-RPC framing

Per ZERO-MUTEX § Mini-TCP the cache service IS a mutex
implementation:
- Shared mutable state (the cache map) lives in one program.
- Clients send batches; the io::select loop serializes every
  batch sequentially.
- Lock-step: client send blocks until driver recv; driver
  reply/ack blocks until client recv. Bounded(1) on both ends.
- The "lock" is the loop body; the "release" is the reply
  (Get) or ack (Put) send.

Batch granularity == lock granularity. A single batch holds
the cache's "lock" for one Vec<Op> worth of work. This is
exactly the durability-boundary pattern ZERO-MUTEX § "When
this is the right shape" describes.

## Gaze verdicts (consolidated from three passes)

### Pass 1 — variant-scoped vs unified Reply

Variant-scoped wins. INVENTORY § K's `Ack*` (unit) vs `Reply*`
(data) distinction is load-bearing; unified Reply enum forces
half its variants to be payload-less and lie about the family
they belong to.

### Pass 2 — `PutAck*` vs `PutReply*`

`PutAck*` wins. Substrate's load-bearing rule names families
by what the back-edge CARRIES (unit vs data), not which verb
owns it. Put's back-edge is `Sender<unit>` → joins the Ack
family alongside Telemetry / Console.

### Pass 3 — verb names + request shape + element name

- **Q1 — verb names**: bare `get` / `put`. Substrate has no
  `-batch` / `-many` suffix convention; introducing one here
  would Level 2 mumble against the rest of the surface.
- **Q2 — request shape**: enum-based. LRU's tagged-tuple
  `Body<K,V>` was honest pre-batch (one envelope, one tag bit
  to distinguish); under batch-asymmetric returns
  (`Vec<Option<V>>` for Get, `unit` for Put) the tag-tuple
  forces the cold reader to ask "why does PUT carry an
  Option<V> field that's always Some?" — Level 2 mumble.
  Body retires; enum becomes the canonical Pattern B
  Request shape.
- **Q3 — batch element name**: `Entry<K,V>`. Standard cache-
  domain word (Java `Map.Entry`, Rust HashMap entry API);
  unambiguous. `Item` is generic-mumble, `Pair` collides with
  channel-pair vocabulary, `Slot` collides with cache-internal
  storage.
- **Q4 — service spawn**: confirm `:wat::lru::spawn` /
  `:wat::holon::lru::spawn`. Already settled by K.lru /
  K.holon-lru flatten.

### What gaze surfaced as Level 1 lie

Earlier draft DESIGN said "LRU is Pattern B with a unified
return type" as a comparison row. **True today; false post-119.**
This rewrite removes that paragraph. The substrate's LRU stops
being unified-return when arc 119 lands.

## Substrate work scope

### Both service crates revise

#### `:wat::lru::*` (LRU CacheService)

- **Retire**: `:wat::lru::Body<K,V>` typealias.
- **Mint**: `:wat::lru::Entry<K,V>`, `:wat::lru::PutAckTx`,
  `:wat::lru::PutAckRx`, `:wat::lru::PutAckChannel`.
- **Reshape**: `:wat::lru::Request<K,V>` from
  `(Body<K,V>, ReplyTx<V>)` to enum-based with batch fields.
- **Reshape**: `:wat::lru::ReplyTx<V>` body — `Sender<Option<V>>`
  → `Sender<Vec<Option<V>>>` (batch return). Rename in flight:
  arc 119 keeps the K.lru-shipped name `ReplyTx` but its body
  type widens; this matches Pattern B (data-bearing back-edge)
  unchanged.
- **Driver**: `:wat::lru::handle` switches from per-request
  per-item dispatch to per-request batch dispatch. Loop reads a
  Request; if Get, processes the probe vec, sends
  `Vec<Option<V>>` reply; if Put, processes the entry vec,
  sends `()` ack.
- **Verb signatures**: `:wat::lru::get` and `:wat::lru::put`
  take/return Vec types. Single-item callers wrap in
  batches-of-one.
- **Wat-tests + lab consumers**: every existing call updates to
  the batch shape.

#### `:wat::holon::lru::*` (HologramCacheService — pre-K.holon-lru-flatten)

- **Mint**: `PutAckTx`, `PutAckRx`, `PutAckChannel`, `Entry`.
- **Reshape**: `Request` enum — Get carries `(probes :Vec<HolonAST>)`
  and `reply-tx :GetReplyTx`; Put carries
  `(entries :Vec<Entry>)` and `ack-tx :PutAckTx`.
- **Reshape**: `GetReplyTx` body — `Sender<Option<HolonAST>>` →
  `Sender<Vec<Option<HolonAST>>>` (batch return).
- **Driver**: Per Get dispatch, iterate probes, look up each
  via `HologramCache/get`, collect into `Vec<Option<HolonAST>>`,
  send on reply-tx. Per Put dispatch, iterate entries, call
  `HologramCache/put` for each, send `()` on ack-tx after
  whole batch persisted.
- **Verb signatures**: `(get probes)` and `(put entries)` —
  bare verbs over Vec arguments.
- **Wat-tests + lab consumers**: same batch-shape sweep as LRU.

### Shared

- The naming families (`GetReplyTx`/`Rx`/`Channel`,
  `PutAckTx`/`Rx`/`Channel`, `Entry`) carry through K.holon-lru
  unchanged when that slice flattens (HologramCacheService::* →
  :wat::holon::lru::*).
- arc 117's scope-deadlock walker recognizes `Sender` /
  `Channel` shapes; it doesn't care about the inner payload
  type, so it continues to work without modification when
  ReplyTx's body widens to `Vec<Option<...>>`.

## Estimated scope

- LRU substrate file rewrite: ~50-100 lines diff
  (typealias retire, enum mint, driver loop reshape, verb
  signatures).
- HolonLRU substrate file rewrite: ~similar.
- Wat-tests sweep across both crates: ~30-50 sites total;
  each call site changes shape (singleton `(get k)` →
  `(get [k])`, etc.).
- Lab consumers: probably ~10-20 sites in the trading lab
  using LRU's cache; trading lab may not use HolonLRU
  (substrate-only).

Substantial. Mechanical-but-judgment-driven (the singleton-to-
batch-of-1 shift requires call-site editing, not pure sed). Plan:
substrate first (both files), probe-verify, then sonnet sweep
under substrate-as-teacher with diagnostic stream as the brief.

## Sequencing

1. **Arc 119** — protocol fix in BOTH services (this arc).
2. **K.holon-lru** (109 slice) — naming flatten only;
   HologramCacheService:: prefix retires; the locked names
   from arc 119 (Entry, GetReplyTx, PutAckTx etc.) flatten
   along with everything else.
3. **K.thread-process** (109 slice) — last K-slice.
4. **arc 109 INSCRIPTION** — closes the arc.

After 119 + 109: both cache services are disciplinarily correct,
canonically named, and substrate-symmetric. The mutex-via-RPC
framing is honest at every layer (wire shape, naming, doctrine
docs).

## Execution checklist (compaction-amnesia-resistant)

Read this section first if picking up arc 119 mid-flight. The
runtime task list (#202–#209) holds the same items but does not
survive a fresh session; this checklist does. **Steps run
sequentially** — each depends on the prior.

| # | Step | Status |
|---|---|---|
| 1 | Revert HolonLRU substrate WIP — `git checkout HEAD -- crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`. Discards stale partial single-item Put+ack-tx work that doesn't match this DESIGN. Returns substrate to canonical pre-119 state. | pending |
| 2 | Reshape `:wat::lru::*` substrate file. Retire `Body<K,V>`; mint `Entry<K,V> = (K,V)`; mint `PutAckTx/Rx/Channel` (`Sender<unit>`); reshape `Request<K,V>` from `(Body, ReplyTx)` tagged-tuple to enum `{Get(probes:Vec<K>, reply-tx:GetReplyTx<V>) \| Put(entries:Vec<Entry<K,V>>, ack-tx:PutAckTx)}`; widen `ReplyTx<V>` body from `Sender<Option<V>>` to `Sender<Vec<Option<V>>>`. Variant-scoped names: `GetReplyTx` for data-back, `PutAckTx` for unit-ack. | pending |
| 3 | Reshape `:wat::lru::*` driver loop + verbs. Driver: Get iterates probes, builds `Vec<Option<V>>`, sends on reply-tx; Put iterates entries, mutates state, sends `()` on ack-tx after batch persisted. Verbs `:wat::lru::get` / `:wat::lru::put` take `Vec` types; single-item callers wrap in batches-of-one. | pending |
| 4 | Reshape `:wat::holon::lru::HologramCacheService` surface. Mint `PutAckTx/Rx/Channel` + `Entry` typealiases under the `HologramCacheService::` prefix (K.holon-lru flattens them later); widen `GetReplyTx` body to `Sender<Vec<Option<HolonAST>>>`; reshape Request enum to batch-Get + batch-Put variants matching LRU's surface (with K=V=HolonAST). | pending |
| 5 | Reshape HolonLRU driver loop + verbs. Per Get dispatch: iterate probes, look up via `HologramCache/get`, collect into `Vec<Option<HolonAST>>`, send on reply-tx. Per Put dispatch: iterate entries, call `HologramCache/put` for each, send `()` on ack-tx after batch persisted. Verbs `(get probes)` / `(put entries)` take Vec args. | pending |
| 6 | `cargo test --release --workspace` baseline + diagnostic capture. Expect baseline pre-sweep failures localized to call-site shape mismatches in wat-tests + lab consumers. Confirm no Rust-level regressions (parser/walker/dispatch). Capture substrate-as-teacher diagnostic stream as the brief for step 7. | pending |
| 7 | **Discipline correction** (NOT just a mechanical rewrite — see "Realization" below). HolonLRU's wat-tests test the wrong layer; rewrite them to call the consumer surface (the helper verbs minted in step 4). LRU's single test channel-splits + batch-of-ones the existing helper-verb call. Scope is wat-rs only — lab consumers (`holon-lab-trading/`) are downstream, separate workspace, separate arc. 5 failing wat-tests at step-6 baseline: 1 in `crates/wat-lru/wat-tests/lru/CacheService.wat` + 4 in `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`. The HolonLRU 4 are raw-protocol tests that hand-build `Request::Put`/`Get` constructors and call `:wat::kernel::send`/`recv` directly — they speak for the implementer, not the consumer. Convergence: both crates' tests look the same shape post-step-7 — spawn, pop req-tx, allocate per-test channels, call helper verb in batch-of-one form. **Orchestrator verifies via `git diff --stat`** against agent reports per the trust-but-verify protocol. | pending |
| 8 | Closure: arc 119 INSCRIPTION + 058 changelog row + arc 109 INVENTORY § K mark. Note: substrate-uniform batch convention now holds (every non-Console service obeys `CONVENTIONS.md` § "Batch convention"). K.holon-lru becomes unblocked for the naming flatten. | pending |

## Realization (surfaced 2026-05-01)

Step 7 was originally framed as "wrap singletons in batch-of-one
+ thread the new ack-tx." Mechanically true. Conceptually wrong.

When the substrate-as-teacher diagnostic stream reported the
HolonLRU wat-tests failing, those tests turned out to be calling
raw `(:wat::kernel::send tx (Request::Put k v))` directly —
hand-building the Request enum constructor and walking the
`Result<Option<T>, ThreadDiedError>` chain manually. They were
"raw protocol tests."

Compare LRU's single wat-test, which calls `(:wat::lru::put
req-tx reply-tx reply-rx key val)` — the helper verb. A consumer
of the cache calls the helper verb. Nobody but a substrate
implementer hand-builds the Request enum.

The HolonLRU tests had been written before HolonLRU shipped any
helper verbs — there was nothing higher-level to call. That
historical accident left them at the wrong vantage: they verified
the wire protocol from the implementer's perspective, but they
lived in a consumer crate's `wat-tests/` directory where their
audience is the consumer.

User direction (2026-05-01):

> all of our code should be measurable from the caller's
> perspective.. that's the interface to confirm via

Codified as `CONVENTIONS.md` § "Caller-perspective verification"
and policed by the `/vocare` ward. Step 7 stops being "wrap
singletons" and becomes "rewrite the wat-tests at the right
vantage" — preserving test scenarios (multi-client, eviction,
counted-recv) while moving the call shape to helper verbs.

Wire-protocol pedagogy retains its home in
`wat-rs/wat-tests/service-template.wat` — that file's caller IS
the service implementer.

**Discipline anchors** (apply to every step):

- **No broken commits.** Each step lands on a green workspace
  (`cargo test --release --workspace` passes). Step 6's failures
  are pre-sweep expected; step 7 closes them.
- **Push on commit.** Every commit pushes to origin/main.
- **Trust but verify agent reports.** Orchestrator runs
  `git diff --stat` after every sonnet sweep and matches against
  the agent's claimed file list before commit.
- **Substrate-as-teacher diagnostic stream is the brief.** Step
  7's sweep agent receives `cargo test` diagnostic output, not a
  hand-written rename map; the substrate teaches what to fix.
- **Compaction-amnesia mitigation.** This DESIGN.md is the
  durable record. If context dies mid-arc, the next session
  reads this checklist and the locked plan above; nothing else
  is required to resume.

## Cross-references

- `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels" — the
  doctrine this arc enforces.
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § K channel-
  naming-patterns subsection — Pattern A (Ack) vs Pattern B
  (Reply) vocabulary.
- `docs/arc/2026/04/109-kill-std/SLICE-K-HOLON-LRU.md` — queued
  naming flatten that lands AFTER this arc.
- `docs/arc/2026/04/109-kill-std/SLICE-K-LRU.md` — already-
  shipped slice; arc 119 does NOT undo K.lru's naming changes,
  it reshapes LRU's protocol body.
- `crates/wat-lru/wat/lru/CacheService.wat` — substrate file.
- `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`
  — substrate file.

## What 109 surfaced

User direction (2026-05-01) on why arc 119 exists:

> this the kind of thing 109 is meant to surface - get them
> fixed

109's nominal mission was naming + filesystem cleanup. In
practice it surfaced a substrate-discipline gap (HolonLRU's
fire-and-forget Put), a substrate-symmetry gap (LRU and
HolonLRU diverging surfaces despite doing the same job), and a
substrate-protocol-completeness gap (single-item granularity
when batches are the natural unit). All three are
naming-adjacent — incorrect surfaces become incorrect names —
and 109 found them by asking the four questions of the
existing service crates and following the gaze findings.

Arc 119 is the protocol fix that follows from the naming
clarity 109 forced.
