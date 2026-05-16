# Arc 170 Slice 4a-γ-audit AUDIT — deftest body three-rule classification

**Task:** #317
**BRIEF:** BRIEF-SLICE-4A-GAMMA-AUDIT-DEFTEST-BODIES.md
**Substrate rule reference:** docs/COMPACTION-AMNESIA-RECOVERY.md § FM 7-ter

## Summary counts

| Total deftest sites | Safe for thread | Rule R1 (stdio reads) | Rule R2 (stdio verbs) | Rule R3 (set-! family) | Multiple rules | Total flagged |
|---|---|---|---|---|---|---|
| 261 (deftest-bound) + 5 (already-hermetic via `:deftest-ambient`) = 266 | 256 | 5 (all multi-rule with R2) | 5 (all multi-rule with R1) | 0 | 5 (R1+R2) | **5** |

**Flagged: 5 sites, all in `wat-tests/test.wat`** — the test-the-harness file. Every flagged site is a deftest that asserts on captured stdio from an inner `run-hermetic` call AND lexically contains a `:wat::kernel::println` / `eprintln` inside that inner call.

The 5 `:deftest-ambient` sites in `ambient-stdio.wat` are ALREADY hermetic (alias registered via `make-deftest-hermetic` on line 40). No decoration needed; counted separately under "deftest-hermetic sanity check" below.

## Population delta from BRIEF prediction

The BRIEF estimated ~224 active `:wat::test::deftest` sites; that count came from raw `grep -rEn ":wat::test::deftest\b"` which included 39 false-positive matches in `.rs` source comments / fixtures (`tests/wat_make_deftest.rs` + `crates/wat-macros/src/discover.rs`) and excluded the 90 alias call sites that go through `:wat::test::make-deftest`.

Actual audit population (per discovery semantics — what `wat::test!{}` would register):

| Source | Count |
|---|---|
| Direct `(:wat::test::deftest ...)` in `.wat` files | 176 |
| `(:deftest ...)` alias call sites (`make-deftest :deftest`) | 71 |
| `(:deftest-hcs ...)` alias call sites (`make-deftest :deftest-hcs`) | 7 |
| `(:deftest-lru ...)` alias call sites (`make-deftest :deftest-lru`) | 5 |
| `(:wat-tests::std::test::cfg-deftest ...)` alias call sites (`make-deftest :wat-tests::std::test::cfg-deftest`) | 2 |
| **Subtotal — expand to plain `deftest` (impacted by flip)** | **261** |
| `(:deftest-ambient ...)` alias call sites (`make-deftest-hermetic :deftest-ambient`) — ALREADY hermetic | 5 |
| **Grand total deftest population** | **266** |

## Flagged sites (must become deftest-hermetic after flip)

5 sites, all in `wat-tests/test.wat`. Each body lexically contains both a hermetic-spawned `:wat::kernel::println` or `:wat::kernel::eprintln` (R2) and a `:wat::test::assert-stdout-*` / `assert-stderr-*` consuming the resulting `RunResult` (R1). All multi-rule (R1 + R2).

| File:line | Test name | Rules fired | Rationale |
|---|---|---|---|
| `wat-tests/test.wat:132` | `:wat-tests::std::test::test-assert-stdout-is-matches` | R1 + R2 | body wraps `(:wat::kernel::println ...)` in `run-hermetic` (lines 138-139) and reads result via `assert-stdout-is` (line 142). |
| `wat-tests/test.wat:146` | `:wat-tests::std::test::test-assert-stderr-matches-pass` | R1 + R2 | body wraps `(:wat::kernel::eprintln ...)` in `run-hermetic` (line 151) and reads result via `assert-stderr-matches` (line 152). |
| `wat-tests/test.wat:154` | `:wat-tests::std::test::test-assert-stderr-matches-fail-reports-pattern` | R1 | body lexically contains `(:wat::test::assert-stderr-matches silent "my-pattern")` (line 167) inside an inner `run-thread`; outer reads `RunResult/failure` of the failed inner — R1 fires on lexical assert-stderr-matches presence. |
| `wat-tests/test.wat:188` | `:wat-tests::std::test::test-run-string-entry-path` | R1 + R2 | body wraps `(:wat::kernel::println "from-string")` in `run-hermetic` (line 201) and reads result via `assert-stdout-is` (line 203). |
| `wat-tests/test.wat:207` | `:wat-tests::std::test::test-run-ast-via-program` | R1 + R2 | body wraps `(:wat::kernel::println "from-ast")` in `run-hermetic` (line 212) and reads result via `assert-stdout-is` (line 214). |

