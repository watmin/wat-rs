# Arc 130 Slice 1 — Sonnet Brief

**Refreshed 2026-05-02 (evening)** — arc 135 slice 1's complectens
sweep landed in the interim, reshaping
`crates/wat-lru/wat-tests/lru/CacheService.wat` from a single
round-trip deftest into a 5-layer compositional structure
(per arc 130 REALIZATIONS.md — the file's header comments
explicitly reference this arc's REALIZATIONS as the source of
the layered shape). The substrate-side reshape (typealiases,
Spawn, Request enum, helper-verb signatures) is unchanged from
the prior brief. The test-side reshape is now: update 4
prelude helpers (Layer 0 / Layer 1a / Layer 1b / Layer 2),
drop 5 `:should-panic("channel-pair-deadlock")` annotations,
leave deftest names unchanged.

**Refreshed 2026-05-02 (early)** — the arc was paused 2026-05-01
when the original sweep killed mid-run; the deadlock-class
chain (arcs 131 / 132 / 133 / 134) shipped in the interim.
That refresh updated the brief for the chain. Arc 130 itself
is unchanged in goal; only the surrounding substrate +
test-file context shifted.

**Goal:** reshape `:wat::lru::*` (the LRU CacheService) substrate
+ tests to use pair-by-index via HandlePool with a unified
`Reply<V>` enum. After this slice, all 5 deadlock-class deftests
in `crates/wat-lru/wat-tests/lru/CacheService.wat` PASS via the
new shape (no `:should-panic` annotations; arc 126's check
doesn't fire on any prelude helper).

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Scope:** `:wat::lru::*` only. Slice 2 mirrors this work for
`:wat::holon::lru::*` in a separate session. Slice 3 is closure
docs.

**Console reference:** `wat/console.wat` (~298 LOC, in the
top-level wat tree, NOT inside `crates/`). This file IS the
working pair-by-index pattern. Read it cover-to-cover before
touching the LRU substrate.

## Read-in-order anchor docs

1. **`.claude/skills/complectens/SKILL.md`** — REQUIRED
   PRE-READ. The discipline that shapes the test file. The
   existing 4-helper / 5-deftest layered structure follows
   this doctrine; the reshape MUST preserve it. The SKILL
   covers the four questions for test-file shape, the
   no-helper-for-channel-pair rule, the pop-before-finish
   lifecycle requirement, and the rune-exemption format.
2. `docs/arc/2026/05/130-cache-services-pair-by-index/DESIGN.md`
   — the new typealiases, helper-verb signatures, driver
   reshape, and the four-questions framing. Source of truth.
3. `docs/arc/2026/05/130-cache-services-pair-by-index/REALIZATIONS.md`
   — the doctrine that named the complectens discipline (the
   test file's header references this).
4. `docs/arc/2026/05/126-channel-pair-deadlock-prevention/INSCRIPTION.md`
   — the rule arc 130 routes around. Specifically the
   "queued follow-ups" section names this redesign explicitly.
5. `docs/ZERO-MUTEX.md` § "Routing acks" — the canonical
   "pair-by-index vs embedded reply-tx" doctrine. Console is
   the reference implementation.
6. `wat/console.wat` (~298 LOC) — the working pair-by-index
   reference implementation. Read cover-to-cover. Console uses
   `Console::Handle = (Tx, AckRx)` with a HandlePool of
   pre-allocated handles. Slice 1 mirrors this pattern.
7. `crates/wat-lru/wat/lru/CacheService.wat` — the file that
   reshapes. ~456 lines today. Read it fully before editing.
8. `crates/wat-lru/wat-tests/lru/CacheService.wat` — the test
   file that retires its `:should-panic` annotations + has its
   helpers updated. ~265 LOC. The 4-helper / 5-deftest
   compositional structure stays; only the helper bodies +
   annotations change.

## What changes

### `crates/wat-lru/wat/lru/CacheService.wat`

**ADD** typealiases:

- `:wat::lru::Reply<V>` enum:
  - `(GetResult (results :Vector<Option<V>>))`
  - `(PutAck)`
- `:wat::lru::ReplyTx<V>` = `Sender<Reply<V>>` (REBINDS the
  existing name; old body was `Sender<Vector<Option<V>>>`)
- `:wat::lru::ReplyRx<V>` = `Receiver<Reply<V>>` (REBINDS)
- `:wat::lru::ReplyChannel<V>` = `(ReplyTx<V>, ReplyRx<V>)`
- `:wat::lru::Handle<K,V>` = `(ReqTx<K,V>, ReplyRx<V>)`
- `:wat::lru::DriverPair<K,V>` = `(ReqRx<K,V>, ReplyTx<V>)`

**RETIRE** typealiases:

- `:wat::lru::PutAckTx`
- `:wat::lru::PutAckRx`
- `:wat::lru::PutAckChannel`

**RESHAPE** Request enum (drop embedded channels):

```scheme
(:wat::core::enum :wat::lru::Request<K,V>
  (Get  (probes  :wat::core::Vector<K>))
  (Put  (entries :wat::core::Vector<wat::lru::Entry<K,V>>)))
```

**RESHAPE** Spawn typealias:

```scheme
;; Old: HandlePool<ReqTx<K,V>>
;; New: HandlePool<Handle<K,V>>
(:wat::core::typealias :wat::lru::Spawn<K,V>
  :(wat::kernel::HandlePool<wat::lru::Handle<K,V>>,
    wat::kernel::Thread<wat::core::unit,wat::core::unit>))
```

**RESHAPE** the spawn factory body:

The current spawn allocates N request channels and populates
HandlePool with N ReqTx entries. The reshape:
- Allocate N request channels (existing).
- Allocate N **reply** channels (NEW — one per slot).
- Build N Handle<K,V> tuples = (ReqTx, ReplyRx) — the client
  side.
- Build N DriverPair<K,V> tuples = (ReqRx, ReplyTx) — the
  driver side.
- Populate HandlePool with the N Handle tuples.
- Pass the N DriverPair vector to the driver thread.

Mirror Console's existing factory pattern; if you're unsure
of the exact shape, read Console's substrate verbatim.

**RESHAPE** the driver loop:

The current driver matches Request, recovers `reply-tx` from
the embedded variant, and sends. The reshape:
- The driver holds a `Vec<DriverPair<K,V>>` (was
  `Vec<ReqRx<K,V>>`).
- Select fires on the request-rx side; the index ALSO indexes
  into the DriverPair vector to locate the matching ReplyTx.
- After processing the Request, send the appropriate Reply
  variant on the matched reply-tx.
- Mirror Console's driver-pair-by-index logic.

**RESHAPE** the helper-verb bodies:

```scheme
;; OLD:
(:wat::lru::get<K,V>
  (req-tx :wat::lru::ReqTx<K,V>)
  (reply-tx :wat::lru::ReplyTx<V>)
  (reply-rx :wat::lru::ReplyRx<V>)
  (probes :Vector<K>)
  -> :Vector<Option<V>>)

;; NEW:
(:wat::lru::get<K,V>
  (handle :wat::lru::Handle<K,V>)
  (probes :Vector<K>)
  -> :Vector<Option<V>>)
```

The helper-verb body projects `(first handle)` for ReqTx,
`(second handle)` for ReplyRx, sends Request::Get, recvs
Reply, matches on the GetResult variant. See DESIGN.md for
the exact body shape.

Same pattern for `:wat::lru::put`. Returns `unit` after
matching `Reply::PutAck`.

### `crates/wat-lru/wat-tests/lru/CacheService.wat`

**Current shape (arc 135 slice 1 outcome):** the file is a
4-helper / 5-deftest compositional structure per the layered
discipline arc 130 REALIZATIONS.md introduced. The file's own
header comments (lines 1-42) document it. DO NOT restructure.
The reshape is mechanical updates to the existing helpers +
deftest annotations.

**Helper layout (lines 44-210, single `make-deftest` block):**

| Helper | Lines | What it does | Channel-pair pattern? |
|---|---|---|---|
| `:test::lru-spawn-and-shutdown` | 51-69 | Layer 0: spawn → finish pool → join. Pure lifecycle. | NO (no `make-bounded-channel`) |
| `:test::lru-spawn-then-put` | 82-114 | Layer 1a: spawn + one Put | YES (ack-pair, lines 102-105) |
| `:test::lru-spawn-then-get` | 116-152 | Layer 1b: spawn + one Get | YES (reply-pair, lines 135-141) |
| `:test::lru-spawn-put-then-get` | 160-210 | Layer 2: spawn + Put + Get | YES (both pairs, lines 182-185 + 193-199) |

**Deftest layout (lines 226-265):**

| Deftest | Line | Annotation | Body |
|---|---|---|---|
| `test-lru-spawn-and-shutdown` | 226-229 | `:should-panic("channel-pair-deadlock")` | calls Layer 0 helper |
| `test-lru-spawn-then-put` | 232-235 | `:should-panic("channel-pair-deadlock")` | calls Layer 1a helper |
| `test-lru-spawn-then-get` | 238-244 | `:should-panic("channel-pair-deadlock")` | calls Layer 1b helper |
| `test-lru-spawn-put-then-get` | 247-253 | `:should-panic("channel-pair-deadlock")` | calls Layer 2 helper |
| `test-cache-service-put-then-get-round-trip` | 256-265 | `:should-panic("channel-pair-deadlock")` | Layer 2 helper + assert results |

All 5 deftests carry the `:should-panic` annotation because the
shared prelude (the `make-deftest` block, lines 44-210) includes
Layer 1+ helpers whose bodies contain `make-bounded-channel`
calls. Arc 126's check fires at freeze time; the annotation
catches the intentional panic.

**The reshape — 4 helper updates + 5 annotation drops:**

1. **Layer 0 helper (`:test::lru-spawn-and-shutdown`):** update
   the `pool` typealias from `HandlePool<ReqTx<K,V>>` to
   `HandlePool<Handle<K,V>>`. No body changes — Layer 0 doesn't
   call any helper verbs. Lines 60-65 area only.

2. **Layer 1a helper (`:test::lru-spawn-then-put`):**
   - Update `pool` typealias as above.
   - Replace the `req-tx :ReqTx<K,V> = HandlePool::pop pool`
     binding with `handle :Handle<K,V> = HandlePool::pop pool`.
   - DELETE the `ack-pair / ack-tx / ack-rx` bindings entirely
     (lines 102-105) — the substrate owns ack channels now.
   - Update the `:wat::lru::put` call from `(put req-tx ack-tx
     ack-rx entries)` to `(put handle entries)`.

3. **Layer 1b helper (`:test::lru-spawn-then-get`):**
   - Same pool/handle updates.
   - DELETE the `reply-pair / reply-tx / reply-rx` bindings
     (lines 135-141).
   - Update the `:wat::lru::get` call from `(get req-tx reply-tx
     reply-rx probes)` to `(get handle probes)`.

4. **Layer 2 helper (`:test::lru-spawn-put-then-get`):** apply
   the union of (2) and (3) — drop both pair allocations,
   update both verb calls.

5. **Deftest annotations:** REMOVE all 5
   `:wat::test::should-panic("channel-pair-deadlock")` lines
   (227, 233, 239, 248, 258). The deftest names + bodies stay
   unchanged.

6. **`:time-limit` annotations:** the file has none currently.
   Arc 132 made 200ms the default; no need to add explicit
   annotations unless a test legitimately exceeds the default
   (these should not).

**Inner-let* nesting:** the existing helpers ALREADY use the
canonical inner-let* shape per SERVICE-PROGRAMS.md § "The
lockstep" — outer scope holds only the driver Thread, inner
scope owns pool + handle + work, inner returns the Thread (or
a `(Thread, results)` tuple for the helpers that produce
data). DO NOT restructure the let* nesting; arc 131 + arc 135
already applied it. The reshape is purely the substitutions
above.

**Worked example for Layer 1a's helper (post-reshape):**

```scheme
(:wat::core::define
  (:test::lru-spawn-then-put
    (k :wat::core::String)
    (v :wat::core::i64)
    -> :wat::core::unit)
  (:wat::core::let*
    (((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::core::let*
        (((spawn :wat::lru::Spawn<wat::core::String,wat::core::i64>)
          (:wat::lru::spawn 16 1
            :wat::lru::null-reporter
            (:wat::lru::null-metrics-cadence)))
         ((pool :wat::kernel::HandlePool<wat::lru::Handle<wat::core::String,wat::core::i64>>)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
          (:wat::core::second spawn))
         ((handle :wat::lru::Handle<wat::core::String,wat::core::i64>)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :wat::core::unit)
          (:wat::kernel::HandlePool::finish pool))
         ;; ack-pair / ack-tx / ack-rx GONE — substrate owns them.
         ((_put :wat::core::unit)
          (:wat::lru::put handle
            (:wat::core::conj
              (:wat::core::Vector :wat::lru::Entry<wat::core::String,wat::core::i64>)
              (:wat::core::Tuple k v)))))
        d))
     ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver)))
    ()))
```

The other helpers reshape analogously. Layer 1b and Layer 2's
helpers return a `(Thread, results)` tuple instead of just the
Thread; that pattern stays unchanged.

## Constraints

- **Two files change:**
  - `crates/wat-lru/wat/lru/CacheService.wat` (substrate)
  - `crates/wat-lru/wat-tests/lru/CacheService.wat` (test)
  
  No `:wat::holon::lru::*` work in this slice (slice 2 does
  HolonLRU). No documentation work (slice 3). No commits.

- **Workspace stays GREEN:** `cargo test --release -p wat-lru
  --test test` exits 0; the single LRU test reports `... ok`
  (NOT `... ok (should panic)`).

- **Arc 126's check must NOT fire** on the new helper-verb
  bodies or on the rewritten test body. Verify with `cargo
  build` — any `ChannelPairDeadlock` errors mean the redesign
  has a leak; STOP and report.

- **All other workspace tests stay green.** `cargo test --release
  --workspace` ships the same 100 `test result: ok` lines as
  before slice 1. Specifically: HolonLRU's tests STILL fail or
  use `:should-panic` until slice 2 reshapes them. If they
  newly break (because the LRU reshape leaks types or shared
  symbols into HolonLRU's universe), STOP and report.

- **No commits, no pushes.**

## What success looks like

1. `cargo test --release -p wat-lru`: 12 tests passed, 0
   failed, 0 ignored, **0 should-panic markers**. All 5
   deadlock-class tests (`test-lru-spawn-and-shutdown`,
   `test-lru-spawn-then-put`, `test-lru-spawn-then-get`,
   `test-lru-spawn-put-then-get`,
   `test-cache-service-put-then-get-round-trip`) report
   plain `... ok` (NOT `... ok (should panic)`). Final
   round-trip test's `assert-eq` passes (results contain
   `Some Some 42`).

2. `cargo test --release --workspace`: exit=0; 1820 passed,
   0 failed, 1 ignored (arc-122 mechanism), **14
   should-panic markers** (was 19; the 5 LRU markers came
   off, the 14 in HolonLRU + step-B + others remain — slice
   2 retires those).

3. The LRU substrate file's typealias section reflects the
   new shape exactly per the DESIGN.

4. Helper-verb signatures match the DESIGN's "NEW" shape.

5. No `make-bounded-channel` calls remain in the LRU test
   file's helper bodies (they all moved into the substrate's
   spawn factory).

## Console reference

Console's substrate at `wat/console.wat` (~298 LOC) IS the
working pair-by-index reference. Read it cover-to-cover before
reshaping. Key landmarks (line numbers approximate):

- `Console::Message` typealias — Console's payload (single
  variant; no Reply enum needed because Console is one-verb).
