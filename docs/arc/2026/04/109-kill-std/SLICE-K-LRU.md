# Arc 109 Slice K.lru — `CacheService` grouping noun → namespace flatten + Pattern B fill-in + ReqPair → ReqChannel rename

**Status: shipped 2026-05-01.** Substrate (commit `158dbef`) +
consumer sweep. 8 files swept (1 substrate + 7 consumer); 131
internal renames in CacheService.wat + ~36 consumer renames; 3
new typealiases (ReqChannel — replacing ReqPair; ReplyRx;
ReplyChannel); cargo test --release --workspace 1476/0.

Three coupled transformations validated atomically:

1. **§ K grouping-noun retirement** — same mechanism as
   K.telemetry / K.console; rehearsed.
2. **Pattern B canonicalization (gaze finding)** — ReqPair
   renamed to ReqChannel; eliminates the in-crate
   ReqPair/ReplyChannel suffix mumble. Substrate-wide every
   service crate now uses the "Channel" suffix.
3. **Pattern B fill-in (gaze finding)** — added missing
   `ReplyRx<V>` + `ReplyChannel<V>` typealiases; the
   unallocated reply receiver in get/put now has a domain name.

LRU is now the **Pattern B canonical reference** alongside
Telemetry's Pattern A reference.

**Originally drafted as a compaction-amnesia anchor mid-slice;
preserved here as the durable record.** Slice K.lru is the
sixth Pattern 3 application after slices 1c/1d/1e/9d/K.telemetry/
K.console. First slice to validate § K's doctrine on Pattern B
(Request + Reply), and first to consolidate the channel-naming
patterns substrate-wide via the ReqPair → ReqChannel rename.

**Walker shape:** `validate_legacy_lru_cache_service_path` is
K.console's walker plus a `canonical_lru_leaf` helper that maps
`ReqPair → ReqChannel`. Otherwise identical Pattern 3 keyword-
prefix detection.

**Sweep notes:** the consumer-sweep agent surfaced an additional
contextual finding — `:wat::lru::CacheService<K,V>` (program-
level grouping references in live docs) didn't have a 1:1
replacement under § K's flatten rule (the program-as-grouping is
exactly what retires). Agent rewrote those to
`:wat::lru::*` (the cluster) + `:wat::lru::spawn<K,V,G>` (the
verb) per the substrate header's canonical phrasing. Plus
discovered a stale USER-GUIDE example call-site
`(:wat::lru::CacheService 1024 8)` already wrong per arc 078
(spawn now takes 4 args); updated to current signature.

## Audit result (2026-05-01)

`:wat::lru::CacheService` IS a grouping noun (no struct named
`CacheService` itself). The real types are nested **under** it:
- `:wat::lru::CacheService::Stats` (struct, line 78)
- `:wat::lru::CacheService::MetricsCadence<G>` (struct, line 96)
- `:wat::lru::CacheService::State<K,V>` (struct, line 132)

Per § K's doctrine: `CacheService` retires as a grouping noun;
the three real structs keep their PascalCase + `/methods` (just
one less namespace segment deep).

## What this slice does

Two coupled transformations:

1. **§ K grouping-noun retirement.** `:wat::lru::CacheService::*`
   and `:wat::lru::CacheService/*` flatten to `:wat::lru::*`.
   Real structs Stats / MetricsCadence / State preserve their
   `/methods` (just shorter path).
2. **§ K Pattern B fill-in (gaze finding).** Today's
   `ReplyTx<V>` has no sibling `ReplyRx<V>` — the
   unallocated-receiver-with-no-domain-name Level 2 mumble.
   Add `:wat::lru::ReplyRx<V>` + `:wat::lru::ReplyChannel<V>`
   typealiases.

Pattern A vs B reminder (per INVENTORY § K's channel-patterns
subsection): LRU is Pattern B (data forward + data back; sender
embedded in request). `ReqTx<K,V>` / `ReqRx<K,V>` carry the
forward edge with an embedded `ReplyTx<V>`; the consumer
allocates a sibling `ReplyRx<V>` and reads the response off it.
Today the consumer allocates the rx ad-hoc; post-K.lru it's a
named typealias.

## Substrate work scope

### CacheService grouping retirement (13 typealiases + 9 verbs flatten)

