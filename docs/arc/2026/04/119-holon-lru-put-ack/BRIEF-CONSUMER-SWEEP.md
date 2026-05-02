# Arc 119 — Sonnet Brief: Consumer Sweep (step 7)

**Status:** durable record of the brief sent to sonnet for arc
119's consumer sweep (step 7 of the execution checklist). This is
not a mechanical rewrite — it is a **discipline correction**.

## Read this first

Before any edits, read:

1. `/home/watmin/work/holon/wat-rs/docs/CONVENTIONS.md` § "Caller-perspective verification" — the principle this sweep enforces.
2. `/home/watmin/work/holon/wat-rs/docs/SERVICE-PROGRAMS.md` § "Audience boundary" — separates wire-protocol pedagogy (service implementers) from consumer-API pedagogy (service consumers).
3. `/home/watmin/work/holon/wat-rs/.claude/skills/vocare/SKILL.md` — the ward that defends caller-perspective; what it flags and why.
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md` § "Realization (surfaced 2026-05-01)" — the framing for this brief.

## What this is NOT

This is NOT "wrap every singleton in a batch-of-one and thread an
ack-tx." That mechanical rewrite would land tests that still test
the wrong layer.

## What this IS

A **discipline correction**: the wat-tests are at the wrong
vantage. They speak for the implementer when they should speak
for the consumer. The sweep moves them to caller-perspective.

## The principle

> All code is measurable from the caller's perspective. That's
> the interface to confirm.

A consumer of HologramCacheService calls
`(HologramCacheService/get req-tx reply-tx reply-rx probes)`. They
do not hand-build Request enum constructors. They do not call raw
`:wat::kernel::send`. They do not manually walk
`Result<Option<T>, ThreadDiedError>`.

The wat-tests in a consumer crate's `wat-tests/` directory should
look like consumer call sites. Anything else is at the wrong
vantage and teaches the next reader the wrong shape.

Wire-protocol pedagogy (raw send/recv, manual Request
construction, manual Result-Option chains) lives in
`wat-rs/wat-tests/service-template.wat` — that file's caller IS
the service implementer. Consumer-crate tests do NOT mirror that
style.

## In-scope files (5 failing wat-tests at step-6 baseline)

1. `crates/wat-lru/wat-tests/lru/CacheService.wat` — 1 test
   - `test-cache-service-put-then-get-round-trip`
   - Already at the right vantage (calls `:wat::lru::put` /
     `:wat::lru::get` helper verbs). Just needs the channel split
     + batch-of-one wrap because the helper signatures changed.
2. `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` — 4 tests
   - `test-step3-put-only`
   - `test-step4-put-get-roundtrip`
   - `test-step5-multi-client-via-constructor`
   - `test-step6-lru-eviction-via-service`
   - **All four are at the wrong vantage.** They hand-build
     `(:wat::holon::lru::HologramCacheService::Request::Put k v)`
     and call `:wat::kernel::send` directly. They speak for the
     implementer.

The other two HolonLRU tests (`test-step1-spawn-join`,
`test-step2-counted-recv`) are passing at baseline; leave them
alone.

## Out of scope

- `holon-lab-trading/` — separate workspace, separate downstream
  arc. The lab gets fixed in its own session against the new
  substrate. Workspace boundary discipline.
- Any file outside `crates/wat-lru/wat-tests/` and
  `crates/wat-holon-lru/wat-tests/`.
- Substrate `.wat` files in `crates/*/wat/` — those are step
  2-5's territory and shipped at HEAD.
- Rust source in `src/`, `Cargo.toml`, anything else.

## How the rewrite works

### LRU (1 test) — minimal change

The test already calls helper verbs. Two issues:

1. The shared `reply-pair` channel is now structurally wrong —
   post-arc-119, Get's reply-rx wants `Vector<Option<V>>` and
   Put's ack-rx wants `unit`. They are different channel types.
   Allocate two separate channels.
2. The `(:wat::lru::put req-tx reply-tx reply-rx k v)` call is
   now `(:wat::lru::put req-tx ack-tx ack-rx entries-vec)`.
   Wrap the singleton in a batch-of-one.
3. Same for `(:wat::lru::get req-tx reply-tx reply-rx k)` →
   `(:wat::lru::get req-tx reply-tx reply-rx probes-vec)`. The
   return is now `Vector<Option<V>>`; pull the single result with
   `(:wat::core::first results)` to get `Option<V>`.

Test scenario unchanged: spawn → put → get → assert "hit".

### HolonLRU (4 tests) — vantage rewrite

These tests preserve their **scenarios** but lose their **wire-
protocol style**. Each scenario rewrites to use the helper verbs
sonnet minted in step 4:

- `:wat::holon::lru::HologramCacheService/get`
- `:wat::holon::lru::HologramCacheService/put`

Read those verb signatures in
`crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`
(at HEAD). Mirror the LRU test's call shape but with HolonAST
keys/values and the `HologramCacheService/` prefix on each verb.

Test scenarios to preserve:

- **step3-put-only**: do N puts; assert eviction or len behavior.
- **step4-put-get-roundtrip**: put, get, assert the value is
  returned.
- **step5-multi-client-via-constructor**: two clients (each pops
  its own req-tx from the HandlePool); each does a
  put-then-get; assert each client sees its own data.
- **step6-lru-eviction-via-service**: fill cache past capacity;
  assert oldest entries evict.

Each test loses its `(:wat::kernel::send tx (Request::...))`
calls, the manual `(:wat::core::match (:wat::kernel::recv rx) ...)`
arms, and the inner Option-extraction noise. Each gains the
helper-verb call shape — clean, observable, what a consumer
would write.

If a scenario CANNOT be expressed via helper verbs, that's a gap
in the consumer surface, not a license to drop back to raw
protocol. Surface the gap to the orchestrator.

## Required disciplines

Same as `BRIEF-LRU-RESHAPE.md` §§ "Required disciplines" —
inherited by reference. Read that section.

The two patterns you'll need most for the helper-verb call sites:

```scheme
;; Send pattern — one Result/expect, no Option layer:
(:wat::core::Result/expect -> :wat::core::unit
  (:wat::kernel::send tx val)
  "X: tx disconnected — peer died?")

;; Recv pattern — Option/expect WRAPPING Result/expect:
(:wat::core::Option/expect -> :T
  (:wat::core::Result/expect -> :wat::core::Option<T>
    (:wat::kernel::recv rx)
    "X: rx peer died — protocol violation")
  "X: rx channel closed — peer dropped tx?")
```

The helper verbs already encode these patterns. When you call
`:wat::lru::get` / `:wat::lru::put` /
`HologramCacheService/get` / `HologramCacheService/put` in the
test, you don't write the patterns yourself. The helper verbs do.
That's the WHOLE POINT — the test is a consumer; consumers call
helpers.

The patterns above only appear when the test allocates its own
channels (channel-make + extracting tx/rx pairs). That's setup,
not protocol-driving.

## Validation gate

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release --workspace 2>&1 | grep -oE "[0-9]+ (passed|failed|ignored)" | sort | uniq -c
```

Target: 1480 passed, 0 failed, 0 ignored.

If any test still fails, the substrate-as-teacher diagnostic
stream tells you exactly which one and why. Iterate.

## Constraints

- Edit ONLY:
  - `crates/wat-lru/wat-tests/lru/CacheService.wat`
  - `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
- Do NOT touch substrate `.wat` files (steps 2-5; shipped).
- Do NOT touch `holon-lab-trading/` (separate arc).
- Do NOT touch `src/*.rs`, `Cargo.toml`, or any docs.
- Do NOT add new tests; existing tests should be updated.
- If a test scenario can't be expressed via helper verbs, STOP
  and report. Do not fall back to raw protocol — that's the
  layer this sweep is moving AWAY from.

## Reporting back

When done (or blocked):
1. `git status --short` — file list (ONLY the two test files).
2. `git diff --stat` — line counts.
3. Final `cargo test --release --workspace` summary —
   confirm 1480/0/0.
4. For each rewritten HolonLRU test: one sentence on what
   scenario it now exercises through the helper verbs (so the
   orchestrator can verify the scenario coverage was preserved).
5. Any judgment calls beyond the brief (especially: helper-verb
   gaps you discovered).

## Cross-references

- `docs/CONVENTIONS.md` § "Caller-perspective verification" — the
  principle being enforced.
- `docs/SERVICE-PROGRAMS.md` § "Audience boundary" — the audience
  separation that explains why these tests rewrite.
- `.claude/skills/vocare/SKILL.md` — the ward that catches tests
  at the wrong vantage. (Use it to self-check before reporting.)
- `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md` § "Realization"
  — what surfaced this discipline correction.
- `docs/arc/2026/04/119-holon-lru-put-ack/BRIEF-LRU-RESHAPE.md`
  — full discipline rules + send/recv patterns inherited here.
- `crates/wat-lru/wat/lru/CacheService.wat` (HEAD) — substrate
  reference for `:wat::lru::get` / `:wat::lru::put` signatures.
- `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`
  (HEAD) — substrate reference for `HologramCacheService/get` /
  `HologramCacheService/put` signatures.
