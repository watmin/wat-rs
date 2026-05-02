# Arc 109 Slice K.holon-lru — `HologramCacheService` grouping noun → namespace flatten (rides AFTER arc 119)

**Compaction-amnesia anchor.** Read this first if you're picking
up slice K.holon-lru mid-flight.

## Provenance

Originally scoped 2026-05-01 mid-arc-109 K-cleanups; gaze blocked
the `GetReplyPair → GetReplyChannel` rename pending the kernel-
side `QueuePair → Channel` rename (otherwise the wrapper would
have said "Channel" while its body said "Pair" — a Level 1 lie at
the boundary).

K.kernel-channel shipped 2026-05-01 (commit `155163f`). The
kernel now uses `:wat::kernel::Channel<T>`; HolonLRU's
`GetReplyPair` body is now `:wat::kernel::Channel<Option<HolonAST>>`.
That blocker cleared.

**Then arc 119 surfaced** (2026-05-01, same day) — what looked
like a small "Put needs an ack-tx" finding grew through three
gaze passes + user direction into a substrate-wide protocol
reshape covering BOTH Pattern B services (LRU and HolonLRU).
Arc 119 lands BEFORE K.holon-lru. Both services adopt
symmetric batch protocol; HolonLRU mints new typealiases
(`PutAckTx/Rx/Channel`, `Entry`); the Request enum reshapes
from "Get has reply / Put is fire-and-forget" to "Get carries
`Vec<HolonAST>` probes returning `Vec<Option<HolonAST>>`; Put
carries `Vec<Entry>` returning `unit` ack". See
`docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md` for the full
locked plan.

K.holon-lru rides AFTER arc 119: this slice flattens the
post-119 namespace shape (more typealiases; no fire-and-forget
commentary needed because the asymmetry retires).

## What this slice does

Two coupled transformations (post-arc-119):

1. **§ K grouping-noun retirement.** `HologramCacheService` is a
   grouping noun (no struct of that name; nested real types
   `Request`, `Stats`, `Report`, `MetricsCadence`, `State` are
   the actual ADTs). Per § K's "/ requires a real Type" doctrine
   the grouping noun retires; verbs and typealiases flatten to
   `:wat::holon::lru::*`.

2. **§ K Pattern B canonicalization (now unblocked).**
   `GetReplyPair` → `GetReplyChannel`. The wrapper's name now
   matches the body (kernel `Channel<T>`); same Pattern B
   convention as `:wat::lru::ReqChannel` / `ReplyChannel`
   (shipped K.lru). Variant-scoped naming (`Get*Reply*` /
   `Put*Ack*`) is preserved — Pattern B for Get (data back)
   + Pattern A for Put (unit-ack release), variant-scoped per
   arc 119's gaze verdict.

The "fire-and-forget Put" clarifying comment from this slice's
earlier draft is **retired**: arc 119 makes Put lock-step with
the rest of the substrate, so the asymmetry it was explaining
no longer exists.

## Audit confirmed: HologramCacheService IS a grouping noun

Three real struct types + two real enum types nested inside:
- `:wat::holon::lru::HologramCacheService::Request` — enum
  (Get / Put variants); line 50
- `:wat::holon::lru::HologramCacheService::Stats` — struct;
  line 108
- `:wat::holon::lru::HologramCacheService::Report` — enum;
  line 124
- `:wat::holon::lru::HologramCacheService::MetricsCadence<G>` —
  struct; line 132
- `:wat::holon::lru::HologramCacheService::State` — struct;
  line 171

Each keeps PascalCase + `/methods` (just one less namespace
segment deep). HologramCacheService itself has no struct
definition — confirms the grouping-noun status.

## Substrate work scope (post-arc-119 shape)

### HologramCacheService grouping retirement (typealiases + verbs flatten)