- `Console::ReqTx` / `ReqRx` / `ReqChannel` typealiases.
- `Console::AckTx` / `AckRx` / `AckChannel` typealiases.
- `Console::Handle = (Tx, AckRx)` — the client's view.
- `Console::DriverPair = (Rx, AckTx)` — the driver's view.
- `Console::Spawn = (HandlePool<Handle>, Thread<unit, unit>)`.
- `Console`'s spawn factory body (allocates N message channels
  + N ack channels, builds N handles + N driver-pairs,
  populates pool, hands driver-pairs to the driver thread).
- `Console`'s driver loop (selects on the ReqRx vector;
  index → DriverPair → recv → process → send Ack on the
  matching AckTx).

Console is single-verb: ack carries `unit`. Cache is multi-
verb: Get returns `Vec<Option<V>>`, Put returns `unit`. The
Reply<V> enum unifies these so both verbs share one reply
channel per slot. Mirror Console's structure but substitute:

- `Console::Message` → `lru::Request<K,V>` enum (Get / Put).
- `Console::AckChannel` → `lru::ReplyChannel<V>` carrying
  `Reply<V>` enum (GetResult / PutAck).
- Driver's `recv → process → send Ack` becomes
  `recv → match Request → produce Reply variant → send Reply`.

If Console's pattern doesn't transcribe cleanly somewhere,
flag it in the report — but the parallel should hold:
Console:Cache :: single-verb-ack:multi-verb-reply.

