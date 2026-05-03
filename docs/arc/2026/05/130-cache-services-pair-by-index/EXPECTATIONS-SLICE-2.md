# Arc 130 Slice 2 — Pre-handoff expectations

**Drafted 2026-05-02 (evening)** assuming slice 1 ships
clean. After slice 1's SCORE arrives, this file may receive a
calibration refresh if slice 1's actual outcome diverges from
the prediction. The structural shape (3-file diff, two-prelude
preservation, 9 annotations dropped) is locked.

**Brief:** `BRIEF-SLICE-2.md`
**Output:** 3-file diff (HCS substrate + HCS test + step-B
proof) + ~250-word written report.

## Setup — workspace state pre-spawn (assuming slice 1 shipped)

- LRU substrate uses POST-arc-130-slice-1 shape (HandlePool<Handle>,
  unified Reply<V> enum, simplified Request enum, helper-verbs
  take Handle).
- LRU test file uses POST-slice-1 shape (4 layered helpers
  updated; 5 :should-panic annotations dropped).
- HolonLRU substrate uses pre-arc-130 shape (HandlePool<ReqTx>,
  GetReply* + PutAck* dual reply families, embedded reply-tx
  + ack-tx in Request enum).
- HolonLRU test file uses arc 130's prior complectens
  two-prelude shape (`:deftest-hermetic` + `:deftest-service`
  factories). 8 deftests in `:deftest-service` carry
  `:should-panic("channel-pair-deadlock")`.
- step-B-single-put.wat: 1 deftest, carries
  `:should-panic("channel-pair-deadlock")`.
- Workspace test green (post-slice-1):
  - `cargo test --release --workspace`: exit=0
  - 1820 passed, 0 failed, 1 ignored (arc-122 mechanism)
  - **14 `should panic ... ok` markers total** (was 19
    pre-slice-1; the 5 LRU markers retired)
  - Of the 14: **9 are `channel-pair-deadlock`** (8 HCS + 1
    step-B; slice 2 retires all 9); 5 are unrelated
    (`tmp-totally-bogus.wat`, wat-edn rust tests).
- HolonLRU package baseline: `cargo test --release -p
  wat-holon-lru` → 26 passed, 0 failed, 0 ignored, **9 `should
  panic` markers** on the 8 service deftests + step-B proof.

