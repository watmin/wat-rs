# Arc 119 — Vocare Audit (2026-05-01)

**Status:** trust-restoration evidence. Full wat-rs codebase
surveyed against `/vocare`. The 5 known-failing wat-tests are
the only Tier 1+2 violations. No broader drift.

## Provenance

User direction (2026-05-01) after the discipline correction
landed:

> use vocare - find /all/ our failures - the entire code base -
> i don't trust the ground

Sonnet agent surveyed all test files in wat-rs against the vocare
methodology (`.claude/skills/vocare/SKILL.md`) and the
caller-perspective principle (`docs/CONVENTIONS.md` § "Caller-
perspective verification"). Survey only — no edits.

## Scope surveyed

- `wat-tests/` root: 20 files
- `crates/*/wat-tests/`: 16 files
- `crates/*/tests/*.rs`: 8 files (Rust integration test harnesses)
- `tests/*.rs`: 57 files (top-level Rust integration tests)
- `examples/*/wat/`: 5 files (example programs)
- `examples/*/wat-tests/`: 2 files

**Out of scope** per vocare's "What vocare does NOT flag":
- `wat-rs/wat-tests/service-template.wat` (its caller IS the
  service implementer)
- Rust unit tests inside `src/**/*.rs #[cfg(test)]` blocks
- `holon-lab-trading/` (separate workspace)

## Findings

### Tier 1 — Wrong vantage (raw protocol where helper exists)

**4 tests, all in
`crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`:**

- `test-step3-put-only` — hand-builds `Request::Put k v` enum
  variants and calls `:wat::kernel::send` directly. Reaches into
  `HologramCacheService/loop` in the worker helper, bypassing
  `HologramCacheService/run`. The `/loop` is driver internals; a
  consumer never calls it.
- `test-step4-put-get-roundtrip` — manually constructs Request
  variants for both Put and Get; manually unwraps the
  `Result<Option<Option<HolonAST>>, _>` chain by hand. Both
  helper verbs (`HologramCacheService/get`, `/put`) exist and
  hide all of this.
- `test-step5-multi-client-via-constructor` — uses
  `HologramCacheService/spawn` correctly (right entry point) but
  then drops to raw protocol per request. After popping the
  `ReqTx`, the consumer should call `HologramCacheService/put`
  and `/get`, not `kernel::send` with raw enum variants.
- `test-step6-lru-eviction-via-service` — same raw-protocol
  pattern as 4–5. Eviction is observable at the helper-verb
  level (a `get` after overflow returns `:None` for that probe
  in the result vec); raw-protocol wiring adds no coverage the
  helper path doesn't already cover.

**Recommended fix:** rewrite each test to use
`HologramCacheService/get` / `HologramCacheService/put`,
preserving the test scenarios (multi-client, eviction, etc.).
This is exactly arc 119 step 7's job.

### Tier 2 — Stale call shape (right vantage, wrong arguments)

**1 test in `crates/wat-lru/wat-tests/lru/CacheService.wat`:**

- `wat-lru::test-cache-service-put-then-get-round-trip` — calls
  the helper verbs `:wat::lru::put` and `:wat::lru::get` (right
  vantage), but with the pre-arc-119 single-item signatures.
  Allocates a single `Channel<Option<i64>>` shared between Put
  ack and Get reply, which only made sense under the old
  per-item protocol.

**Recommended fix:** allocate two separate channels (PutAck
channel + reply channel for batch get); wrap the singletons in
batch-of-one. Same scope as arc 119 step 7.

### Tier 3 — Soft mumbles (right vantage, idiomatic concerns)

**~5–6 tests across telemetry / console / WorkUnit, none failing
vocare in a falsifying sense:**

- `crates/wat-telemetry/wat-tests/telemetry/Service.wat` and
  `crates/wat-telemetry-sqlite/wat-tests/telemetry/Sqlite.wat` —
  both manually destructure the Handle tuple into `req-tx` and
  `ack-rx` via `(:wat::core::first handle)` /
  `(:wat::core::second handle)`. Right verbs called
  (`batch-log`, `spawn`, `HandlePool::pop`); the destructure step
  is idiomatic wat for tuples but boilerplate-y. Low severity.
- `wat-tests/console.wat` — multi-writer test wraps workers in
  `spawn-thread` lambdas with unused `_in`/`_out` substrate pipe
  args. The pattern IS what `spawn-thread` requires; the test
  doesn't explain why. Low severity; worth a brief comment.
- `crates/wat-telemetry/wat-tests/telemetry/WorkUnit.wat` —
  `test-build-counter-metric`, `test-build-duration-metric`,
  `test-collect-metrics-*` call `WorkUnit/scope::build-counter-
  metric` and `WorkUnit/scope::collect-metric-events` directly.
  These are internal helpers of `WorkUnit/scope`; consumers
  should call `WorkUnit/scope` or `WorkUnit/make-scope` and let
  orchestration happen inside. The full-scope round-trip
  (`test-make-scope-ships-counter`) exists separately. The slice
  tests look like "prove each piece before composing"
  scaffolding that may be permanent — but they live in a
  consumer-crate wat-tests directory, where a reader would think
  "I should know about `scope::build-counter-metric`." They
  shouldn't. **Medium-low severity; deferred as task #211.**

### Files definitively at right vantage (sample)

- `crates/wat-lru/wat-tests/lru/LocalCache.wat` — pure
  LocalCache consumer surface
- `crates/wat-holon-lru/wat-tests/holon/lru/HologramCache.wat` —
  calls `HologramCache/make`, `/put`, `/get`, `/len`
- `wat-tests/holon/Hologram.wat` — calls `Hologram/make`, `/put`,
  `/get`, `/capacity`, `/len` throughout

## Counts

| Tier | Count |
|---|---|
| Tier 1 — Wrong vantage | **4 tests** (HologramCacheService.wat) |
| Tier 2 — Stale call shape | **1 test** (CacheService.wat) |
| Tier 3 — Soft mumbles | ~5–6 tests, none falsifying |

**Total Tier 1 + Tier 2: 5 tests** — exactly the count surfaced
by arc 119 step 6's workspace test baseline. No drift beyond the
known scope.

## Summary judgment

The ground is solid. The 5 known-failing wat-tests are the only
Tier 1 / Tier 2 violations in the wat-rs codebase. No silent
drift has accumulated.

The Tier 3 mumbles are real but minor:
- Two destructure-noise issues are pure idiom (not vantage).
- The Console multi-writer test could use a comment.
- The WorkUnit.wat slice tests are the only Tier 3 finding worth
  follow-up — tracked as task #211 outside arc 119 scope.

Arc 119 step 7's consumer sweep targets exactly the 5 tests this
audit confirms are the full violation set. The brief
(`BRIEF-CONSUMER-SWEEP.md`) frames it as a discipline correction;
the audit confirms the discipline holds elsewhere.

## What this proves

Vocare works. The principle (caller-perspective verification)
holds across the codebase. When the principle was violated
(HolonLRU's step-tests written before helper verbs existed), the
violation shows up as raw-protocol code that vocare flags
directly. The 5 known violations are not the tip of an iceberg —
they're the iceberg.

## Cross-references

- `.claude/skills/vocare/SKILL.md` — the ward applied here.
- `docs/CONVENTIONS.md` § "Caller-perspective verification" — the
  principle.
- `docs/SERVICE-PROGRAMS.md` § "Audience boundary" — the audience
  separation that informs each file's vantage.
- `docs/arc/2026/04/119-holon-lru-put-ack/BRIEF-CONSUMER-SWEEP.md`
  — step 7's brief, targeting the 5 violations this audit
  confirms.
- Task #211 — Tier 3 follow-up for WorkUnit.wat.
