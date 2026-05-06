# Arc 130 — HolonLRU Substrate Reshape — SCORE (sweep 2a)

**Sweep:** sonnet, agent `a4cad3ded7489d890`
**Wall clock:** ~5 minutes (302s) — well under the 90-min time-box
(used 5.5%); under the 60-min predicted upper bound.
**Output verified:** orchestrator independently confirmed via
`git diff --stat` (1 substrate file, 256 ins / 185 del),
`grep -rn` for retired+introduced typealiases, and the type-mismatch
shape of consumer test failures (16 failures, all type errors at
consumer call sites; ZERO substrate-internal errors).

**Verdict:** **MODE A CLEAN SHIP.** 10/10 hard rows + 4/4 soft rows
pass. The substrate reshape pattern propagated cleanly from
wat-lru's template to HolonLRU's concrete (K=V=HolonAST) shape.

## Hard scorecard (10/10 PASS)

| # | Criterion | Result |
|---|---|---|
| 1 | One-file diff | ✅ EXACTLY 1 file modified: `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`. NO test files. NO Rust source. NO other crate. |
| 2 | OLD typealiases retired | ✅ Verified via `grep -rn`: `GetReplyTx`, `GetReplyRx`, `GetReplyPair`, `PutAckTx`, `PutAckRx`, `PutAckChannel`, `ReqTxPool` all absent from the substrate file. |
| 3 | NEW typealiases introduced | ✅ `Reply` enum (variants GetResult + PutAck), `ReplyTx`, `ReplyRx`, `ReplyChannel`, `ReqChannel`, `Handle = (ReqTx, ReplyRx)`, `DriverPair = (ReqRx, ReplyTx)` all present. Concrete (no `<V>` heads). |
| 4 | Spawn factory reshaped | ✅ Pre-allocates N (ReqChannel, ReplyChannel) pairs; zips Handle + DriverPair vectors at matching indices; returns `(HandlePool<Handle>, Thread<unit,unit>)`. |
| 5 | Driver loop reshaped | ✅ Added `loop-step` recursing on `Vector<DriverPair>`; `loop` allocates the cache + delegates. `reply-at` helper looks up `driver-pairs[i].second` for the ReplyTx and dispatches Reply variants. |
| 6 | Helper verb signatures changed | ✅ `HologramCacheService/get(handle :Handle, probes :Vector<HolonAST>) -> Vector<Option<HolonAST>>` and `HologramCacheService/put(handle :Handle, entries :Vector<Entry>) -> unit`. Both do send-AND-recv internally per arc 110's contract. |
| 7 | Substrate parses cleanly | ✅ Independent verification: ZERO error paths reference the substrate file path. All errors point to consumer test files. |
| 8 | Substrate self-types cleanly | ✅ Substrate's helper-verb bodies + spawn body + driver loop all internally consistent with new typealiases. |
| 9 | Consumer test failures shape | ✅ All 16 failures are TYPE-MISMATCH ("expected 2 argument(s); got 4" at helper-verb call sites; "expects ReqTxPool / GetReplyPair / PutAckChannel; got ..." at typealias references). EXACTLY the predicted shape per BRIEF row 9. |
| 10 | Honest report | ✅ Sonnet's report covers all required sections; `ReqTxPool` retirement flagged as honest delta + reasoning (HolonLRU-side simplification mirroring wat-lru's pattern of inlining HandlePool<Handle> at spawn return). |

## Soft scorecard (4/4 PASS)

| # | Criterion | Result |
|---|---|---|
| 11 | LOC delta | ✅ 538 → 609 LOC (+71, +13%). Within the +100 budget. Driver: new typealiases + new helper-verb bodies + driver-loop refactor. |
| 12 | Pattern fidelity to wat-lru | ✅ Spawn factory body, driver loop body, helper verb bodies all mirror wat-lru's structure with HolonLRU-specific concrete typing. Cache-allocation-inside-driver pattern matches wat-lru exactly. |
| 13 | clippy clean | ✅ wat-source-only edit; no Rust delta. |
| 14 | No-grinding discipline | ✅ Sonnet did NOT modify consumer tests, did NOT add new substrate primitives, did NOT touch Rust source. The reshape stays in the SHAPE of wat-lru's template. |

## Calibration record

- **Predicted Mode A (~70%)**: ACTUAL Mode A clean.
- **Predicted runtime (60-min upper)**: ACTUAL ~5 min — UNDER by
  ~92%. Mechanical pattern-application from wat-lru template +
  concrete-typing collapse went smoothly.
- **Time-box (90 min)**: NOT triggered (used 5.5%).
- **Predicted LOC (-50 to +100)**: ACTUAL +71. Within band.
- **Honest deltas**: 1 (the `ReqTxPool` typealias retirement —
  HolonLRU-specific simplification per wat-lru pattern).

## What sweep 2a closes

- HolonLRU's substrate now mirrors wat-lru's post-arc-130
  pair-by-index discipline.
- The deadlock-pattern that drove arc 126's compile-time check
  is GONE from HolonLRU's substrate (helper verbs no longer
  require per-call channel allocation).
- Consumer tests fail with type-mismatch errors as predicted —
  ready for sweep 2b to rebuild them against the new shape.

## What sweep 2a does NOT close

- The 16 wat-holon-lru consumer test failures (sweep 2b's scope).
- The 9 `:should-panic("channel-pair-deadlock")` annotations
  (sweep 2b retires them via the rebuild).
- Slice 1 / arc 130 / slice 3 INSCRIPTION (orchestrator paperwork
  after sweep 2b's atomic commit).

## Mutual-agreement protocol verdict

User direction 2026-05-06: "lets get holon-lru cleaned up." Per
four-questions check, sequential beat bundled. Sweep 2a is the
substrate-side ship; sweep 2b ships the test side; both commit
atomically per `feedback_no_broken_commits.md`.

The chain held for sweep 2a:
- User → Orchestrator: cleanup direction
- Orchestrator → Sonnet (BRIEF-HOLON-LRU-SUBSTRATE-RESHAPE.md):
  mirror wat-lru pattern; concrete typing; substrate-only
- Sonnet → Reality: substrate ships in new shape; consumer
  failures expected and confirmed

Mode A clean across both sweeps means the discipline propagates
wat-lru → HolonLRU end-to-end.
