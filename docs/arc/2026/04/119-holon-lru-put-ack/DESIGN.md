# Arc 119 — HologramCacheService Put ack-tx (mini-TCP discipline correction)

**Status:** locked 2026-05-01 (post-gaze). Option A + `PutAck*`
family per gaze verdict. Substrate work proceeding.

## Provenance

Surfaced 2026-05-01 mid-arc-109 K.holon-lru anchor work. The
original gaze finding had said "Put is fire-and-forget — no
PutReply* types by design"; I drafted a clarifying comment
codifying that as intentional. User caught the lie and pointed
at `docs/ZERO-MUTEX.md`:

> the client connect[cannot] continue until the server confirms...
> both [directions] are lock step

ZERO-MUTEX § "When this is the right shape" is unambiguous:

> Any case where the producer needs to know when the work is
> *done*, not just *queued* — durability boundaries (commit
> acked; bytes written to fd; transaction sealed). Bounded send
> alone gives backpressure on accept; the ack gives backpressure
> on completion. Use both when "done" matters.

A cache `Put` IS a durability boundary. The client wants to know
"the value is in the cache and accessible to subsequent Gets,"
not "my message was queued for processing." The current Put
violates the substrate's own zero-mutex mini-TCP discipline.

## The problem

Today's `:wat::holon::lru::HologramCacheService::Request`:

```scheme
(:wat::core::enum :wat::holon::lru::HologramCacheService::Request
  (Get
    (probe :wat::holon::HolonAST)
    (reply-tx :wat::holon::lru::HologramCacheService::GetReplyTx))
  (Put
    (key :wat::holon::HolonAST)
    (val :wat::holon::HolonAST)))      ;; NO reply-tx — fire-and-forget
```

- **Get** carries a `reply-tx`; client allocates `(GetReplyTx,
  GetReplyRx)`, embeds the tx, blocks on the rx for the result.
  Honest mini-TCP.
- **Put** carries no reply path. Client sends and continues
  before the cache has actually stored the value. Subsequent
  Gets racing the Put may miss. **Discipline violation.**

## Comparison: how the substrate's other services do it right

- **Console** (`:wat::console::*`): every print waits on
  `AckRx` for `()` after the driver writes durably. Pattern A.
- **Telemetry** (`:wat::telemetry::*`): every `batch-log` waits
  on `AckRx` after the dispatcher returns. Pattern A.
- **LRU CacheService** (`:wat::lru::*`): every Get AND every Put
  routes through the same `Body` envelope with an embedded
  `ReplyTx<Option<V>>`. Even Put gets a reply (`:None`) so the
  client unblocks AFTER the cache mutation completes. Pattern B
  with a unified return type.

HolonLRU is the only service in the substrate that does
fire-and-forget. **Outlier — and wrong by the doctrine.**

## The fix

Put gains an ack-tx field. Two shape options:

### Option A — variant-scoped Pattern A flavor for Put

```scheme
(:wat::core::typealias :wat::holon::lru::HologramCacheService::PutAckTx
  :wat::kernel::Sender<wat::core::unit>)
(:wat::core::typealias :wat::holon::lru::HologramCacheService::PutAckRx
  :wat::kernel::Receiver<wat::core::unit>)
(:wat::core::typealias :wat::holon::lru::HologramCacheService::PutAckChannel
  :wat::kernel::Channel<wat::core::unit>)

(:wat::core::enum :wat::holon::lru::HologramCacheService::Request
  (Get  (probe :HolonAST)
        (reply-tx :GetReplyTx))      ;; Pattern B — data-bearing reply
  (Put  (key :HolonAST) (val :HolonAST)
        (ack-tx :PutAckTx)))         ;; Pattern A — unit-ack release
```

**Reads honestly:** Get returns data → "Reply"; Put returns
nothing → "Ack". Variant-scoped naming (`Get*`/`Put*`) makes
the per-verb shape difference visible. Mirrors the substrate's
existing channel-naming patterns A/B taxonomy at the variant
level.

### Option B — unified Reply enum

```scheme
(:wat::core::enum :wat::holon::lru::HologramCacheService::Reply
  (GetResult (value :wat::core::Option<wat::holon::HolonAST>))
  (PutAck))

(:wat::core::typealias :wat::holon::lru::HologramCacheService::ReplyTx
  :wat::kernel::Sender<wat::holon::lru::HologramCacheService::Reply>)

(:wat::core::enum :wat::holon::lru::HologramCacheService::Request
  (Get (probe :HolonAST) (reply-tx :ReplyTx))
  (Put (key :HolonAST) (val :HolonAST) (reply-tx :ReplyTx)))
```

**One reply type both variants share.** Client allocates one
`(ReplyTx, ReplyRx)` per request; matches on the Reply variant
to extract Get's value or Put's ack. Single typealias family
(`ReplyTx` / `ReplyRx` / `ReplyChannel`); no Get/Put-prefixed
asymmetry in the channel typealiases.

### Recommendation

**Option A** unless gaze says otherwise. Reasoning:

- Per-verb reply shapes are honestly different (data vs
  release). Squashing them into a Reply enum forces the client
  to match-and-discard "the wrong arm" on every reply,
  obscuring per-verb intent.
- The variant-scoped `Get*Reply*` naming has been in place; A
  extends the pattern to `Put*Ack*` rather than rewriting both.
- Smaller blast radius: Get's surface is unchanged; only Put
  and its ack family is added.

But this naming question wants gaze before locking in.

## Consumer impact

- **Driver loop** in HologramCacheService.wat: per Put dispatch,
  send `()` on the embedded ack-tx after `HologramCache/put`
  returns.
- **Client call sites**: every `(Put k v)` becomes
  `(Put k v ack-tx)` where the caller pre-allocated
  `(PutAckTx, PutAckRx)` and blocks on the rx after sending.
- **wat-tests + lab consumers**: same shape; agent-sweepable
  once the driver and typealiases land.

Substrate sweep: probably ~20-50 sites across the crate's wat
+ wat-tests. Bigger than a pure rename because it's a real
protocol change — call sites need new bindings.

## Why this is a NEW arc, not arc 109

Arc 109 is naming + filesystem cleanup. Mechanical, doctrine-
driven, sonnet-sweepable.

Arc 119 is **substrate-discipline correction** — the protocol
itself is wrong (violates mini-TCP). Fixing requires changes
to the Request enum's shape, the driver loop's behavior, and
every client call site's allocation pattern. Different kind of
work; deserves its own arc identity.

User direction (2026-05-01):

> new arc - fix the comms - then fix the names

Arc 119 fixes the comms. K.holon-lru (queued in 109) then
fixes the names — landing against a corrected protocol.

## Sequencing

1. **Arc 119** — protocol fix (this arc).
2. **K.holon-lru** (109) — naming flatten + GetReplyPair →
   GetReplyChannel + the new PutAck* family flattens too.

After both: HologramCacheService is disciplinarily correct AND
canonically named. Then K.thread-process is the last K-slice;
then 109's INSCRIPTION can close the arc.

## Gaze verdict (2026-05-01)

**Question 1 — Option A wins.** The substrate already declared
`Ack*` vs `Reply*` load-bearing at the crate level (INVENTORY
§ K); arc 119 propagates that distinction to the variant level
inside one crate. Option B's unified `Reply` family with a
payload-less `(PutAck)` variant would force the cold reader to
trust a name that contradicts half its instances — Level 2
mumble. The cold reader sees `(Put k v ack-tx)` under A and the
field name matches the type body; under B they'd see
`(Put k v reply-tx)` and find the "reply" carries nothing.

**Question 2 — `PutAck*` family.** Honest body wins over surface
symmetry. The substrate's load-bearing rule names families by
what the back-edge *carries*, not by which verb owns it. Put's
back-edge carries `unit`; it joins the Ack* family alongside
Telemetry's and Console's. `PutReply*` would force a
`Sender<unit>` into the Reply* family — same Level 2 mumble.

The locked typealiases (post-K.holon-lru flatten):

```
:wat::holon::lru::PutAckTx       = Sender<unit>
:wat::holon::lru::PutAckRx       = Receiver<unit>
:wat::holon::lru::PutAckChannel  = Channel<unit>
```

Plus the existing data-bearing Get pair (post-K.holon-lru):

```
:wat::holon::lru::GetReplyTx     = Sender<Option<HolonAST>>
:wat::holon::lru::GetReplyRx     = Receiver<Option<HolonAST>>
:wat::holon::lru::GetReplyChannel = Channel<Option<HolonAST>>  ;; renamed from GetReplyPair
```

Final Request enum shape:

```scheme
(:wat::core::enum :wat::holon::lru::Request
  (Get  (probe :HolonAST) (reply-tx :GetReplyTx))     ;; data-bearing reply
  (Put  (key :HolonAST) (val :HolonAST)
        (ack-tx :PutAckTx)))                          ;; unit-ack release
```

Honest at every layer: variant declares which pattern; field
name declares Reply vs Ack; type body confirms data vs unit.

## Open questions for gaze

1. **Option A (PutAckTx) vs Option B (unified Reply enum)?**
2. **`PutAck*` vs `PutReply*`?** Reply implies data; Ack implies
   release. Pattern A's `AckTx` precedent suggests "Ack" is the
   right word when the back-edge is unit. Naming aligns to
   Pattern A flavor at the variant level.
3. **Does this audit catch any OTHER fire-and-forget operations
   in the substrate?** Worth a sweep — if HolonLRU has the
   discipline gap, are there others? (Probably not — Console,
   Telemetry, LRU all checked clean. But worth scanning.)

## Cross-references

- `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels" — the
  doctrine this arc enforces.
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § K channel-
  naming-patterns subsection — Pattern A (Ack) vs Pattern B
  (Reply) vocabulary.
- `docs/arc/2026/04/109-kill-std/SLICE-K-HOLON-LRU.md` — queued
  naming flatten that lands AFTER this arc.
- `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`
  — the substrate file to fix.