## Reporting back

Target ~200 words (slightly longer than usual; this is a
substrate reshape):

1. **File:line refs** for the new typealiases (Reply enum,
   Handle, DriverPair) + the reshaped Spawn + the reshaped
   Request enum + the reshaped helper-verb signatures.

2. **The exact final form of:**
   - The `:wat::lru::Reply<V>` enum
   - The `:wat::lru::Handle<K,V>` typealias
   - The `:wat::lru::get<K,V>` signature

3. **Driver-loop reshape note:** how does the driver index
   into the DriverPair vector to find the matching ReplyTx?
   Does it select-by-index, or via the request payload
   carrying the index? (Console's pattern is the reference;
   confirm what shape you adopted.)

4. **Test totals:**
   - `cargo test --release -p wat-lru --test test`: passed /
     failed / ignored.
   - `cargo test --release --workspace`: passed / failed /
     ignored.

5. **Arc 126 check status:** confirm the new helper-verb body
   compiles without firing `ChannelPairDeadlock` (run `cargo
   build` and verify clean).

6. **Honest deltas:** anything you needed to invent because
   Console's pattern didn't transcribe directly. The new
   Reply enum is the load-bearing addition; if you needed
   anything else surface it.

7. **LOC delta:** rough line-count change in each file. The
   substrate file should grow ~30-50 LOC (new typealiases) and
   shrink ~30-50 LOC (Request enum simplification + helper-verb
   simplification) — net near zero. The test file should shrink
   significantly (no per-call channel allocations).

