# Arc 130 Slice 1 — Sonnet Brief

**Refreshed 2026-05-02** — the arc was paused 2026-05-01 when
the original sweep killed mid-run; the deadlock-class chain
(arcs 131 / 132 / 133 / 134) shipped in the interim. This
brief is the post-chain restart and supersedes the original
2026-05-01 framing. Arc 130 itself is unchanged in goal; only
the surrounding substrate context shifted.

**Goal:** reshape `:wat::lru::*` (the LRU CacheService) substrate
+ tests to use pair-by-index via HandlePool with a unified
`Reply<V>` enum. After this slice, the LRU's single deadlock-class
test (`test-cache-service-put-then-get-round-trip`) PASSES via the
new shape (no `:should-panic`; arc 126's check doesn't fire).

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Scope:** `:wat::lru::*` only. Slice 2 mirrors this work for
`:wat::holon::lru::*` in a separate session. Slice 3 is closure
docs.

**Console reference:** `wat/console.wat` (~298 LOC, in the
top-level wat tree, NOT inside `crates/`). This file IS the
working pair-by-index pattern. Read it cover-to-cover before
touching the LRU substrate.

## Read-in-order anchor docs

1. `docs/arc/2026/05/130-cache-services-pair-by-index/DESIGN.md`
   — the new typealiases, helper-verb signatures, driver
   reshape, and the four-questions framing. Source of truth.
2. `docs/arc/2026/05/126-channel-pair-deadlock-prevention/INSCRIPTION.md`
   — the rule arc 130 routes around. Specifically the
   "queued follow-ups" section names this redesign explicitly.
3. `docs/ZERO-MUTEX.md` § "Routing acks" — the canonical
   "pair-by-index vs embedded reply-tx" doctrine. Console is
   the reference implementation.
4. **Console's substrate** — find Console's wat files (likely
   `wat/console/Console.wat` or under `crates/wat-console/`).
   Console uses `Console::Handle = (Tx, AckRx)` with a
   HandlePool of pre-allocated handles. Slice 1 mirrors this
   pattern for the cache service.
5. `crates/wat-lru/wat/lru/CacheService.wat` — the file that
   reshapes. ~456 lines today. Read it fully before editing.
6. `crates/wat-lru/wat-tests/lru/CacheService.wat` — the test
   file that retires its `:should-panic` annotations and
   rewrites the test body to the new shape.

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

**REMOVE** the `:wat::test::should-panic
"channel-pair-deadlock"` annotation.
**REMOVE** the `:wat::test::time-limit "200ms"` annotation
(arc 132 made 200ms the default — explicit annotation is now
an override only needed when a test legitimately exceeds
200ms; the cache test should not). If the test takes longer
in practice, leave the annotation tuned to a realistic
budget; otherwise drop it.

**REWRITE** the test body to use the new helper-verb shape
**AND** the canonical inner-let* nesting pattern. Arc 131
makes `HandlePool<Handle<K,V>>` Sender-bearing structurally
(`Handle = (ReqTx, ReplyRx)` contains a Sender), so a let*
with `pool + driver + Thread/join-result driver` siblings will
fire `ScopeDeadlock` unless inner-let* nesting drops the pool
before the join. SERVICE-PROGRAMS.md § "The lockstep" is the
reference; arc 131 slice 2 swept other consumer tests to this
shape.

The canonical inner-let*:

```scheme
(:wat::test::deftest :wat-lru::test-cache-service-put-then-get-round-trip
  ()
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-hermetic-ast
        (:wat::test::program
          (:wat::core::define
            (:user::main ...)
            ;; OUTER scope holds ONLY the Thread.
            (:wat::core::let*
              (((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                ;; INNER scope owns pool + handle + all the work.
                (:wat::core::let*
                  (((state :wat::lru::Spawn<wat::core::String,wat::core::i64>)
                    (:wat::lru::spawn 16 1 ...))
                   ((pool :wat::kernel::HandlePool<wat::lru::Handle<wat::core::String,wat::core::i64>>)
                    (:wat::core::first state))
                   ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                    (:wat::core::second state))
                   ((handle :wat::lru::Handle<wat::core::String,wat::core::i64>)
                    (:wat::kernel::HandlePool::pop pool))
                   ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))

                   ;; Put one entry — handle replaces the old
                   ;; (req-tx, ack-tx, ack-rx) triple:
                   ((_ :wat::core::unit)
                    (:wat::lru::put handle
                      (:wat::core::conj
                        (:wat::core::Vector :wat::lru::Entry<wat::core::String,wat::core::i64>)
                        (:wat::core::Tuple "answer" 42))))

                   ;; Get the entry:
                   ((results :wat::core::Vector<wat::core::Option<wat::core::i64>>)
                    (:wat::lru::get handle
                      (:wat::core::conj
                        (:wat::core::Vector :wat::core::String)
                        "answer")))
                   ;; ... existing assertion logic on `results` ...
                   )
                  ;; Inner returns the Thread; pool + handle drop here.
                  d)))
              ;; Outer's only operation: join the now-disconnected driver.
              (:wat::kernel::Thread/join-result driver)))
          ...)))))
```

Two big changes vs the pre-arc-130 test:
1. No `make-bounded-channel` calls anywhere in the test body.
   The cache service owns the channels internally.
2. Pool + handle + work nest inside an INNER let*; the OUTER
   let* holds only the driver Thread; the OUTER body's only
   operation is `Thread/join-result driver`. Inner returns the
   Thread; on inner-scope exit, pool + handle drop, driver's
   recv-loop sees the channel disconnect and the thread exits;
   outer's join-result returns Ok.

This is the SERVICE-PROGRAMS.md § "The lockstep" canonical
shape. Arc 131 slice 2 already applied it across other
service-test files. Mirror that shape here.

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

1. `cargo test --release -p wat-lru --test test`: 8 tests
   passed, 0 failed, 0 ignored. The
   `test-cache-service-put-then-get-round-trip` test passes
   cleanly (no `should panic` marker).

2. `cargo test --release --workspace`: 100 `test result: ok`
   lines; 6 tests still in `:should-panic` state (the 5 in
   HolonLRU + step-B); 1 ignored (arc-122 mechanism).
   Workspace exit=0.

3. The LRU substrate file's typealias section reflects the
   new shape exactly per the DESIGN.

4. Helper-verb signatures match the DESIGN's "NEW" shape.

5. No `make-bounded-channel` calls remain in the LRU test
   file's test body.

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
2. Read `wat/console.wat` — the working reference.
3. Read `crates/wat-lru/wat/lru/CacheService.wat` (~456 LOC)
   to understand the current shape.
4. Read `crates/wat-lru/wat-tests/lru/CacheService.wat` (~143
   LOC) to understand the current test.
5. Read arc 131's INSCRIPTION + SERVICE-PROGRAMS.md § "The
   lockstep" to understand the inner-let* nesting that the
   rewritten test MUST use.
6. Run `cargo test --release -p wat-lru --test test 2>&1 |
   tail -20` to see the baseline (1 should-panic test, others
   passing).
7. Reshape the substrate file in place.
8. Reshape the test file in place using the canonical inner-
   let* pattern.
9. Run `cargo test --release -p wat-lru --test test 2>&1 |
   tail -20` — verify all 8 tests pass cleanly.
10. Run `cargo test --release --workspace 2>&1 | tail -3` —
    verify workspace stays green.
11. Run `cargo build --release 2>&1 | tail -10` — verify no
    `ChannelPairDeadlock` or `ScopeDeadlock` errors leak from
    the substrate or test files.
12. Report.

Then DO NOT commit. Working tree stays modified for the
orchestrator to score.
