# Arc 130 Slice 1 — Pre-handoff expectations

**Written:** 2026-05-01, AFTER spawning the sonnet agent and
BEFORE its first deliverable. Durable scorecard.

**Brief:** `BRIEF-SLICE-1.md`
**Output:** `crates/wat-lru/wat/lru/CacheService.wat` +
`crates/wat-lru/wat-tests/lru/CacheService.wat` modifications +
~200-word written report.

## Setup — workspace state pre-spawn

- LRU substrate uses pre-arc-130 shape (HandlePool<ReqTx>,
  per-verb reply types, embedded reply-tx in Request).
- LRU's single test carries `:should-panic("channel-pair-deadlock")`
  + `:time-limit "200ms"` (slice 2 of arc 126).
- HolonLRU's tests: same `:should-panic` shape on 5 tests + 1
  in proofs/. Slice 2 of arc 130 (separate session) reshapes
  HolonLRU.
- Workspace test green: `cargo test --release --workspace`
  exit=0; 6 should-panic tests pass via the panic; 1 ignored.

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Two-file diff | Exactly 2 files modified: `crates/wat-lru/wat/lru/CacheService.wat` (substrate) + `crates/wat-lru/wat-tests/lru/CacheService.wat` (test). No HolonLRU files. No documentation. No Rust files. |
| 2 | Reply enum minted | `:wat::lru::Reply<V>` enum present with `(GetResult (results :Vector<Option<V>>))` and `(PutAck)` variants. Verifiable via `grep -n "wat::lru::Reply" crates/wat-lru/wat/lru/CacheService.wat`. |
| 3 | Handle / DriverPair typealiases minted | Both present. `Handle<K,V>` = `(ReqTx<K,V>, ReplyRx<V>)`; `DriverPair<K,V>` = `(ReqRx<K,V>, ReplyTx<V>)`. |
| 4 | Spawn typealias reshape | `:wat::lru::Spawn<K,V>` body changes from `HandlePool<ReqTx<K,V>>` to `HandlePool<Handle<K,V>>`. |
| 5 | Request enum simplification | `Request<K,V>` enum no longer carries embedded `reply-tx` or `ack-tx` fields. Variants are `(Get (probes :Vec<K>))` and `(Put (entries :Vec<Entry<K,V>>))`. |
| 6 | PutAck typealiases retired | `:wat::lru::PutAckTx`, `:PutAckRx`, `:PutAckChannel` no longer present in the substrate file. `grep -c "PutAck" crates/wat-lru/wat/lru/CacheService.wat` should return 0 OR refer only to the new `Reply::PutAck` variant. |
| 7 | Helper-verb signatures reshape | `:wat::lru::get<K,V>` and `:wat::lru::put<K,V>` take `(handle :Handle<K,V>)` as their first parameter, NOT `(req-tx, reply-tx, reply-rx)` or `(req-tx, ack-tx, ack-rx)`. |
| 8 | **`cargo test --release -p wat-lru --test test`** | Exit=0; 8 passed; 0 failed; 0 ignored. The `test-cache-service-put-then-get-round-trip` test reports `... ok` (NOT `... ok (should panic)`). |
| 9 | **`cargo test --release --workspace`** | Exit=0; 100 `test result: ok` lines (or close); HolonLRU's 6 tests STILL `should panic` (not affected by this slice). 1 ignored (arc-122 mechanism test). |
| 10 | Honest report | 200-word report includes: file:line refs for the new typealiases + the reshaped Spawn + Request enum + helper-verb signatures; the exact final form of the Reply enum + Handle typealias + get signature; the driver-loop reshape note (how DriverPair is indexed); test totals; arc-126 check status; honest deltas (anything Console's pattern didn't directly transcribe). |

**Hard verdict:** all 10 must pass for slice 1 to ship clean.
Row 8 + 9 are load-bearing — they validate the redesign at
runtime end-to-end.

