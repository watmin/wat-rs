# Arc 130 Slice 1 — Pre-handoff expectations

**Refreshed 2026-05-02 (evening)** after arc 135 slice 1's
complectens sweep reshaped the LRU test file from a single
round-trip into a 4-helper / 5-deftest compositional structure
(per arc 130 REALIZATIONS.md). Workspace baseline re-locked
against the post-arc-135 state.

**Refreshed 2026-05-02 (early)** after the deadlock-class
chain (arcs 131 / 132 / 133 / 134) shipped. Restart with
post-chain context.

**Brief:** `BRIEF-SLICE-1.md`
**Output:** `crates/wat-lru/wat/lru/CacheService.wat` +
`crates/wat-lru/wat-tests/lru/CacheService.wat` modifications +
~250-word written report.

## Setup — workspace state pre-spawn (verified 2026-05-02 evening)

- LRU substrate uses pre-arc-130 shape (HandlePool<ReqTx>,
  per-verb reply types, embedded reply-tx in Request).
- LRU test file: 4-helper / 5-deftest compositional structure
  per arc 135 slice 1's complectens shape (file header
  documents the layers). All 5 deftests carry
  `:should-panic("channel-pair-deadlock")` because the shared
  prelude includes Layer 1+ helpers with `make-bounded-channel`
  calls. The 200ms default time-limit (arc 132) is in force;
  no explicit `:time-limit` annotations on the file.
- HolonLRU's tests: similar `:should-panic` shape on 5 tests +
  1 in `proofs/arc-119/step-B-single-put.wat`. Slice 2 of arc
  130 (separate session) reshapes HolonLRU.
- Arc 131's check fires on `HandlePool<Handle<K,V>>` siblings
  to `Thread/join-result` — the existing helpers ALREADY use
  inner-let* nesting per SERVICE-PROGRAMS.md § "The lockstep"
  (arc 131 slice 2 swept it across all consumer tests + arc
  135 preserved it through the layered split).
- Workspace test green:
  - `cargo test --release --workspace`: exit=0
  - 103 `test result: ok` lines
  - **1820 passed**, 0 failed, **1 ignored** (arc-122 mechanism)
  - **19 `should panic` markers** total across the workspace
- LRU package baseline: `cargo test --release -p wat-lru` →
  12 passed, 0 failed, 0 ignored, **5 `should panic` markers**
  on the deadlock-class deftests.

## Hard scorecard (11 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Two-file diff | Exactly 2 files modified: `crates/wat-lru/wat/lru/CacheService.wat` (substrate) + `crates/wat-lru/wat-tests/lru/CacheService.wat` (test). No HolonLRU files. No documentation. No Rust files. |
| 2 | Reply enum minted | `:wat::lru::Reply<V>` enum present with `(GetResult (results :Vector<Option<V>>))` and `(PutAck)` variants. Verifiable via `grep -n "wat::lru::Reply" crates/wat-lru/wat/lru/CacheService.wat`. |
| 3 | Handle / DriverPair typealiases minted | Both present. `Handle<K,V>` = `(ReqTx<K,V>, ReplyRx<V>)`; `DriverPair<K,V>` = `(ReqRx<K,V>, ReplyTx<V>)`. |
| 4 | Spawn typealias reshape | `:wat::lru::Spawn<K,V>` body changes from `HandlePool<ReqTx<K,V>>` to `HandlePool<Handle<K,V>>`. |
| 5 | Request enum simplification | `Request<K,V>` enum no longer carries embedded `reply-tx` or `ack-tx` fields. Variants are `(Get (probes :Vec<K>))` and `(Put (entries :Vec<Entry<K,V>>))`. |
| 6 | PutAck typealiases retired | `:wat::lru::PutAckTx`, `:PutAckRx`, `:PutAckChannel` no longer present in the substrate file. `grep -c "PutAck" crates/wat-lru/wat/lru/CacheService.wat` should return 0 OR refer only to the new `Reply::PutAck` variant. |
| 7 | Helper-verb signatures reshape | `:wat::lru::get<K,V>` and `:wat::lru::put<K,V>` take `(handle :Handle<K,V>)` as their first parameter, NOT `(req-tx, reply-tx, reply-rx)` or `(req-tx, ack-tx, ack-rx)`. |
| 8 | **`cargo test --release -p wat-lru`** | Exit=0; **12 passed; 0 failed; 0 ignored; 0 should-panic markers in output.** ALL 5 deadlock-class tests (`test-lru-spawn-and-shutdown`, `test-lru-spawn-then-put`, `test-lru-spawn-then-get`, `test-lru-spawn-put-then-get`, `test-cache-service-put-then-get-round-trip`) report plain `... ok` — NOT `... ok (should panic)`. The Final test's `assert-eq` passes (`Some Some 42`). |
| 9 | **`cargo test --release --workspace`** | Exit=0; **1820 passed; 0 failed; 1 ignored** (arc-122 mechanism); ~103 `test result: ok` lines; **14 `should panic` markers remaining** (was 19 pre-slice; the 5 LRU markers retired; the 14 in HolonLRU + step-B + others stay until slice 2). |
| 10 | **Complectens preserved** | The 4-helper / 5-deftest layered structure of the test file STAYS. No helpers merged. No layers collapsed. No new `make-deftest` factory needed. Deftest names unchanged. Layer 0 helper still does pure lifecycle (no `make-bounded-channel`, just spawn → pop-then-finish → join). Layers 1a/1b/2 helpers update verb signatures + drop channel-pair allocations. Verifiable: `grep -c "deftest" crates/wat-lru/wat-tests/lru/CacheService.wat` returns same count as baseline (5). |
| 11 | Honest report | 200-word report includes: file:line refs for the new typealiases + the reshaped Spawn + Request enum + helper-verb signatures; the exact final form of the Reply enum + Handle typealias + get signature; the driver-loop reshape note (how DriverPair is indexed); test totals; arc-126 check status; honest deltas (anything Console's pattern didn't directly transcribe). |