Arc 119 mints new typealiases under the existing
`HologramCacheService::` prefix (per the "lands before K.holon-lru
flatten" sequence); K.holon-lru flattens all of them in one pass.

Get-side typealiases (Pattern B; data-bearing reply):
```
:wat::holon::lru::HologramCacheService::GetReplyTx    → :wat::holon::lru::GetReplyTx
:wat::holon::lru::HologramCacheService::GetReplyRx    → :wat::holon::lru::GetReplyRx
:wat::holon::lru::HologramCacheService::GetReplyPair  → :wat::holon::lru::GetReplyChannel  ;; PATTERN B — rename
```

Put-side typealiases (Pattern A; unit-ack release; **arc-119-minted**):
```
:wat::holon::lru::HologramCacheService::PutAckTx       → :wat::holon::lru::PutAckTx
:wat::holon::lru::HologramCacheService::PutAckRx       → :wat::holon::lru::PutAckRx
:wat::holon::lru::HologramCacheService::PutAckChannel  → :wat::holon::lru::PutAckChannel
```

Batch element typealias (**arc-119-minted**):
```
:wat::holon::lru::HologramCacheService::Entry         → :wat::holon::lru::Entry
```

Forward-edge channel halves:
```
:wat::holon::lru::HologramCacheService::ReqTx         → :wat::holon::lru::ReqTx
:wat::holon::lru::HologramCacheService::ReqRx         → :wat::holon::lru::ReqRx
:wat::holon::lru::HologramCacheService::ReqTxPool     → :wat::holon::lru::ReqTxPool
```

Real types (keep PascalCase + `/methods`, just one less segment):
```
:wat::holon::lru::HologramCacheService::Request       → :wat::holon::lru::Request           ;; real enum
:wat::holon::lru::HologramCacheService::Report        → :wat::holon::lru::Report             ;; real enum
:wat::holon::lru::HologramCacheService::Stats         → :wat::holon::lru::Stats              ;; real struct
:wat::holon::lru::HologramCacheService::MetricsCadence → :wat::holon::lru::MetricsCadence    ;; real struct
:wat::holon::lru::HologramCacheService::State         → :wat::holon::lru::State              ;; real struct
:wat::holon::lru::HologramCacheService::Spawn         → :wat::holon::lru::Spawn
:wat::holon::lru::HologramCacheService::Reporter      → :wat::holon::lru::Reporter
:wat::holon::lru::HologramCacheService::Step          → :wat::holon::lru::Step
```

Verbs flatten to namespace level:
```
:wat::holon::lru::HologramCacheService/handle               → :wat::holon::lru::handle
:wat::holon::lru::HologramCacheService/loop                 → :wat::holon::lru::loop
:wat::holon::lru::HologramCacheService/null-metrics-cadence → :wat::holon::lru::null-metrics-cadence
:wat::holon::lru::HologramCacheService/null-reporter        → :wat::holon::lru::null-reporter
:wat::holon::lru::HologramCacheService/run                  → :wat::holon::lru::run
:wat::holon::lru::HologramCacheService/spawn                → :wat::holon::lru::spawn
:wat::holon::lru::HologramCacheService/tick-window          → :wat::holon::lru::tick-window
```

Plus arc-119-minted bare batch verbs (already at namespace level
post-119):
```
:wat::holon::lru::get   ;; (get probes :Vec<HolonAST>) → Vec<Option<HolonAST>>
:wat::holon::lru::put   ;; (put entries :Vec<Entry>) → unit
```

### Real-Type /methods preserve

Stats / MetricsCadence / State / Report / Request methods (e.g.,
`Stats/new`, `State/empty`, `MetricsCadence/tick`, `Request::Get`
constructor) flatten on prefix only.

## Pattern 3 walker

**`CheckError::BareLegacyHolonLruPath`** — fires on any keyword
starting with `:wat::holon::lru::HologramCacheService::` or
`:wat::holon::lru::HologramCacheService/`. Single walker; the
`canonical_holon_lru_leaf` helper handles the
`GetReplyPair → GetReplyChannel` rename (parallel to K.lru's
`canonical_lru_leaf` for ReqPair → ReqChannel).

## What to ship

### Substrate (Rust + wat-stdlib)

Note: arc 119 has already landed by this point. The internal
renames operate on the post-119 file shape (with `PutAckTx/Rx/
Channel`, `Entry`, batch fields in the Request enum, and
batch-Vec verb signatures already in place under the
`HologramCacheService::` prefix).

1. **Internal renames in HologramCacheService.wat** — sed:
   - `:wat::holon::lru::HologramCacheService::GetReplyPair` → `:wat::holon::lru::GetReplyChannel`
   - `:wat::holon::lru::HologramCacheService::` → `:wat::holon::lru::` (catches all typealiases including arc-119-minted PutAckTx/Rx/Channel + Entry)
   - `:wat::holon::lru::HologramCacheService/` → `:wat::holon::lru::` (verbs)
   - Inner-no-colon forms (`wat::holon::lru::HologramCacheService::*`)
2. **Header doc rewrite** to identify the post-K.holon-lru
   namespace shape; cite Pattern B + Pattern A variant-scoped
   pairing (Get-data-back / Put-unit-ack); note the
   GetReplyPair → GetReplyChannel rename rationale; cross-ref
   arc 119 for the protocol body.
3. **Mint `CheckError::BareLegacyHolonLruPath`** in
   `src/check.rs`: variant + Display + Diagnostic + walker
   `validate_legacy_holon_lru_path` + `canonical_holon_lru_leaf`
   helper. Walker wired into `check_program` alongside slice
   9d / K.telemetry / K.console / K.lru / K.kernel-channel
   walkers.
5. **No file move.** Substrate file path stays at
   `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`
   (the path's last segment matches the enum-Request-protocol
   service name — one of those judgment calls; could be
   `holon-lru-service.wat` but that's stylistic; not a Level 2
   mumble; deferred).

### Verification

Probe coverage:
- `(:wat::holon::lru::HologramCacheService/spawn ...)` → fires
- `(:wat::holon::lru::spawn ...)` → silent
- `:wat::holon::lru::HologramCacheService::GetReplyPair` → fires;
  canonical is `:wat::holon::lru::GetReplyChannel`
- `:wat::holon::lru::HologramCacheService::Stats/new` → fires
  (as one keyword); canonical is
  `:wat::holon::lru::Stats/new`
- `:wat::holon::lru::Stats` → silent
- `:wat::holon::lru::GetReplyChannel` → silent
- `:my::pkg::HologramCacheService::*` (user paths) → silent

## Sweep order

Same four-tier discipline.

1. **Substrate stdlib** —
   `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`
   (substrate file, internal renames).
2. **Lib + early integration tests** — none expected.
3. **`wat-tests/`** + **`crates/*/wat-tests/`** —
   `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`.
4. **`tests/`**, **`examples/`**, **`crates/*/wat/`** — live
   docs (`docs/CONVENTIONS.md`); the cross-reference in
   `crates/wat-lru/wat/lru/CacheService.wat`.

## Estimated scope

Post-arc-119 self-ref count grows somewhat (new
PutAckTx/Rx/Channel + Entry typealiases + their use sites),
but the rename mechanic is identical — same Pattern 3 walker,
same sed-driven flatten.

- HologramCacheService.wat self-refs: ~140-160 sites
  (post-119 includes new typealiases)
- Consumer scope: ~5 files (wat-tests + cross-ref in CacheService.wat
  + live docs + INSCRIPTIONs)
- Production-side rename count: ~60-80 sites (post-119 has more
  call-site bindings to rename)
- Combined: ~220-240 rename sites

Comparable to K.lru / K.console size. Sonnet-tractable.

## What does NOT change

- **Real types** — Stats / MetricsCadence / State / Report /
  Request keep PascalCase + `/methods`; just one less namespace
  segment deep.
- **`ReqTxPool`** — unique structure (pool of just-tx halves,
  not full channels); kept as-is. Functionally honest; not a
  Level 2 mumble.
- **Variant-scoped channel naming** — `Get*Reply*` (Pattern B,
  data back) + `Put*Ack*` (Pattern A, unit-ack release)
  preserved post-arc-119. Each variant declares its own
  back-edge family because Get and Put genuinely differ in
  what they return.
- **File path** — substrate file stays at
  `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`.
  Renaming the file would couple with a stylistic decision
  (`holon-lru-service.wat`?) that's not gaze-flagged and would
  expand scope.

## Closure (slice K.holon-lru step N)

When sweep is structurally complete:

1. Update `INVENTORY.md` § K — strike HologramCacheService row
   in the grouping-noun cleanup table; mark ✓ shipped K.holon-lru.
   Channel-naming-patterns table's HolonLRU row marked ✓
   (variant-scoped Pattern B + Pattern A naming per arc 119;
   GetReplyPair → GetReplyChannel rename shipped; PutAckTx/Rx/
   Channel + Entry typealiases minted by arc 119 and flattened
   in this slice).
2. Update `J-PIPELINE.md` — slice K.holon-lru done.
3. Update `SLICE-K-HOLON-LRU.md` — flip from anchor to durable
   shipped-record.
4. Add 058 changelog row noting:
   - Fourth § K application; closes the service-crate naming
     cleanup family
   - First slice to validate § K + K.kernel-channel's combined
     vocabulary on a Pattern B real codebase
   - All four service crates (Telemetry, Console, LRU, HolonLRU)
     now share canonical channel-family naming uniformly

## Cross-references

- `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md` — protocol
  reshape that lands BEFORE this slice (mints PutAckTx/Rx/Channel
  + Entry; reshapes Request to enum-based batch; covers both LRU
  and HolonLRU symmetrically).
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § K — doctrine +
  channel-naming-patterns subsection.
- `docs/arc/2026/04/109-kill-std/SLICE-K-LRU.md` — Pattern B
  precedent (Pair → Channel rename + ReplyRx/ReplyChannel
  fill-in). Note: arc 119 also reshapes LRU's protocol body
  (Body retires; ReplyTx widens to Vec<Option<V>>) but the
  K.lru-shipped names stay.
- `docs/arc/2026/04/109-kill-std/SLICE-K-KERNEL-CHANNEL.md` —
  the prerequisite slice that unblocked the GetReplyChannel
  rename.
- `docs/SUBSTRATE-AS-TEACHER.md` — Pattern 3 mechanism.
- `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat` —
  the substrate file flattening internally.