## Soft scorecard (6 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget | Substrate file: ~30-50 LOC additions, ~30-50 LOC deletions, net ~0 (similar size). Test file: significant shrinkage (no per-call channel allocations). Total diff in 100-300 LOC range. >500 LOC = re-evaluate. |
| 12 | Console pattern reference | Sonnet's report references Console's substrate file by path and describes how the cache service mirrors its pattern. Honest delta surfaced if Console's pattern needed adaptation for the multi-verb Reply enum. |
| 13 | Driver-pair indexing | The driver's pair-by-index logic works (whatever shape: select-by-index, via payload carrying index, or some equivalent). Sonnet describes the chosen shape clearly. |
| 14 | Helper-verb body shape | Helper-verb body projects `(first handle)` and `(second handle)` to get ReqTx + ReplyRx. Sends Request, recvs Reply, matches Reply variant. Mirrors the DESIGN's worked-out body. |
| 15 | No new public API | No new `pub` exports. No new file. Only the existing typealias section + signatures change. |
| 16 | Workspace test runtime | Total `cargo test --release --workspace` runtime stays under 60s. The redesign shouldn't slow anything down. |

## Independent prediction

Before reading the agent's output, the orchestrator predicts:

- **Most likely (~50%):** all 10 hard + 5-6 soft pass cleanly.
  Console's pattern transcribes well; the Reply enum unification
  is the only real new substrate concept; sonnet ships in
  10-20 min.
- **Second-most-likely (~30%):** 9-10 hard pass + soft drift on
  Console-pattern-fidelity (sonnet invents a slightly different
  shape because Console doesn't have a multi-verb Reply pattern).
  Outcome still committable.
- **Driver-loop reshape difficulty (~12%):** 8 of 10 hard pass;
  the driver-loop's pair-by-index logic doesn't quite match
  Console's pattern (Console is single-verb-unit-reply; cache is
  multi-verb-with-Reply-enum-dispatch). Sonnet may need to
  invent a shape that select-by-index + dispatch-by-Reply-variant
  composes correctly. If row 8 (LRU green) fails, this is the
  failure mode; surface in the report and we open arc 131.
- **Type-system surprise (~8%):** the `Reply<V>` enum's
  parametric instantiation might not unify cleanly with
  generic Sender<...> typing in some place we didn't anticipate.
  Sonnet would surface this with a specific compile error;
  another arc opens.

## Methodology

After the agent reports back, the orchestrator MUST:

1. Read this file FIRST.
2. Score each row of both scorecards explicitly.
3. Diff via `git diff --stat` (expect 2 files, both LRU).
4. Verify hard rows 2-7 by `grep -n` for the new
   typealiases.
5. Verify hard rows 8 + 9 by reading the cargo-test totals
   from the agent's report.
6. Verify hard row 10 by reading the report itself for
   completeness.
7. Score; commit SCORE-SLICE-1.md as a sibling.

## Why this slice matters for the chain

Arc 130 is the FIRST service-redesign arc in the
failure-engineering chain. Previous arcs were:
- Structural-rule arcs (arc 126)
- Substrate-fix arcs (arc 128 — check walker; arc 129 — proc
  macro)
- Annotation conversion arcs (arc 126 slice 2)

Slice 1 of arc 130 tests whether the artifacts-as-teaching
discipline scales to substrate reshapes. Console's existing
pattern provides the reference; the DESIGN names the new
typealiases precisely; the brief is file-and-line-specific.
If sonnet ships clean, the discipline holds for substrate
work too.

## What we learn

- **All hard pass:** discipline scales fully; substrate
  redesigns ship via the same pattern. Slice 2 (HolonLRU)
  inherits the calibration.
- **Row 8 fails (LRU not green):** the redesign's runtime
  shape is wrong. Diagnose via the failing test's panic;
  open follow-on arc.
- **Row 9 fails (workspace not green):** the LRU reshape
  leaked into HolonLRU somehow (shared symbols / types).
  Surface and fix; the slice 1 boundary should have been
  cleaner.

The next sweep timing matters: arc 126 first sweep took 13.5
min, arc 129 took 2.5 min, slice 2 of arc 126 took 5.3 min.
Arc 130 is bigger (substrate + tests reshape, two crates'
shape changes); estimating 10-25 min wall-clock.