## What this brief is testing (meta)

Per `REALIZATIONS.md`'s artifacts-as-teaching discipline, this
brief tests whether the artifacts (DESIGN + ZERO-MUTEX +
SERVICE-PROGRAMS.md + Console's existing substrate) compose
into a teaching that gets a sonnet sweep to ship a SUBSTRATE-
RESHAPE arc. Previous arcs in this chain proved structural-
rule arcs (arc 126), substrate-fix arcs (arc 128 / 129 /
133 / 134), AND wat-test annotation arcs (arc 126 slice 2)
propagate via the artifacts. Slice 1 of arc 130 is the first
SERVICE-REDESIGN arc. If it ships clean, the discipline scales
to substrate reshapes too.

The reshape is bounded but real (~300-500 LOC). Take time to
read Console's pattern at `wat/console.wat` carefully before
editing. The clarity of the existing reference is what makes
this brief feasible in one sweep.

## Sequencing — what to do, in order

1. Read DESIGN.md cover to cover.
2. Read `wat/console.wat` (~298 LOC) — the working reference.
3. Read `crates/wat-lru/wat/lru/CacheService.wat` (~456 LOC)
   to understand the current substrate shape.
4. Read `crates/wat-lru/wat-tests/lru/CacheService.wat` (~265
   LOC) — note the 4-helper / 5-deftest layered structure
   per arc 130 REALIZATIONS.md (file's own header documents
   this).
5. Read arc 131's INSCRIPTION + SERVICE-PROGRAMS.md § "The
   lockstep" — confirm the inner-let* nesting is ALREADY
   applied; you do NOT need to add it.
6. Run `cargo test --release -p wat-lru` to see the baseline
   (12 passed, 5 should-panic markers on the deadlock-class
   tests).
7. Reshape `crates/wat-lru/wat/lru/CacheService.wat` substrate
   in place (typealiases, Spawn, Request enum, helper-verbs,
   spawn factory body, driver loop). Mirror Console's
   pair-by-index pattern.
8. Reshape `crates/wat-lru/wat-tests/lru/CacheService.wat`
   helpers in place: 4 helper updates (Layer 0 typealias only;
   Layers 1a/1b/2 drop channel-pair allocations + update verb
   calls) + drop 5 `:should-panic` annotations.
9. Run `cargo test --release -p wat-lru` — verify all 12
   tests pass cleanly (0 should-panic markers in output).
10. Run `cargo test --release --workspace 2>&1 | grep "test
    result"` — verify exit=0, ~103 `test result: ok` lines,
    1820 passed total, 1 ignored, 14 should-panic markers
    remaining (was 19; the 5 LRU markers retired).
11. Run `cargo build --release 2>&1 | tail -10` — verify no
    `ChannelPairDeadlock` or `ScopeDeadlock` errors fire on
    the new helper-verb bodies or the reshaped helpers.
12. Report per the "Reporting back" section.

Then DO NOT commit. Working tree stays modified for the
orchestrator to score.
