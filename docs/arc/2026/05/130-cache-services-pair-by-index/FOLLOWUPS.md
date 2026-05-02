# Arc 130 — Followups: complectēns sweep queue

The *complectēns* spell's first cast across the codebase (2026-05-03)
found 22 deftests in 9 files with body line-counts above the
empirical "suspect" threshold (>30 lines). These are the test
files queued for compositional rewrite — bottom-up proof tree +
top-down dependency graph + named-helper layers per the
discipline.

Each entry is a phase-1 candidate. Phase-2 judgment (Level 1 lie
vs Level 2 mumble vs Level 3 taste) happens when the rewrite
arc lands; some of these may turn out to be inherently complex
deftests that don't fully shrink, and that's fine — the spell's
own SKILL says line-count is a candidate flag, not a verdict.

The codebase stays pristine via "we observe subpar; we file it;
we work it down arc-by-arc." This doc is the queue.

## Status legend

| Symbol | Meaning |
|---|---|
| 🔴 | Level 1 definite (>100 lines) — definite monolithic violation |
| 🟠 | Level 1 likely (>50 lines) — almost certainly violates |
| 🟡 | Suspect (>30 lines) — worth phase-2 judgment |
| ✓ | Already shipped via complectens sweep |

## Already shipped

- ✓ `crates/wat-lru/wat-tests/lru/CacheService.wat` — the worked demonstration. 5 layered deftests, final body 6 lines. Commit `98fa7c9`.
- ✓ `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` — sonnet calibration sweep, 2026-05-03. 14 → 22 deftests; body shrink 75% average (e.g. step3 75→4 lines). Two-prelude pattern surfaced as a document gap. See `CALIBRATION-HOLOGRAM-SCORE.md`.

## Queue (sweep candidates)

### ✓ `wat-tests/service-template.wat` — shipped 2026-05-03 (arc 135 slice 1)

- ✓ L230 `:svc::test-template-end-to-end` body 106→1. 4 layered helpers in the prelude.

### ✓ `wat-tests/console.wat` — shipped 2026-05-03 (arc 135 slice 1)

- ✓ L92 `:wat-tests::std::service::Console::test-multi-writer` body 101→81 (visual; OUTER logical bindings 8→5). Hermetic-program tests have inherently irreducible inner-program bodies; outer scaffolding factored into helpers per phase-2 judgment.
- ✓ L35 `:wat-tests::std::service::Console::test-hello-world` body 42→37 (visual; outer logical similar). Same hermetic-program constraint.

Both files use a single `make-deftest` prelude (no mixed outcomes; all clean pass). 5 outer-scaffolding helpers added to console.wat; 4 lifecycle/scenario helpers to service-template. SCORE-SLICE-1.md surfaced three new SKILL deltas: Thread/output drain on non-unit O; arc 126 fires at call sites passing both halves; hermetic-program inner bodies are irreducible.

### `crates/wat-telemetry/wat-tests/telemetry/Console.wat`

- 🟠 L8 `:wat-telemetry::Console::test-dispatcher-edn` body=80
- 🟠 L92 `:wat-telemetry::Console::test-dispatcher-json` body=65

Both test the same dispatcher with different output formats — helpers should largely overlap.

### `crates/wat-telemetry/wat-tests/telemetry/Service.wat`

- 🟠 L71 `:wat-telemetry::test-batch-roundtrip` body=59
- 🟠 L134 `:wat-telemetry::test-cadence-fires` body=58
- 🟡 L22 `:wat-telemetry::test-spawn-drop-join` body=42

Three deftests in one file; lifecycle + batch + cadence each get their own layer.

### `crates/wat-telemetry/wat-tests/telemetry/WorkUnit.wat`

- 🟠 L593 `:wat-telemetry::WorkUnit::test-make-scope-ships-counter` body=62
- 🟠 L467 `:wat-telemetry::WorkUnit::test-collect-metrics-two-duration-samples` body=57
- 🟠 L273 `:wat-telemetry::WorkUnit::test-build-counter-metric` body=56
- 🟠 L336 `:wat-telemetry::WorkUnit::test-build-duration-metric` body=55
- 🟡 L400 `:wat-telemetry::WorkUnit::test-collect-metrics-empty` body=47

5 mid-size deftests; likely strong helper-reuse opportunity (build + collect + emit).

### `crates/wat-telemetry/wat-tests/telemetry/WorkUnitLog.wat`

- 🟠 L142 `:wat-telemetry::WorkUnitLog::test-each-level-emits-log` body=89
- 🟠 L69 `:wat-telemetry::WorkUnitLog::test-info-emits-log-event` body=64

Two log-emission tests; likely shareable spawn + emit + drain helpers.

### `crates/wat-holon-lru/wat-tests/holon/lru/HologramCache.wat`

- 🟡 L74 `:wat-tests::holon::HologramCache::test-lru-evicts-from-hologram` body=35
- 🟡 L115 `:wat-tests::holon::HologramCache::test-get-bumps-lru` body=35

Both at the suspect threshold; phase-2 judgment may exempt them as inherently complex match expressions on cache state.

### `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat`

- 🟡 L25 `:step_B_single_put` body=43

Arc 119 stepping-stone proof. Already in stepping-stone shape; the body is the proof's content. Phase-2 may exempt.

### `wat-tests/stream.wat`

- 🟡 L81 `:wat-tests::std::stream::test-with-state-dedupe-adjacent` body=31

Stream pipeline test; likely simple to factor.

### `wat-tests/test.wat`

- 🟡 L191 `:wat-tests::std::test::test-assert-stderr-matches-fail-reports-pattern` body=42
- 🟡 L114 `:wat-tests::std::test::test-assert-coincident-fail-renders-explanation` body=36
- 🟡 L69 `:wat-tests::std::test::test-assert-contains-fail-populates-actual` body=31

Three meta-tests of the test framework's assertion failure paths. The "construct a deliberate failure → check it surfaces correctly" shape may resist clean decomposition; phase-2 judgment likely.

## How to work this queue

Each file becomes its own sweep arc (or a small slice within an arc that touches its substrate). The brief shape mirrors `CALIBRATION-HOLOGRAM-BRIEF.md`:

1. Read in order: complectēns SKILL → arc 130 REALIZATIONS → complected/README → wat-lru CacheService.wat (the worked demo).
2. Target ONE file from this queue.
3. Substrate OFF-LIMITS unless the rewrite genuinely needs it.
4. Outcomes preserved (any `:should-panic` annotations stay).
5. Per-helper deftests for every new helper added.
6. NO commits.
7. Score against the four questions; mark this entry ✓ in this file when shipped.

The queue is processed by priority + co-location:

- 🔴 (definite) first.
- Then 🟠 (likely) by proximity (Console.wat + Console.wat — shared helpers; WorkUnit.wat all 5 together).
- Then 🟡 (suspect) only after phase-2 judgment confirms violation.

## When to re-cast

When the queue shrinks AND new tests have been added (any commit touching `wat-tests/` or `crates/*/wat-tests/`), re-cast complectēns. The phase-1 scan is cheap; let it run. New violations get appended here; existing entries get marked ✓ as they ship.

## Cross-references

- `.claude/skills/complectens/SKILL.md` — the spell.
- `REALIZATIONS.md` — the discipline.
- `complected-2026-05-02/` — what bad looks like.
- `crates/wat-lru/wat-tests/lru/CacheService.wat` — what good looks like.
