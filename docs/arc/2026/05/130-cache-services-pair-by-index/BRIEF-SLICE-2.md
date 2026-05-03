# Arc 130 Slice 2 — Sonnet Brief

**Drafted 2026-05-02 (evening)** — slice 1 (LRU) shipped via
sonnet sweep; slice 2 mirrors slice 1's substrate-reshape
pattern for `:wat::holon::lru::HologramCacheService`. Three
files modify (vs slice 1's two), since the proof file
`step-B-single-put.wat` also retires its `:should-panic`
annotation. The HolonLRU test file already has a complectens
two-prelude structure (per arc 130's prior complectens
rewrite); the reshape preserves both preludes and only updates
prelude 2's helpers + drops the 8 `:should-panic` annotations
from the prelude 2 deftests.

**Goal:** reshape `:wat::holon::lru::HologramCacheService`
substrate + tests + step-B proof to use pair-by-index via
HandlePool with a unified `Reply` enum (HolonAST-typed
specialization). After this slice, all 8 deadlock-class
deftests in `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
PASS via the new shape, plus the 1 deftest in
`crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat`
PASSES via the new shape. **0 channel-pair-deadlock markers
remain in the workspace.**

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Scope:** `:wat::holon::lru::*` only. Slice 1 (LRU) already
shipped — it's the reference for this slice's substrate
reshape. Slice 3 is closure docs (separate session).

## Read-in-order anchor docs

1. **`.claude/skills/complectens/SKILL.md`** — REQUIRED
   PRE-READ. The discipline that shapes the test files. The
   HolonLRU test file already follows the two-prelude pattern
   (Mixed-outcome files — the two-prelude pattern, in the
   SKILL's "Edge cases" section). Preserve both preludes.
2. **`docs/arc/2026/05/130-cache-services-pair-by-index/SCORE-SLICE-1.md`**
   (if it exists by the time you read this) — slice 1's
   shipped diff is the worked-example template. Mirror its
   substrate-reshape pattern.
3. **`crates/wat-lru/wat/lru/CacheService.wat`** —
   slice 1's shipped substrate. The `:wat::holon::lru::*`
   substrate mirrors this same shape (Reply enum, Handle,
   DriverPair, simplified Request enum, helper-verb
   signatures, factory body, driver loop) with the HolonAST
   specialization (`K = V = HolonAST`).
4. **`crates/wat-lru/wat-tests/lru/CacheService.wat`** —
   slice 1's shipped test file. The reshape pattern (4 helper
   updates + 5 annotation drops, layered structure preserved)
   transfers directly.
5. `docs/arc/2026/05/130-cache-services-pair-by-index/DESIGN.md`
   — the source of truth for the substrate reshape (slice 2
   section is at the bottom; the typealiases, helper-verb
   signatures, etc. are the same shape).
6. `docs/arc/2026/05/130-cache-services-pair-by-index/REALIZATIONS.md`
   — the doctrine that named the complectens discipline.
7. `wat/console.wat` (~298 LOC) — the working pair-by-index
   reference implementation.
8. `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`
   (538 LOC) — the substrate file you'll reshape.
9. `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
   (731 LOC, two-prelude complectens structure) — the main
   test file you'll update.
10. `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat`
    (68 LOC, 1 deftest with `:should-panic`) — the proof file
    you'll reshape.

## What changes

### `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`

The substrate reshape mirrors slice 1's `:wat::lru::*`
reshape, with names adapted to the HolonLRU namespace
(`HologramCacheService::*`) and HolonAST specialization (no
`<K,V>` parameters — K and V are baked as HolonAST in the
HCS substrate).

**ADD** typealiases:

- `:wat::holon::lru::HologramCacheService::Reply` enum:
  - `(GetResult (results :Vector<Option<HolonAST>>))`
  - `(PutAck)`
- `:wat::holon::lru::HologramCacheService::ReplyTx` =
  `Sender<Reply>` (REBINDS the existing name; old body was
  `Sender<Vector<Option<HolonAST>>>` per the GetReplyTx slot)
- `:wat::holon::lru::HologramCacheService::ReplyRx` =
  `Receiver<Reply>` (REBINDS)
- `:wat::holon::lru::HologramCacheService::ReplyChannel` =
  `(ReplyTx, ReplyRx)`
- `:wat::holon::lru::HologramCacheService::Handle` =
  `(ReqTx, ReplyRx)`
- `:wat::holon::lru::HologramCacheService::DriverPair` =
  `(ReqRx, ReplyTx)`

**RETIRE** typealiases:

- `:wat::holon::lru::HologramCacheService::PutAckTx`
- `:wat::holon::lru::HologramCacheService::PutAckRx`
- `:wat::holon::lru::HologramCacheService::PutAckChannel`
- `:wat::holon::lru::HologramCacheService::GetReplyTx`
- `:wat::holon::lru::HologramCacheService::GetReplyRx`
- `:wat::holon::lru::HologramCacheService::GetReplyPair`

(The HolonLRU substrate uses `GetReply*` for the data-bearing
reply family, distinct from the LRU's `Reply*`. Both retire
in favor of the unified `Reply` enum + new ReplyTx/Rx names.)

**RESHAPE** Request enum (drop embedded channels):

```scheme
(:wat::core::enum :wat::holon::lru::HologramCacheService::Request
  (Get  (probes  :wat::core::Vector<wat::holon::HolonAST>))
  (Put  (entries :wat::core::Vector<wat::holon::lru::HologramCacheService::Entry>)))
```

**RESHAPE** Spawn typealias from
`HandlePool<ReqTx>` to `HandlePool<Handle>`.

**RESHAPE** the spawn factory body (mirrors slice 1):
allocate N reply channels, build N Handle tuples + N
DriverPair tuples, populate HandlePool with Handles, hand
DriverPair vector to the driver thread.

**RESHAPE** the driver loop (mirrors slice 1): driver holds
`Vec<DriverPair>`, select fires on req-rx side, index into
DriverPair vector finds matching ReplyTx, after processing
the Request, send the appropriate Reply variant.

**RESHAPE** helper-verb signatures (`HologramCacheService/get`,
`HologramCacheService/put`):

```scheme
;; OLD:
(:wat::holon::lru::HologramCacheService/get
  (req-tx :HologramCacheService::ReqTx)
  (reply-tx :HologramCacheService::GetReplyTx)
  (reply-rx :HologramCacheService::GetReplyRx)
  (probes :Vector<HolonAST>)
  -> :Vector<Option<HolonAST>>)

;; NEW:
(:wat::holon::lru::HologramCacheService/get
  (handle :HologramCacheService::Handle)
  (probes :Vector<HolonAST>)
  -> :Vector<Option<HolonAST>>)
```

Same pattern for `HologramCacheService/put`. Body projects
`(first handle)` + `(second handle)`, sends Request, recvs
Reply, matches on Reply variant.

### `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`

**Current shape (arc 130 prior complectens rewrite):** the
file has TWO `make-deftest` factories per the SKILL's
two-prelude pattern. Read the file's header (lines 1-60)
for the canonical map.

| Prelude | Deftests | Annotations |
|---|---|---|
| `:deftest-hermetic` (lines 67-249) | step1, step2, hcs-trivial-spawn-recv-join, hcs-spawn-and-shutdown, hcs-spawn-send-3-count, hcs-recv-count-and-join (6 deftests, lines 256-296) | None — clean-pass (no arc-126 trigger in prelude 1) |
| `:deftest-service` (lines 308-628) | step3, step4, step5, step6, hcs-spawn-put-get, hcs-spawn-put-3-verify, hcs-spawn-2clients-put-get-verify, hcs-spawn-put-3-eviction (8 deftests, lines 632-724) | All 8 carry `:should-panic("channel-pair-deadlock")` because prelude 2 has helpers with `make-bounded-channel` + helper-verb call sites |

**Post-reshape:** the two-prelude split STAYS (it's the
canonical shape for mixed-outcome tests). What changes:

1. **Prelude 2 helpers** (lines 308-628) — update each helper
   that calls `HologramCacheService/get` or
   `HologramCacheService/put`:
   - DELETE the per-call `make-bounded-channel` allocations
     (ack-pair, reply-pair, ack-tx, ack-rx, reply-tx,
     reply-rx bindings).
   - Replace `req-tx :HologramCacheService::ReqTx` bindings
     with `handle :HologramCacheService::Handle`.
   - Update verb call from `(get/put req-tx reply-tx
     reply-rx ...)` or `(get/put req-tx ack-tx ack-rx ...)`
     to `(get/put handle ...)`.

2. **Pool typealias** in any helper that binds the pool —
   update from `HandlePool<ReqTx>` to `HandlePool<Handle>`.

3. **Drop 8 `:should-panic("channel-pair-deadlock")`
   annotations** at lines 632, 643, 652, 660, 680, 694, 711,
   724.

4. **Prelude 1 helpers** — likely UNCHANGED (they don't call
   helper verbs; they're pure lifecycle / send-recv proofs).
   If any prelude-1 helper binds the pool, update its typealias
   too. Otherwise no change.

5. **Deftest names + counts UNCHANGED.** The 14 total deftests
   (6 hermetic + 8 service) all pass cleanly post-reshape.

### `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat`

Single proof file. 1 deftest, 1 `:should-panic` annotation
at line 23.

**Reshape:**

1. Drop the `:should-panic("channel-pair-deadlock")`
   annotation.
2. Update the deftest body's helper structure:
   - DELETE the per-call `make-bounded-channel` ack-pair
     allocation.
   - Bind `handle :HologramCacheService::Handle` instead of
     `req-tx :ReqTx`.
   - Update the `HologramCacheService/put` call from
     `(put req-tx ack-tx ack-rx entries)` to
     `(put handle entries)`.
3. Update pool typealias from `HandlePool<ReqTx>` to
   `HandlePool<Handle>`.

The proof shape (single deftest, narrow assertion on Put
behavior) stays unchanged.

## Constraints

- **Three files change:**
  - `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat` (substrate)
  - `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` (main test)
  - `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat` (proof)

  No LRU files (slice 1 already shipped). No documentation
  work (slice 3). No commits.

- **Workspace stays GREEN:** `cargo test --release --workspace`
  exits 0. The 14 HCS test-file deftests + 1 step-B deftest
  all PASS without panic. Other workspace tests unchanged.

- **Arc 126's check must NOT fire** on the new helper-verb
  bodies, the reshaped HCS prelude 2 helpers, or the reshaped
  step-B proof. Verify with `cargo build` — any
  `ChannelPairDeadlock` errors mean the redesign has a leak;
  STOP and report.

- **Two-prelude pattern STAYS.** Don't merge `:deftest-hermetic`
  + `:deftest-service` into a single factory just because both
  now pass cleanly. The split is the canonical shape per the
  /complectens SKILL's mixed-outcome edge case; future
  additions to prelude 2 may legitimately re-trigger
  arc-126-class checks.

- **No commits, no pushes.**

## What success looks like

1. `cargo test --release -p wat-holon-lru`: **26 passed; 0
   failed; 0 ignored; 0 should-panic markers in output**. All
   8 prelude-2 service deftests + the step-B proof report
   plain `... ok` (NOT `... ok (should panic)`). The 6 hermetic
   deftests + step-A proof + 4 HologramCache tests all stay
   green.

2. `cargo test --release --workspace`: exit=0; **1820 passed;
   0 failed; 1 ignored** (arc-122 mechanism); ~103 `test
   result: ok` lines; **0 `channel-pair-deadlock` markers
   remaining** (was 9 pre-slice-2; the 8 HCS + 1 step-B
   markers retired). Other unrelated `:should-panic` markers
   (e.g., `tmp-totally-bogus.wat`, wat-edn rust tests, arc-122)
   stay as-is.

3. The HolonLRU substrate file's typealias section reflects
   the new shape exactly per the DESIGN + slice 1's pattern.

4. Helper-verb signatures match the DESIGN's "NEW" shape
   (handle parameter; project first/second; send Request,
   recv Reply, match variant).

5. No `make-bounded-channel` calls remain in the HCS test
   file's prelude-2 helper bodies or in the step-B proof.

## Reporting back

Target ~250 words (slice 2 mirrors slice 1; the report can
reference slice 1's pattern):

1. **File:line refs** for the new typealiases (Reply enum,
   Handle, DriverPair) + the reshaped Spawn + Request enum
   + helper-verb signatures.

2. **The exact final form of:**
   - The `:wat::holon::lru::HologramCacheService::Reply` enum
   - The `:wat::holon::lru::HologramCacheService::Handle` typealias
   - The `:wat::holon::lru::HologramCacheService/get` signature

3. **Two-prelude preservation note:** confirm both
   `:deftest-hermetic` and `:deftest-service` factories
   remain; both still hold their original helper sets
   (modulo the channel-pair-allocation removals in prelude 2).

4. **Test totals:**
   - `cargo test --release -p wat-holon-lru`: passed /
     failed / ignored.
   - `cargo test --release --workspace`: passed / failed /
     ignored / channel-pair-deadlock marker count.

5. **Arc 126 check status:** confirm `cargo build` clean.

6. **Honest deltas:** anything you needed to invent or
   diverge from slice 1's pattern. The HolonAST
   specialization may surface edge cases slice 1 didn't hit
   (e.g., the `Vector<Option<HolonAST>>` typing in the new
   Reply enum's GetResult variant).

7. **LOC delta:** rough line-count change in each file.
   Substrate: similar to slice 1 (~30-50 LOC each direction,
   net near zero). Tests: shrinkage from removed channel-pair
   allocations.

## Sequencing — what to do, in order

1. Read the /complectens SKILL.md.
2. Read slice 1's SCORE (if present) + slice 1's shipped diff
   in `crates/wat-lru/wat/lru/CacheService.wat` +
   `crates/wat-lru/wat-tests/lru/CacheService.wat`.
3. Read DESIGN.md + REALIZATIONS.md.
4. Read `wat/console.wat` (the working reference).
5. Read `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`
   (~538 LOC) — note the GetReply* + PutAck* dual families
   that unify into the single Reply enum.
6. Read `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
   (~731 LOC) — note the two-prelude structure.
7. Read `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat`
   (68 LOC).
8. Run `cargo test --release -p wat-holon-lru` to see the
   baseline (26 passed, 9 should-panic markers).
9. Reshape the HCS substrate file in place.
10. Reshape the HCS test file in place — prelude 2 helpers
    update; 8 annotations drop; two-prelude split stays.
11. Reshape the step-B proof file in place.
12. Run `cargo test --release -p wat-holon-lru` — verify all
    26 tests pass cleanly (0 should-panic markers).
13. Run `cargo test --release --workspace 2>&1 | grep "should
    panic ... ok" | wc -l` — verify 0 channel-pair-deadlock
    markers remain (other unrelated should-panic markers may
    persist).
14. Run `cargo build --release` — verify no
    `ChannelPairDeadlock` or `ScopeDeadlock` errors fire on
    the new helper-verb bodies or reshaped helpers.
15. Report per the "Reporting back" section.

Then DO NOT commit. Working tree stays modified for the
orchestrator to score.