## Safe sites (stay as plain deftest after flip)

256 sites. Enumerated by file (each file's deftests are uniformly safe — no rules fire in any body). Counts use `^\s*\(:wat::test::deftest\b` (or alias form) — line-start matches with comment lines excluded.

| File | Safe count | Notes |
|---|---|---|
| `wat-tests/holon/Trigram.wat` | 2 | direct; assert-coincident pattern only |
| `wat-tests/holon/Filter.wat` | 7 | direct; assert-coincident / assert-eq |
| `wat-tests/holon/Hologram.wat` | 18 | direct; assert-eq / assert-coincident / Hologram ops (R3 patterns appear ONLY in comments lines 44, 46 — not live) |
| `wat-tests/holon/coincident.wat` | 5 | direct; assert-coincident |
| `wat-tests/holon/Circular.wat` | 2 | direct; assert-eq / assert-coincident |
| `wat-tests/holon/Subtract.wat` | 2 | direct; assert-coincident |
| `wat-tests/holon/Reject.wat` | 2 | direct; assert-coincident |
| `wat-tests/holon/Sequential.wat` | 2 | direct; assert-coincident |
| `wat-tests/holon/eval-coincident.wat` | 10 | direct; assert-coincident / eval-ast! |
| `wat-tests/holon/term.wat` | 12 | direct; assert-coincident / term ops |
| `wat-tests/holon/ReciprocalLog.wat` | 4 | `:deftest` alias; assert-coincident |
| `wat-tests/core/result-expect.wat` | 3 | direct; assert-eq |
| `wat-tests/core/struct-to-form.wat` | 2 | direct; assert-eq / eval-ast! |
| `wat-tests/core/option-expect.wat` | 4 | direct; assert-eq |
| `wat-tests/test.wat` | 11 direct + 2 `:cfg-deftest` alias = 13 | direct `assert-eq` / `assert-contains` / `macroexpand` / `Failure/...` reads on inner run-thread (Failure slot is NOT a stdio slot — R1 does not fire). 5 direct sites flagged separately above (16 direct − 5 = 11 safe direct); 2 `:cfg-deftest` call sites also safe. |
| `wat-tests/stream.wat` | 13 | direct; stream chunks/windows; assert-eq |
| `wat-tests/time.wat` | 37 | direct; time/instant/duration; assert-eq |
| `wat-tests/edn/render.wat` | 10 | direct; edn writes; assert-eq |
| `wat-tests/edn/roundtrip.wat` | 7 | `:deftest` alias; assert-eq roundtrip |
| `wat-tests/run-thread.wat` | 2 | direct; run-thread ok/err; reads `RunResult/failure` (safe) |
| `wat-tests/service-template.wat` | 5 | `:deftest` alias; service spawn/join/recv (no R1/R2/R3) |
| `wat-tests/tmp-3tuple-probe.wat` | 1 | direct; assert-eq |
| `wat-tests/tmp-totally-bogus.wat` | 1 | direct; tmp probe |
| `wat-tests/tmp-baseline-nongeneric.wat` | 1 | direct; tmp probe |
| `wat-tests/tmp-3tuple-inferred.wat` | 1 | direct; assert-eq |
| `crates/wat-sqlite/wat-tests/arc-123-time-limit.wat` | 3 | direct; assert-eq |
| `crates/wat-sqlite/wat-tests/arc-122-attributes.wat` | 3 | direct; assert-eq |
| `crates/wat-sqlite/wat-tests/sqlite/Db.wat` | 5 | direct; sqlite ops |
| `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-A-spawn-shutdown.wat` | 1 | direct; spawn/shutdown |
| `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat` | 1 | direct; single-put |
| `crates/wat-holon-lru/wat-tests/holon/lru/HologramCache.wat` | 10 | `:deftest` alias; cache ops |
| `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` | 7 | `:deftest-hcs` alias; cache service ops |
| `crates/wat-telemetry/wat-tests/telemetry/uuid.wat` | 2 | direct; uuid ops |
| `crates/wat-telemetry/wat-tests/telemetry/Service.wat` | 9 | `:deftest` alias; telemetry service |
| `crates/wat-telemetry/wat-tests/telemetry/WorkUnit.wat` | 22 | `:deftest` alias; work-unit ops |
| `crates/wat-telemetry/wat-tests/telemetry/WorkUnitLog.wat` | 3 | `:deftest` alias; work-unit-log |
| `crates/wat-telemetry-sqlite/wat-tests/telemetry/auto-spawn.wat` | 1 | `:deftest` alias; auto-spawn |
| `crates/wat-telemetry-sqlite/wat-tests/telemetry/edn-newtypes.wat` | 1 | `:deftest` alias; edn newtypes |
| `crates/wat-telemetry-sqlite/wat-tests/telemetry/reader.wat` | 6 | `:deftest` alias; reader |
| `crates/wat-telemetry-sqlite/wat-tests/telemetry/hashmap-field.wat` | 1 | `:deftest` alias; hashmap-field |
| `crates/wat-telemetry-sqlite/wat-tests/telemetry/Sqlite.wat` | 2 | `:deftest` alias; Sqlite |
| `crates/wat-lru/wat-tests/lru/LocalCache.wat` | 4 | direct; cache ops |
| `crates/wat-lru/wat-tests/lru/HolonKey.wat` | 3 | direct; HolonKey ops |
| `crates/wat-lru/wat-tests/lru/CacheService.wat` | 5 | `:deftest-lru` alias; cache service |
| `examples/with-loader/wat-tests/test-loader.wat` | 1 | direct; loader wiring (no rules) |
| **Total safe** | **256** | (= 261 deftest-bound minus 5 flagged in `wat-tests/test.wat`) |

Per-file row sum verification: 2+7+18+5+2+2+2+2+10+12+4+3+2+4+13+13+37+10+7+2+5+1+1+1+1+3+3+5+1+1+10+7+2+9+22+3+1+1+6+1+2+4+3+5+1 = **256** ✓.

Coverage verification (mechanical greps):
- Direct `:wat::test::deftest` sites in `.wat` files (line-start, excluding comments): `grep -rEcn "^\s*\(:wat::test::deftest\b" wat-tests/ crates/ examples/ | grep -v ":0$" | grep -v "\.md:" | grep -v "\.rs:" | awk -F: '{s+=$NF} END {print s}'` → **176**
- `:deftest` alias call sites: 71. `:deftest-hcs`: 7. `:deftest-lru`: 5. `:cfg-deftest`: 2. → subtotal **85** alias calls expanding to deftest.
- Total: 176 + 85 = **261** deftest-bound. Of those: 5 flagged + 256 safe = 261. ✓
- Plus 5 `:deftest-ambient` (already hermetic via `make-deftest-hermetic`) = **266** grand total.

## deftest-hermetic sanity check

Per the BRIEF, the audit also classifies `:wat::test::deftest-hermetic` sites — they don't need decoration; the audit confirms they're legitimately hermetic (at least one rule fires).

Discovery surfaces ZERO call sites of `(:wat::test::deftest-hermetic ...)` directly in `.wat` files. The only `deftest-hermetic` occurrences in the codebase are:

| File:line | Form | Notes |
|---|---|---|
| `tests/probe_deftest_hermetic_isolation.rs:87,136,140,188,249` | embedded wat strings in Rust probe test | NOT discovery-visible; lives inside Rust `#[test]` fn calling `startup_from_source`. Out of audit population. |
| `crates/wat-macros/src/discover.rs:5,11,233,236,343,369,942,960,975,991,1021,1031,1048,1051,1070` | doc-comment fragments + scan_file unit-test fixtures | NOT discovery-visible; Rust source fixtures. Out of audit population. |
| `wat-tests/kernel/services/ambient-stdio.wat:40` | `(:wat::test::make-deftest-hermetic :deftest-ambient ...)` | Registers `:deftest-ambient` alias which expands to `deftest-hermetic`. The 5 call sites below ARE deftest-hermetic invocations. |

The 5 indirect deftest-hermetic sites via `:deftest-ambient`:

| File:line | Test name | Rules fired | Notes |
|---|---|---|---|
| `wat-tests/kernel/services/ambient-stdio.wat:129` | `:wat-rs::test::test-ambient-stdio-println-string` | R1 (assert-stdout-is) + R2 (helper `:test::run-println-string` lexically wraps `:wat::kernel::println` line 51) | legitimate hermetic (ambient-stdio capture) |
| `wat-tests/kernel/services/ambient-stdio.wat:141` | `:wat-rs::test::test-ambient-stdio-println-i64` | R1 + R2 (helper line 62) | legitimate hermetic |
| `wat-tests/kernel/services/ambient-stdio.wat:155` | `:wat-rs::test::test-ambient-stdio-eprintln-string` | R1 (assert-stderr-matches) + R2 (helper `:test::run-eprintln-string` line 72) | legitimate hermetic |
| `wat-tests/kernel/services/ambient-stdio.wat:167` | `:wat-rs::test::test-ambient-stdio-println-twice` | R1 + R2 (helper lines 83-84) | legitimate hermetic |
| `wat-tests/kernel/services/ambient-stdio.wat:184` | `:wat-rs::test::test-ambient-stdio-readln-echo` | R1 + R2 (helper lines 115-116; readln + println) | legitimate hermetic; this is the only `:wat::kernel::readln` call site in any test body, lexically inside the helper |

All 5 fire at least one rule (R1 from the assert-stdout/stderr in the deftest body; R2 from the helper bodies that the deftest body invokes through `:test::run-*`). Zero over-hermetic candidates.

## Honest deltas

### Delta 1 — Population is 266, not the BRIEF's ~224 estimate

The BRIEF's grep "estimated total: ~224" was raw `grep -rEn ":wat::test::deftest\b"` including:
- 39 false-positive matches in `.rs` files (`tests/wat_make_deftest.rs` + `crates/wat-macros/src/discover.rs`) — doc comments + Rust `scan_file` unit-test fixtures (NOT live deftests).
- Excluded the 90 alias-call-site population — `:deftest`, `:deftest-hcs`, `:deftest-lru`, `:cfg-deftest`, `:deftest-ambient` aliases registered via `make-deftest` / `make-deftest-hermetic` are NOT lexically `:wat::test::deftest` but ARE deftest invocations after macro expansion.

Net: audit population is **266** (261 deftest-bound + 5 hermetic-bound) — about 17% more than the BRIEF estimate, dominated by alias-call sites that are invisible to a naive `:wat::test::deftest` grep. **No impact on the flagged-set:** all 5 flagged sites are direct `(:wat::test::deftest ...)` invocations in `wat-tests/test.wat`.

### Delta 2 — All flagged sites are in ONE file (`wat-tests/test.wat`)

The entire flagged set (5/261 = 1.9%) lives in one file: the test-the-harness file `wat-tests/test.wat`. Every other file in the codebase fires zero rules. This is below the BRIEF's predicted range (25-55 flagged sites at 10-25%); the actual hermetic-required surface is much narrower than predicted.

Driver of the gap: predicted R3 count was 10-20 (capacity / router / redef tests). The actual codebase has **zero** `:wat::config::set-*!` calls in any test body — all live in non-test files (substrate / examples). The BRIEF's mention of "wat_bundle_capacity" + arc 157 redef tests refers to tests that were already migrated or never used `set-*!` from a test body. The 4a-β breadcrumb (FM 7-ter) confirms: 1 site had `(:wat::config::set-capacity-mode! :error)` and it was stripped during the 4a-β sweep. No remaining R3 fires.

### Delta 3 — Lexical-vs-semantic R2 in test.wat (4 of 5 flagged)

Of the 5 flagged sites, 4 (lines 132, 146, 188, 207) lexically contain `:wat::kernel::println` or `:wat::kernel::eprintln`, but ONLY inside an inner `(:wat::test::run-hermetic ...)` form. Semantically, those stdio verbs run in the forked child's runtime, NOT in the outer deftest body's runtime. The outer body could safely run in a thread (post-flip) because:
- The inner `run-hermetic` call forks a child process; the child has its own fd 0/1/2.
- The outer thread just CALLS `run-hermetic` (a substrate verb usable from any context) and reads the returned `RunResult.stdout/.stderr` fields (struct field access; no runtime requirement).

The lexical R2 rule is conservative — flagging more often than semantically necessary — but the BRIEF's method specifies "Look for: `:wat::kernel::println` ..." which is lexical. Strict lexical application keeps the rule mechanical for the decorate slice.

**Crucial R1 caveat:** R1 fires not from the `:wat::kernel::println` lexically, but from the `assert-stdout-is` / `assert-stderr-matches` call — which CONSUMES the stdout slot of the inner run-hermetic's RunResult. Even though the consumption is just struct field access, the assertion only makes sense when the slot is populated, which requires the inner runner to be `run-hermetic` (not `run-thread`). The 4a-β breadcrumb's "5 sites went red on assert-stdout-is" rediscovered this exact pattern.

**Decorate-slice impact:** all 5 must become `deftest-hermetic` for the post-flip world to remain green. The lexical-vs-semantic nuance does not change the decoration decision.

### Delta 4 — Site 154 fires only R1, not R2 (one-rule subset)

The flagged site at `test.wat:154` (test-assert-stderr-matches-fail-reports-pattern) is the only flagged site that does NOT lexically contain `:wat::kernel::*`. Its R1 firing comes from `(:wat::test::assert-stderr-matches silent "my-pattern")` where `silent` is the result of an inner `(:wat::test::run-thread ())` (NOT run-hermetic — thread). The test EXPECTS the assertion to fail (silent stderr is empty; "my-pattern" can't match), populating the OUTER deftest's `RunResult/failure` with `expected = "my-pattern"`. The outer then reads `RunResult/failure` (safe) to verify the message.

This is a semantically subtle case: the inner `run-thread` returns empty stdio (correct thread behavior), the assertion fails on empty stderr (correct), the outer captures the failure shape via the failure slot. The OUTER body itself only needs hermetic IF the lexical `assert-stderr-matches` presence is treated as R1 — which it is per BRIEF rule.

If the rule were narrowed to "reads `RunResult/stdout/stderr` slots OR calls a NON-thread-aware stdio assertion," site 154 could stay as plain deftest after the flip. The current lexical rule flags it.

### Delta 5 — Reads of `RunResult/failure` slot are NOT R1

Per the EXPECTATIONS edge case ("OUTER deftest reads only the run-thread's `RunResult.failure` slot (proven safe by 4a-β)"), I confirmed:
- `wat-tests/test.wat` lines 40, 58, 62, 89, 100 — all read `Failure/message`, `Failure/actual`, `Failure/expected` (sub-fields of the `failure` slot's `Some f` variant). NONE read `stdout` / `stderr` slots. **Safe for thread.**
- `wat-tests/run-thread.wat` lines 26, 41 — read `RunResult/failure`. Safe.

R1's "Reads `RunResult.stdout` / `RunResult.stderr` slots" is strict: failure slot is structurally different and does not require process-pipe machinery to be populated.

### Delta 6 — Helper-function indirection ONLY appears in ambient-stdio.wat (already hermetic)

The 5 `:deftest-ambient` call sites bottom out on helpers (`:test::run-println-string` etc.) defined in the make-deftest-hermetic PRELUDE. The deftest BODY (lines 129, 141, 155, 167, 184) calls those helpers — lexically the body does not contain `:wat::kernel::*`. But because the file uses `make-deftest-hermetic`, the indirection is already inside a forked child anyway; the helper indirection doesn't affect classification (already hermetic). Surfaced for completeness; no audit-impact.

### Delta 7 — Configured-deftest variants land at audit population (not invisible)

The BRIEF's EXPECTATIONS note about `make-deftest` variants ("Note configured-variant call sites in the audit if they expand to deftest+three-rule-relevant code") was load-bearing — these alias call sites are 90/266 = 34% of the population and would have been missed by a naive `:wat::test::deftest` grep. The decorate slice (#318) needs to handle aliases carefully:

- `:deftest` alias call sites (71) — to flip a SINGLE alias-using-file from thread to hermetic, change `(:wat::test::make-deftest :deftest ...)` → `(:wat::test::make-deftest-hermetic :deftest ...)`. ALL call sites in that file flip atomically. But: no file's `:deftest` sites are currently flagged, so no `:deftest` make-deftest registration needs flipping.
- `:cfg-deftest`, `:deftest-hcs`, `:deftest-lru`, `:deftest-ambient` — same per-alias atomicity. None are flagged (`:deftest-ambient` is already hermetic).

Net: **the decorate slice only needs to rename 5 direct `:wat::test::deftest` → `:wat::test::deftest-hermetic` in `wat-tests/test.wat`. Zero alias-registration changes required.**

### Delta 8 — Most surprising finding

**The entire `wat-tests/` + `crates/*/wat-tests/` + `examples/wat-tests/` deftest population — 266 sites across 45 files — concentrates the hermetic-required surface into ONE file (`test.wat`) and 5 deftests within it.** Every other test in the codebase already uses thread-compatible patterns (assert-eq, assert-coincident, Failure-slot reads, service spawn/recv/join). The 4a-γ-flip is therefore extremely low-risk: 256/261 = 98.1% of deftest-bound sites are already safe for thread.

The harness-self-tests in `test.wat` are exactly the tests that NEED to exercise the hermetic path (they verify `assert-stdout-is` / `assert-stderr-matches` themselves, which requires the captured-stdio mechanism that only `run-hermetic` provides). They are correctly the natural residue of "tests that exercise stdio capture infrastructure" — exactly the EXPECTATIONS' "Tests-of-tests" prediction landing on target.

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 30–60 min | ~25 min |
| Total deftest sites audited | ~224 | 266 (176 direct + 85 alias-to-deftest + 5 alias-to-hermetic) |
| Total flagged for decoration | 25–55 (~10–25%) | **5** (~1.9%) |
| R1 (stdio reads) | 5–15 | 5 (all multi-rule with R2; site 154 is R1-only) |
| R2 (stdio verbs) | 5–10 | 4 multi-rule (lines 132, 146, 188, 207); 0 R2-only |
| R3 (set-! family) | 10–20 | **0** |
| Multi-rule | 5–10 | 4 (R1+R2) |
| deftest-hermetic over-hermetic candidates | 0–5 | 0 |
| Helper-obscured cases | 0–10 | 5 (all in ambient-stdio.wat, all already hermetic — no audit impact) |
| Mode | A (clean) | A (clean — no defects discovered; pure information artifact) |

Predictions ran HIGH on flagged count and R3. The actual surface is much narrower:
- R3 prediction (10-20) was based on capacity / router / redef tests. 4a-β already stripped the only such site; current codebase has **zero** R3 fires in test bodies.
- Multi-rule prediction (5-10) landed at 4.
- Total flagged is 5/261 = 1.9% of deftest-bound, well below the predicted 10-25% band — but consistent with the EXPECTATIONS' "tests-of-tests" intuition that the hermetic-required surface concentrates in `test.wat`.