## Hard scorecard (12 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Three-file diff | Exactly 3 files modified: `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat` (substrate) + `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` (main test) + `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat` (proof). No LRU files. No HologramCache.wat. No documentation. No Rust files. |
| 2 | Reply enum minted | `:wat::holon::lru::HologramCacheService::Reply` enum present with `(GetResult (results :Vector<Option<HolonAST>>))` and `(PutAck)` variants. Verifiable via `grep -n "HologramCacheService::Reply" crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`. |
| 3 | Handle / DriverPair typealiases minted | Both present. `Handle` = `(ReqTx, ReplyRx)`; `DriverPair` = `(ReqRx, ReplyTx)`. |
| 4 | Spawn typealias reshape | `:wat::holon::lru::HologramCacheService::Spawn` body changes from `HandlePool<ReqTx>` to `HandlePool<Handle>`. |
| 5 | Request enum simplification | `Request` enum no longer carries embedded `reply-tx` or `ack-tx` fields. Variants are `(Get (probes :Vector<HolonAST>))` and `(Put (entries :Vector<Entry>))`. |
| 6 | PutAck + GetReply typealiases retired | `:HologramCacheService::PutAckTx`, `:PutAckRx`, `:PutAckChannel`, `:GetReplyTx`, `:GetReplyRx`, `:GetReplyPair` no longer present in the substrate file (other than as Reply::PutAck variant or the new ReplyTx/Rx names). |
| 7 | Helper-verb signatures reshape | `HologramCacheService/get` and `HologramCacheService/put` take `(handle :Handle)` as their first parameter, NOT `(req-tx, reply-tx, reply-rx)` or `(req-tx, ack-tx, ack-rx)`. |
| 8 | **`cargo test --release -p wat-holon-lru`** | Exit=0; **26 passed; 0 failed; 0 ignored; 0 should-panic markers in output**. ALL 8 prelude-2 service deftests + the step-B proof report plain `... ok`. The 6 hermetic deftests + step-A proof + 4 HologramCache tests stay green. |
| 9 | **`cargo test --release --workspace`** | Exit=0; **1820 passed; 0 failed; 1 ignored** (arc-122 mechanism); ~103 `test result: ok` lines; **0 `channel-pair-deadlock` markers remaining workspace-wide** (was 9 pre-slice-2; the 8 HCS + 1 step-B markers retired). Other unrelated `:should-panic` markers (5 total: tmp-totally-bogus + wat-edn rust tests + others) stay. Total `should panic ... ok` count drops to 5. |
| 10 | **Two-prelude pattern preserved** | Both `:deftest-hermetic` (lines ~67-249) and `:deftest-service` (lines ~308-628) `make-deftest` factories STILL PRESENT. Verifiable via `grep -c "make-deftest" crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` returns 2. Deftest count unchanged: `grep -c "deftest-hermetic\|deftest-service" ...` returns 14 (6 hermetic deftests + 8 service deftests). No factory merge. |
| 11 | **Complectens layered structure preserved** | All Layer 0 / Layer 1 / Layer 2 helpers remain named the same. No helpers merged, no layers collapsed. Verifiable: `grep -c "^(:wat::core::define" crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` returns same baseline count. The reshape removes channel-pair-allocation bindings inside helpers, not the helpers themselves. |
| 12 | Honest report | 250-word report includes: file:line refs for the new typealiases + the reshaped Spawn + Request enum + helper-verb signatures; the exact final form of the Reply enum + Handle typealias + get signature; two-prelude preservation note; test totals (HolonLRU + workspace + workspace channel-pair-deadlock count); arc-126 check status; honest deltas (anything HolonAST specialization required beyond slice 1's pattern). |

**Hard verdict:** all 12 must pass for slice 2 to ship clean.
Rows 8 + 9 are load-bearing for runtime correctness. Rows 10
+ 11 are load-bearing for the test-shape discipline
(complectens preservation; two-prelude split must NOT collapse
just because both preludes now run clean).

## Soft scorecard (7 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 13 | LOC budget | Substrate file: ~50-80 LOC additions, ~50-80 LOC deletions, net ~0 (similar size; HCS is bigger than LRU since it consolidates two reply families into one Reply enum). Test file: significant shrinkage in prelude-2 helpers (no per-call channel allocations). step-B: small reshape (~10 LOC delta). Total diff in 200-500 LOC range. >700 LOC = re-evaluate. |
| 14 | Slice 1 pattern reuse | Sonnet's report references slice 1's shipped diff explicitly + describes how slice 2 mirrors it. Honest delta surfaced for any HolonAST-specialization detail slice 1's `<K,V>` parametric pattern didn't directly transcribe. |
| 15 | Driver-pair indexing | The driver's pair-by-index logic mirrors slice 1's chosen shape (whatever it was). Sonnet describes it in the report. |
| 16 | Helper-verb body shape | Helper-verb body projects `(first handle)` and `(second handle)` to get ReqTx + ReplyRx. Sends Request, recvs Reply, matches Reply variant. Identical pattern to slice 1's helpers. |
| 17 | step-B proof shape preserved | step-B-single-put.wat's deftest body remains a narrow single-put assertion (the proof's purpose unchanged). Only the channel-pair allocation + verb call signature update. |
| 18 | No new public API | No new `pub` exports. No new file. Only the existing typealias section + signatures change. |
| 19 | Workspace test runtime | Total `cargo test --release --workspace` runtime stays under 60s. No regression vs slice-1 baseline. |

## Independent prediction

Before reading the agent's output, the orchestrator predicts:

- **Most likely (~60%):** all 12 hard + 6-7 soft pass cleanly.
  Slice 1's worked diff is the template; slice 2 is mechanical
  mirroring with HolonAST specialization + 3 files instead of
  2; sonnet ships in 8-15 min (faster than slice 1 because the
  pattern is already proven). The two-prelude preservation is
  obvious from the file's own header documentation.