```
:wat::lru::CacheService::Body            → :wat::lru::Body
:wat::lru::CacheService::MetricsCadence  → :wat::lru::MetricsCadence  ;; real struct; keeps /methods
:wat::lru::CacheService::ReplyTx         → :wat::lru::ReplyTx
:wat::lru::CacheService::Report          → :wat::lru::Report
:wat::lru::CacheService::Reporter        → :wat::lru::Reporter
:wat::lru::CacheService::ReqPair         → :wat::lru::ReqChannel      ;; gaze-renamed (in-crate ReqPair/ReplyChannel mumble)
:wat::lru::CacheService::ReqRx           → :wat::lru::ReqRx
:wat::lru::CacheService::ReqTx           → :wat::lru::ReqTx
:wat::lru::CacheService::Request         → :wat::lru::Request
:wat::lru::CacheService::Spawn           → :wat::lru::Spawn
:wat::lru::CacheService::State           → :wat::lru::State            ;; real struct; keeps /methods
:wat::lru::CacheService::Stats           → :wat::lru::Stats            ;; real struct; keeps /methods
:wat::lru::CacheService::Step            → :wat::lru::Step

:wat::lru::CacheService/get               → :wat::lru::get
:wat::lru::CacheService/handle            → :wat::lru::handle
:wat::lru::CacheService/loop              → :wat::lru::loop
:wat::lru::CacheService/loop-step         → :wat::lru::loop-step
:wat::lru::CacheService/null-metrics-cadence → :wat::lru::null-metrics-cadence
:wat::lru::CacheService/null-reporter     → :wat::lru::null-reporter
:wat::lru::CacheService/put               → :wat::lru::put
:wat::lru::CacheService/spawn             → :wat::lru::spawn
:wat::lru::CacheService/tick-window       → :wat::lru::tick-window
```

### Real-Type /methods preserve

Stats / MetricsCadence / State methods (e.g.,
`CacheService::Stats/new`, `CacheService::State/empty`,
`CacheService::MetricsCadence/tick`) flatten on prefix only:
```
:wat::lru::CacheService::Stats/new           → :wat::lru::Stats/new
:wat::lru::CacheService::Stats/zero          → :wat::lru::Stats/zero
:wat::lru::CacheService::State/empty         → :wat::lru::State/empty
:wat::lru::CacheService::State/...           → :wat::lru::State/...
:wat::lru::CacheService::MetricsCadence/new  → :wat::lru::MetricsCadence/new
:wat::lru::CacheService::MetricsCadence/...  → :wat::lru::MetricsCadence/...
```

### Pattern B fill-in — add missing typealiases

Per gaze finding (INVENTORY § K channel-naming-patterns
subsection):

```
(:wat::core::typealias :wat::lru::ReplyRx<V>
  :wat::kernel::QueueReceiver<wat::core::Option<V>>)
(:wat::core::typealias :wat::lru::ReplyChannel<V>
  :(wat::lru::ReplyTx<V>,wat::lru::ReplyRx<V>))
```

These are NEW typealiases — not flattened from existing names.
Today's `get`/`put` body code constructs the rx ad-hoc; post-
K.lru, those construction sites can use the named alias.

### Renaming `ReqPair` → `ReqChannel` (gaze-resolved 2026-05-01)

Initial draft kept `ReqPair`. User flagged the question; gaze
ward returned a Level 2 mumble finding:

> Within the SAME crate, post-K.lru, the reader sees two
> typealiases of identical shape — `ReqPair<K,V>` and
> `ReplyChannel<V>`. Same structural pattern `(X-tx, X-rx)`,
> different suffix word. The reader who arrives cold MUST stop
> and ask: "is a `Pair` a different kind of thing from a
> `Channel`?" That's the lookup-forcing pause gaze flags as a
> mumble. The answer is "no, they're the same kind of thing"
> — which means the suffix divergence carries no information,
> only friction.

**Verdict: rename `ReqPair<K,V>` → `ReqChannel<K,V>`.** Anchor:
the rename eliminates the in-crate suffix divergence between
`ReqChannel` and `ReplyChannel`. Substrate-wide consistency
(telemetry / console / lru all using `ReqChannel`) is a free
bonus, not the primary justification.

Plus update the doc-comment at the typealias to match
("the (ReqTx, ReqRx) channel as a single name") so prose
doesn't stay-stale post-rename.

## Pattern 3 walker

**`CheckError::BareLegacyLruCacheServicePath`** — fires on any
keyword starting with `:wat::lru::CacheService::` or
`:wat::lru::CacheService/`. Same shape as K.telemetry's walker;
canonical replacement just strips the segment (no leaf rename
like K.console's Tx/Rx → ReqTx/ReqRx, since LRU's leaves all
keep their names — just lose the `CacheService` segment).

## What to ship

### Substrate (Rust + wat-stdlib)

