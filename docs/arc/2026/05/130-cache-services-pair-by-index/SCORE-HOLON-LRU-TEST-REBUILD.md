# Arc 130 — HolonLRU Test Rebuild — SCORE (sweep 2b)

**Sweep:** sonnet, agent `ad4728aa4adf7a676`
**Wall clock:** ~7 minutes (417s) — well under the 90-min time-box
(used 7.7%); under the 60-min predicted upper bound.
**Output verified:** orchestrator independently confirmed via
`git diff --stat` (3 test files + 1 substrate from sweep 2a),
full workspace `cargo test --release` (0 failed across all crates),
grep for `:should-panic` (only arc-122 mechanism self-test
remains; all 9 LRU annotations retired), and spot-check of the
rebuilt main test file's factory + 7 deftests structure.

**Verdict:** **MODE A CLEAN SHIP.** 12/12 hard rows + 3/4 soft
rows pass; 1 soft row (LOC) is mixed at 501 vs 200-450 band, +11%
over, explicitly justified by the optional Layers 5-6 the BRIEF
sanctioned for HolonLRU-specific scenarios.

**Workspace milestone (preserved)**: post-sweep-2b, the entire
workspace test suite remains at **0 failed tests** — the
substrate reshape (sweep 2a) + test rebuild (sweep 2b) hold the
clean baseline established this afternoon.

## Hard scorecard (12/12 PASS)

| # | Criterion | Result |
|---|---|---|
| 1 | File diff scope | ✅ EXACTLY 4 files in working tree: 1 substrate (sweep 2a) + 3 test files (this sweep). NO Rust source. NO other crate. |
| 2 | Main test file rebuilt from empty | ✅ Prior 731 LOC structure GONE; replaced with `(:wat::test::make-deftest :deftest-hcs ...)` factory + 7 layered deftests. |
| 3 | Layer count + naming | ✅ 7 layers shipped: L0 spawn-and-drop, L1 helper-get-empty, L2 helper-put-one, L3 helper-put-then-get, L4 helper-get-many-keys, L5 eviction (cap=2), L6 multi-client (count=2). All `:test::hcs-*` named. |
| 4 | Per-helper deftest discipline | ✅ 7 layer-helpers with their own deftests; 5 sub-helpers (3a `slot-presence`, 3b `put-then-get-on-handle`, 4a `probe-three-on-handle`, 5a `eviction-on-handle`, 6a `client-put-get`) under SKILL's Level-3-taste exemption (each invoked in exactly one place). |
| 5 | Top-down dependency graph | ✅ No forward references. Sub-helpers defined before parent layers per `make-deftest` factory's prelude ordering. |
| 6 | Body line budget | ✅ Helper outer let*: 3-7 bindings (verified via spot-check). Deftest body: 3-7 lines per `(:deftest-hcs ...)` invocation. |
| 7 | Helper-verb usage | ✅ Layer 1+ uses `:wat::holon::lru::HologramCacheService::get` and `:wat::holon::lru::HologramCacheService::put` (post-sweep-2a Handle-taking signatures). NO raw kernel send/recv. |
| 8 | All 9 LRU `:should-panic` annotations retired | ✅ Verified via grep: `:should-panic` matches in workspace are exactly: (a) wat-sqlite arc-122 mechanism self-test (line 25), (b) wat-tests/tmp-totally-bogus.wat scratch file (line 16). NO LRU-related `:should-panic` annotations remain. The 8 in main test file + 1 in step-B-single-put are GONE. |
| 9 | Proof step-A updated | ✅ `step-A-spawn-shutdown.wat` 46 → 50 LOC. `ReqTxPool` → `HandlePool<Handle>`, `_req-tx :ReqTx` → `_handle :Handle`. Same lifecycle shape; pop one Handle, finish, drop, join. Test passes. Educational shape preserved. |
| 10 | Proof step-B updated + :should-panic retired | ✅ `step-B-single-put.wat` 68 → 58 LOC. `:should-panic("channel-pair-deadlock")` annotation REMOVED (only a comment line remains explaining why). Replaced ack-channel allocation + 4-arg `/put` call with single Handle-taking `/put` call. Test passes naturally. |
| 11 | **Workspace at 0 failed** | ✅ `cargo test --release --workspace` shows 0 failed across ALL crates (every test result line shows `... ok`). The wat-holon-lru crate: 19 passed / 0 failed (was 10 passed / 16 failed pre-sweep-2b). |
| 12 | Honest report | ✅ Sonnet's report covers all required sections, including LOC overage justification + Level-3-taste sub-helper enumeration + HolonAST-vs-i64 assertion strategy adaptation. |

**Hard verdict:** 12/12 clean. Rows 8 + 11 are the load-bearing
rows (LRU panic crutches retired AND workspace ships clean). Both
held.