**Hard verdict:** all 11 must pass for slice 1 to ship clean.
Rows 8 + 9 are load-bearing for runtime correctness; row 10 is
load-bearing for the test-shape discipline (complectens
preservation — the layered structure stays).

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

- **Most likely (~55%):** all 11 hard + 5-6 soft pass cleanly.
  Console's pattern transcribes well; the test file's existing
  layered structure makes the test-side reshape mechanical
  (4 helper updates + 5 annotation drops); Reply enum
  unification is the only real new substrate concept; sonnet
  ships in 10-20 min. The arc-135-shipped layered shape +
  /complectens SKILL pre-read gives sonnet a clear discipline
  to preserve.
- **Second-most-likely (~25%):** 10-11 hard pass + soft drift
  on Console-pattern-fidelity (sonnet invents a slightly
  different shape because Console doesn't have a multi-verb
  Reply pattern). Outcome still committable.
- **Driver-loop reshape difficulty (~12%):** 9 of 11 hard pass;
  the driver-loop's pair-by-index logic doesn't quite match
  Console's pattern (Console is single-verb-unit-reply; cache
  is multi-verb-with-Reply-enum-dispatch). Sonnet may need to
  invent a shape that select-by-index + dispatch-by-Reply-
  variant composes correctly. If row 8 (LRU green) fails,
  this is the failure mode; surface in the report and we open
  a follow-on arc.
- **Complectens drift (~5%):** sonnet collapses the layered
  structure (merges helpers, drops a deftest, restructures the
  factory). Row 10 fails; row 8 may still pass. The reland
  brief encodes the discipline more emphatically.
- **Type-system surprise (~3%):** the `Reply<V>` enum's
  parametric instantiation might not unify cleanly with
  generic Sender<...> typing in some place we didn't
  anticipate. Sonnet would surface this with a specific
  compile error; another arc opens.

## Methodology

After the agent reports back, the orchestrator MUST:

1. Read this file FIRST.
2. Score each row of both scorecards explicitly.
3. Diff via `git diff --stat` (expect 2 files, both LRU).
4. Verify hard rows 2-7 by `grep -n` for the new typealiases
   in the substrate file.
5. Verify hard rows 8 + 9 by reading the cargo-test totals
   from the agent's report (and re-running locally to confirm).
6. Verify hard row 10 (complectens preserved) by `grep -c
   "deftest" crates/wat-lru/wat-tests/lru/CacheService.wat` →
   should equal baseline 5.
7. Verify hard row 11 by reading the report itself for
   completeness.
8. Score; commit SCORE-SLICE-1.md as a sibling.

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