1. **Internal renames in `crates/wat-lru/wat/lru/CacheService.wat`** —
   sed-style replacement (both colon-prefixed and inner-no-colon
   forms):
   - `:wat::lru::CacheService::` → `:wat::lru::`
   - `:wat::lru::CacheService/` → `:wat::lru::`
   - `wat::lru::CacheService::` → `wat::lru::` (inner-no-colon)
2. **Add `ReplyRx<V>` + `ReplyChannel<V>` typealiases** in
   `crates/wat-lru/wat/lru/CacheService.wat` near the existing
   typealias section. Update `get`/`put` body code if it
   benefits from the new aliases (judgment call; minimal change
   is fine).
3. **Mint `CheckError::BareLegacyLruCacheServicePath`** in
   `src/check.rs`: variant + Display + Diagnostic + walker
   `validate_legacy_lru_cache_service_path`. Wired into
   `check_program` alongside slice 9d / K.telemetry / K.console.
4. **No file move.** `crates/wat-lru/wat/lru/CacheService.wat`
   path stays — file is named after the substrate concept it
   ships. Header doc gets updated to identify the
   post-K.lru namespace shape.

### Verification

Probe coverage:
- `(:wat::lru::CacheService/get ...)` → fires
- `(:wat::lru::get ...)` → silent
- `:wat::lru::CacheService::Stats` → fires; canonical is
  `:wat::lru::Stats`
- `:wat::lru::CacheService::Stats/new` → fires (as one keyword);
  canonical is `:wat::lru::Stats/new`
- `:wat::lru::ReplyRx<wat::core::i64>` → silent (new typealias)
- `:wat::lru::Stats` → silent
- `:my::pkg::CacheService::*` (user paths) → silent

## Sweep order

Same four-tier discipline.

1. **Substrate stdlib** — `crates/wat-lru/wat/lru/CacheService.wat`
   (substrate file, internal renames + new typealiases).
2. **Lib + early integration tests** — none expected; LRU's
   substrate is wat-only.
3. **`wat-tests/`** + **`crates/*/wat-tests/`** —
   `crates/wat-lru/wat-tests/lru/CacheService.wat`.
4. **`tests/`**, **`examples/`**, **`crates/*/wat/`** — none
   expected; LRU isn't widely consumed outside its own crate +
   live docs (README, CONVENTIONS, ZERO-MUTEX, USER-GUIDE).

## Estimated scope

- CacheService.wat self-refs: **131 sites**
- Consumer files: ~5 production (1 wat-tests + 4 live docs)
- Total occurrences across consumers: 195
- New typealiases: 2 (ReplyRx + ReplyChannel)
- Combined: ~328 rename sites + 2 new typealiases

Bigger than K.telemetry (196 consumer + 117 substrate = 313).
Sonnet-tractable.

## What does NOT change

- Stats / MetricsCadence / State as real struct types — they
  keep their PascalCase + `/methods`; just one less namespace
  segment deep.
- The `Body`, `Report`, `Reporter` typealiases — rename only;
  body unchanged.
- `ReqPair` — kept as-is (gaze-strict; not a Level 2 mumble).
- File path — `crates/wat-lru/wat/lru/CacheService.wat` stays.
  No filesystem move in this slice.
- Rust-Rust dispatch / wat-Rust shim layer — no Rust code
  changes; this is a pure wat-substrate rename + walker.

## Closure (slice K.lru step N)

When sweep is structurally complete:

1. Update `INVENTORY.md` § K — strike CacheService row in the
   grouping-noun cleanup table; mark ✓ shipped K.lru. Channel-
   naming-patterns table's LRU row marked ✓ (Pattern B
   typealiases now complete).
2. Update `J-PIPELINE.md` — slice K.lru done; remove from
   independent-sweeps backlog.
3. Update `SLICE-K-LRU.md` — flip from anchor to durable
   shipped-record.
4. Add 058 changelog row noting:
   - Third § K application (after K.telemetry + K.console)
   - First § K to validate the doctrine on Pattern B (data-
     forward + data-back; reply-tx in request)
   - Pattern B's typealiases now complete

## Cross-references

- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § K — doctrine
  + channel-naming-patterns subsection (Pattern B reference).
- `docs/arc/2026/04/109-kill-std/SLICE-K-TELEMETRY.md` — first
  § K application (Pattern A reference).
- `docs/arc/2026/04/109-kill-std/SLICE-K-CONSOLE.md` — second
  § K application (Pattern A canonicalization).
- `docs/SUBSTRATE-AS-TEACHER.md` — the migration mechanism.
- `crates/wat-lru/wat/lru/CacheService.wat` — the substrate
  file flattening internally.