## Soft scorecard (3/4 PASS — row 13 mixed)

| # | Criterion | Result |
|---|---|---|
| 13 | LOC budget | ⚠️ MIXED. Main file 501 LOC vs 200-450 band; +51 (+11%). Brief's threshold: ">500 = re-evaluate". Sonnet honestly named the +2 optional layers (5 eviction + 6 multi-client) — explicitly allowed by BRIEF Section "Suggested layer plan" — as the driver. Plus HolonAST type names being longer than String/i64 (`:wat::holon::HolonAST` vs `:wat::core::String`). Within tolerance given the explicit-optional-layers permission. |
| 14 | Pattern fidelity to wat-lru rebuild | ✅ Factory prelude pattern (`:deftest-hcs`) mirrors sweep 1's `:deftest-lru`. Tuple-out pattern applied at L1, L3, L4, L5, L6 per arc 117/126's scope-deadlock check. Sub-helper extraction mirrors sweep 1's Level-3-taste pattern. |
| 15 | clippy clean | ✅ wat-source-only edits; no Rust delta. |
| 16 | No-grinding discipline | ✅ Sonnet did NOT modify substrate. Did NOT add `:ignore` as `:should-panic` substitute. STOP-at-first-red held (every layer cargo-tested before next). |

## What this slice closes

- **HolonLRU `:should-panic` crutches retire.** All 9 LRU
  channel-pair-deadlock annotations (8 in main + 1 in arc-119
  proof) are GONE. The substrate's pair-by-index discipline
  removed the deadlock pattern; tests pass naturally.
- **Arc 130 slice 1 (wat-lru) + slice 2 (HolonLRU) substrate
  work effectively complete.** Both LRU services use post-arc-130
  pair-by-index discipline; both have complectens-rebuilt test
  files; workspace ships clean across both crates.
- **The substrate-as-teacher cascade closes another link.** Arc
  130's premise ("when pair-by-index propagates, deadlock-pattern
  tests retire") proves out across both crates.
- **Workspace milestone holds**: 0 failed tests, no `:ignore`
  crutches, no `:should-panic` LRU crutches. First time both LRU
  services have shipped clean test files post-arc-130.

## What this slice does NOT close (forward progress only)

- Arc 130 slice 3 INSCRIPTION (orchestrator paperwork — likely
  next sweep)
- Arc 130 v1 closure trajectory (after slice 3)
- Arc 109 K.holon-lru slice (#195) — becomes tractable
  post-arc-130; separate arc work
- Arc 109 v1 closure (#229) — major chain link closes when
  arc 130 closes

## Calibration record

- **Predicted Mode A (~65%)**: ACTUAL Mode A clean.
- **Predicted runtime (60-min upper)**: ACTUAL ~7 min — UNDER by
  ~88%. The substrate reshape from sweep 2a + the wat-lru
  template made this mechanical pattern-application.
- **Time-box (90 min)**: NOT triggered (used 7.7%).
- **Predicted LOC (200-450)**: ACTUAL 501 (+51, +11%). Sonnet
  honestly named the optional-layers driver.
- **Honest deltas**: 4 (Layers 5-6 included; sub-helpers under
  Level-3-taste; HolonAST presence-pattern assertion strategy;
  LOC budget). All acceptable per BRIEF guidance.

## Two adaptations re-applied from sweep 1

The wat-lru rebuild's two patterns propagated cleanly to HolonLRU:

1. **Factory prelude** — `(:wat::test::make-deftest :deftest-hcs
   ...)` with helpers in the prelude (sandbox-scope leak workaround
   identified during sweep 1).
2. **Tuple-out for scope-deadlock** — `(driver, value)` returned
   from inner let* so spawn/pool/handle drop before outer
   Thread/join-result (arc 117/126 check workaround).

Both patterns survived crate boundaries — the discipline propagates
without re-discovery.

## Mutual-agreement protocol verdict

User direction 2026-05-06: "lets get holon-lru cleaned up."

The chain held end-to-end for both sweeps:
- **User → Orchestrator**: cleanup direction
- **Orchestrator → Sonnet 2a**: substrate reshape brief; Mode A clean
- **Orchestrator → Sonnet 2b**: test rebuild brief; Mode A clean
- **Sonnet → Reality**: substrate + 3 test files + 9 :should-panic
  retired + workspace 0-failed

The four-questions check from earlier (Option A sequential beat
Option B bundled on Obvious/Simple/Honest) is also vindicated:
two clean sweeps (~12 min total wall-clock) produced clean
diagnostic separation and atomic commit hygiene.

**Arc 130 slice 1 + slice 2 substrate work ships clean.** Slice 3
(closure paperwork) is the next forward step.
