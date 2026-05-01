# Arc 109 Slice K.holon-lru — `HologramCacheService` grouping noun → namespace flatten + GetReplyPair → GetReplyChannel rename

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
The rename is unblocked. K.holon-lru rides the full plan.

## What this slice does

Three coupled transformations:

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
   (shipped K.lru). HolonLRU's variant-scoped naming
   (`Get*` because only the Get variant has a reply per the
   Request enum's `Put` fire-and-forget design) is preserved —
   only the `Pair` suffix retires.

3. **Clarifying comment near the Request enum.** Per the
   original gaze finding: "Put is fire-and-forget — no
   `PutReply*` types by design." The asymmetry (variant-
   scoped reply types only on Get) reads correctly once the
   reader sees the comment.

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

## Substrate work scope

### HologramCacheService grouping retirement (14 typealiases + 7 verbs flatten)

```
:wat::holon::lru::HologramCacheService::GetReplyTx    → :wat::holon::lru::GetReplyTx
:wat::holon::lru::HologramCacheService::GetReplyRx    → :wat::holon::lru::GetReplyRx
:wat::holon::lru::HologramCacheService::GetReplyPair  → :wat::holon::lru::GetReplyChannel  ;; PATTERN B — rename
:wat::holon::lru::HologramCacheService::ReqTx         → :wat::holon::lru::ReqTx
:wat::holon::lru::HologramCacheService::ReqRx         → :wat::holon::lru::ReqRx
:wat::holon::lru::HologramCacheService::ReqTxPool     → :wat::holon::lru::ReqTxPool
:wat::holon::lru::HologramCacheService::Request       → :wat::holon::lru::Request           ;; real enum; keeps /methods
:wat::holon::lru::HologramCacheService::Spawn         → :wat::holon::lru::Spawn
:wat::holon::lru::HologramCacheService::Reporter      → :wat::holon::lru::Reporter
:wat::holon::lru::HologramCacheService::Report        → :wat::holon::lru::Report             ;; real enum; keeps /methods
:wat::holon::lru::HologramCacheService::Stats         → :wat::holon::lru::Stats              ;; real struct; keeps /methods
:wat::holon::lru::HologramCacheService::MetricsCadence → :wat::holon::lru::MetricsCadence    ;; real struct; keeps /methods
:wat::holon::lru::HologramCacheService::State         → :wat::holon::lru::State              ;; real struct; keeps /methods
:wat::holon::lru::HologramCacheService::Step          → :wat::holon::lru::Step

:wat::holon::lru::HologramCacheService/handle               → :wat::holon::lru::handle
:wat::holon::lru::HologramCacheService/loop                 → :wat::holon::lru::loop
:wat::holon::lru::HologramCacheService/null-metrics-cadence → :wat::holon::lru::null-metrics-cadence
:wat::holon::lru::HologramCacheService/null-reporter        → :wat::holon::lru::null-reporter
:wat::holon::lru::HologramCacheService/run                  → :wat::holon::lru::run
:wat::holon::lru::HologramCacheService/spawn                → :wat::holon::lru::spawn
:wat::holon::lru::HologramCacheService/tick-window          → :wat::holon::lru::tick-window
```

### Real-Type /methods preserve

Stats / MetricsCadence / State / Report / Request methods (e.g.,
`Stats/new`, `State/empty`, `MetricsCadence/tick`, `Request::Get`
constructor) flatten on prefix only.

### `Get` prefix asymmetry — clarifying comment

Per the original gaze finding (preserved in INVENTORY § K's
channel-naming subsection), HolonLRU's `Get*Reply*` variant-
scoped naming is correct: only the Get variant produces a
reply (returns `Option<HolonAST>`); Put is fire-and-forget. The
absence of `PutReply*` typealiases is intentional and honest.
Add a one-line comment near the `Request` enum stating this
explicitly so future readers don't mistake the asymmetry for
an oversight.

## Pattern 3 walker

**`CheckError::BareLegacyHolonLruPath`** — fires on any keyword
starting with `:wat::holon::lru::HologramCacheService::` or
`:wat::holon::lru::HologramCacheService/`. Single walker; the
`canonical_holon_lru_leaf` helper handles the
`GetReplyPair → GetReplyChannel` rename (parallel to K.lru's
`canonical_lru_leaf` for ReqPair → ReqChannel).

## What to ship

### Substrate (Rust + wat-stdlib)

1. **Internal renames in HologramCacheService.wat** — sed:
   - `:wat::holon::lru::HologramCacheService::GetReplyPair` → `:wat::holon::lru::GetReplyChannel`
   - `:wat::holon::lru::HologramCacheService::` → `:wat::holon::lru::` (catches remaining typealiases)
   - `:wat::holon::lru::HologramCacheService/` → `:wat::holon::lru::` (verbs)
   - Inner-no-colon forms (`wat::holon::lru::HologramCacheService::*`)
2. **Add the clarifying comment** near the Request enum:
   `;; Put is fire-and-forget — no PutReply* types by design.`
3. **Header doc rewrite** to identify the post-K.holon-lru
   namespace shape; cite Pattern B reference family; note the
   GetReplyPair → GetReplyChannel rename rationale.
4. **Mint `CheckError::BareLegacyHolonLruPath`** in
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

- HologramCacheService.wat self-refs: 131 sites
- Consumer scope: ~5 files (wat-tests + 1 cross-ref in CacheService.wat
  + live docs + the deferred-doc INSCRIPTIONs)
- Production-side rename count: ~50-60 sites
- Combined: ~200 rename sites + 1 new clarifying comment

Comparable to K.lru / K.console size. Sonnet-tractable.

## What does NOT change

- **Real types** — Stats / MetricsCadence / State / Report /
  Request keep PascalCase + `/methods`; just one less namespace
  segment deep.
- **`ReqTxPool`** — unique structure (pool of just-tx halves,
  not full channels); kept as-is. Functionally honest; not a
  Level 2 mumble.
- **`Get*` prefix asymmetry** — variant-scoped naming is
  correct (only the Get variant has a reply). Add comment;
  don't restructure.
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
   (variant-scoped Pattern B naming preserved + clarifying
   comment shipped + GetReplyPair → GetReplyChannel rename
   shipped).
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

- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § K — doctrine +
  channel-naming-patterns subsection.
- `docs/arc/2026/04/109-kill-std/SLICE-K-LRU.md` — Pattern B
  precedent (Pair → Channel rename + ReplyRx/ReplyChannel
  fill-in).
- `docs/arc/2026/04/109-kill-std/SLICE-K-KERNEL-CHANNEL.md` —
  the prerequisite slice that unblocked this rename.
- `docs/SUBSTRATE-AS-TEACHER.md` — Pattern 3 mechanism.
- `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat` —
  the substrate file flattening internally.