- **Second-most-likely (~20%):** 11-12 hard pass + soft drift
  on the LOC budget (HCS substrate is bigger than LRU; the
  Reply enum unification consolidates GetReply* + PutAck*
  families which is more substantial than LRU's PutAck-only
  retirement). Outcome still committable.
- **HolonAST specialization surprise (~10%):** the
  `Vector<Option<HolonAST>>` typing in the Reply::GetResult
  variant might surface a HolonAST-specific type-check
  edge case (e.g., the substrate's polymorphic Atom typing
  per arc 057). Sonnet would flag a specific compile error;
  if hard, opens follow-on arc.
- **Two-prelude collapse (~5%):** sonnet "simplifies" by
  merging the two preludes since both now pass cleanly. Row 10
  fails. Rerun with sharper brief that quotes the SKILL's
  mixed-outcome edge case verbatim.
- **Driver-loop divergence (~3%):** the HCS driver loop's
  pair-by-index logic differs from slice 1's in some way
  (e.g., the multi-client fan-in pattern requires different
  selection). Surface in report; may open follow-on arc.
- **Type-system surprise (~2%):** as in slice 1, generic
  Sender<...> + Reply<V> instantiation might trip an unforeseen
  type-checker case. Surface compile error.

## Methodology

After the agent reports back, the orchestrator MUST:

1. Read this file FIRST.
2. Score each row of both scorecards explicitly.
3. Diff via `git diff --stat` (expect 3 files, all under
   `crates/wat-holon-lru/`).
4. Verify hard rows 2-7 by `grep -n` for the new typealiases
   in the substrate file.
5. Verify hard rows 8 + 9 by reading the cargo-test totals
   from the agent's report (and re-running locally to confirm).
6. Verify hard row 10 (two-prelude preserved) via `grep -c
   "make-deftest" crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
   → should return 2.
7. Verify hard row 11 (complectens layers preserved) via
   helper-define grep against baseline.
8. Verify hard row 12 by reading the report itself for
   completeness.
9. Score; commit SCORE-SLICE-2.md as a sibling.

## Why this slice matters for the chain

Slice 2 is the second instance of the substrate-redesign-via-
sonnet pattern. Slice 1 proved artifacts-as-teaching scales
to substrate reshapes. Slice 2 proves the calibration TRANSFERS:
sonnet, given slice 1's shipped diff as a worked template +
the HolonLRU specifics, ships the parallel reshape clean.

If slice 2 ships in significantly less time than slice 1
(say, 50%), that's strong evidence the worked-template
pattern + the artifacts-as-teaching cascade compound across
sessions. The arc-126 reland (7 min vs 13.5 min for sweep 1)
was the first instance of this compound; slice 2's would be
the second.

## What we learn

- **All hard pass:** the substrate-redesign discipline scales
  AND transfers; slice 2 inherits slice 1's calibration
  cleanly. Slice 3 (closure docs) becomes mechanical.
- **Row 8 fails (HolonLRU not green):** the HolonAST
  specialization surfaces a gap slice 1 didn't hit. Diagnose
  via the failing test's panic; open follow-on arc; refresh
  brief.
- **Row 10 fails (two-prelude collapsed):** sonnet over-
  simplified. The reland brief quotes the SKILL's
  mixed-outcome edge case verbatim + explicitly forbids the
  merge.
- **Row 11 fails (helpers merged):** same — reland with
  sharper brief.
- **Row 9 fails (workspace not green outside HolonLRU):**
  the HolonLRU reshape leaked into LRU somehow (shared
  symbols). Surface and fix; the slice boundary should have
  been cleaner.

After slice 2 passes, slice 3 (closure: INSCRIPTION + 058
changelog row + INVENTORY § K mark + cross-references) is
mechanical paperwork. Arc 130 then closes; arc 109's
K.holon-lru slice unblocks; the workspace's
`channel-pair-deadlock` counter is permanently zero.
