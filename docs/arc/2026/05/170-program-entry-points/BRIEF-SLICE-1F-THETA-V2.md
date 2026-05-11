# Arc 170 slice 1f-θ V2 — BRIEF (hermetic test restructure per complectens)

> ⚠️ **SUPERSEDED — DO NOT EXECUTE.**
> V2 sonnet attempted to restructure the existing implementer-vantage tests
> per /complectens. Sonnet fixated on preserving wire-protocol gymnastics
> (spawn + Add + Remove events) — the V2 anchoring on existing test shape
> was itself a vocare violation: tests at implementer vantage when consumer
> vantage is the recommended pattern.
>
> User direction 2026-05-10: "sonnet is fixated on prior state that's
> failing vs just testing what matters... remove the poison."
>
> **Supersedes:** `BRIEF-SLICE-1F-THETA-V3.md` — deletes the existing
> tests entirely; writes fresh consumer-vantage tests against the ambient
> stdio surface.
>
> Original V2 content below preserved as historical record.
>
> ---

**Opus.** V1 of this BRIEF (committed at `8774eef`) was wrong-shaped — it told sonnet to fix deadlocks by tweaking flat-let bind order. The agent flailed and was killed. **The actual problem is structural**: the 3 hermetic test files are monolithic let bodies with anonymous bindings, violating `/complectens` (`/home/watmin/work/holon/wat-rs/.claude/skills/complectens/SKILL.md`). The deadlock is a SYMPTOM of the discipline violation. The fix is to restructure the tests as **layered named helpers + per-layer deftests + 3-7 line final test bodies**.

**Supersedes** `BRIEF-SLICE-1F-THETA.md` (V1; preserved as historical record per "what is inscribed is inscribed").

## Slice surface

> *"Restructure the trio hermetic test files per `/complectens` — layered named helpers, per-layer deftests, top-down dependency graph in one file."*

## The canonical pattern (read first)

**Required reading:** `crates/wat-lru/wat-tests/lru/CacheService.wat` — the worked example. Header comment names the layer order; ONE `make-deftest` factory contains layered helpers in the prelude; per-layer deftests at the bottom each compose ONE helper in 1-3 lines.

```
;; Layer order — top-down, no forward refs:
;;   Layer 0 — :test::lru-spawn-and-drop       lifecycle, no requests
;;   Layer 1 — :test::lru-helper-get-empty     one get round trip
;;   Layer 2 — :test::lru-helper-put-one       one put round trip
;;   Layer 3 — :test::lru-helper-put-then-get  put-one + get-same-key
;;   Layer 4 — :test::lru-helper-get-many-keys multi-key probe alignment
```

Final deftest bodies in CacheService.wat are ~3 lines: `(:deftest-lru :test::test-3-put-then-get (:wat::test::assert-eq 42 (:test::lru-helper-put-then-get)))`. The proof tree is the layer chain.

**Also required:** read `.claude/skills/complectens/SKILL.md` in full before drafting. The four questions, the severity levels, the edge cases on inner-let shape (`make-bounded-channel` not factored; `HandlePool::finish` pop-before-finish; non-unit `Thread<I,O>` recv-before-join) all apply directly to the trio services.

## Why V1 was wrong

V1 said: "fix the flat let by introducing a nested let so `_ctrl-tx` drops before `recv`." This treats the deadlock as a binding-order bug. Sonnet implementing V1 spent its run wrestling with let-body type semantics, never getting to a passing test.

The actual problem: the tests are 30-50 line monolithic `let` bodies with 10+ anonymous sequential bindings doing spawn + register + send + recv + remove + cleanup. Per complectens, **this is a Level 1 lie** — when they fail, you don't know which unit broke. The deadlock is one of many possible failure modes hidden inside the monolith.

The right fix decomposes the monolith into layered named helpers, each individually proven.

## Scope

### Target files (3)

- `wat-tests/kernel/services/stdin.wat`
- `wat-tests/kernel/services/stdout.wat`
- `wat-tests/kernel/services/stderr.wat`

Each currently has 5 monolithic deftests (`spawn-shape`, `add-and-read`/`add-and-write`, `multi-thread-routing`, `remove-drops-entry`, `scope-drop-shutdown`) with flat-let bodies.

### The restructure (per file)

**Proposed layer structure** (refine at slice time after reading the existing tests; this is the working sketch):

| Layer | Name | What it tests |
|---|---|---|
| 0 | `:test::svc-spawn-and-shutdown` | spawn service, drop ControlTx in inner-let, join thread → expect Ok |
| 1 | `:test::svc-register-thread` | Layer 0 + send `Event::Add` with channel pair, drop senders, join |
| 2 | `:test::svc-roundtrip-one` | Layer 1 + send `Event::Read`/`Write`, recv reply/ack, then teardown |
| 3 | `:test::svc-remove-then-add` | Layer 1 + send `Event::Remove`, then Add again, then teardown |
| 4 | `:test::svc-multi-thread` | spawn service + register N threads + concurrent ops |

Each layer has:
- A named `define` in the `make-deftest` factory's prelude
- Its OWN deftest using the factory's alias (e.g., `(:deftest-stdin :test::test-0 (:wat::test::assert-eq 1 (:test::stdin-spawn-and-shutdown)))`)
- 3-7 lines in the deftest body — just `assert-eq` + named helper call

### The deadlock dissolves naturally

When each layer is its own named helper with bounded scope:
- Layer 0's `_ctrl-tx` drops at end of the layer-0 helper body
- Each helper returns a value (count, marker, etc.) the assertion verifies
- `Thread/join-result` is the last op; ControlTx already dropped → service exits cleanly
- Scope-deadlock is impossible if each helper is correctly shaped

This is what `scope-drop-shutdown` already does in V1 (the one passing test). Make EVERY test follow that shape.

### Out of scope

- No substrate Rust edits
- No new test scenarios beyond what the V1 tests intended to cover
- No changes to the wat-side service definitions (`wat/kernel/services/{stdin,stdout,stderr}.wat`)
- No changes to `:wat::test::deftest-hermetic` macro
- Don't commit yourself — orchestrator atomic-commits with SCORE

## Pre-flight verification (mandatory)

Before drafting wat code:

1. **Read `.claude/skills/complectens/SKILL.md` fully** — the four questions, severity levels, edge cases
2. **Read `crates/wat-lru/wat-tests/lru/CacheService.wat`** — pattern source
3. **Read `wat-tests/kernel/services/stdin.wat` current state** — note `scope-drop-shutdown` as the canonical correct shape; note the monolithic bodies in the others
4. **Sample the existing service definition** — `wat/kernel/services/stdin.wat` to confirm Event enum + Spawn return shape

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A | Each of 3 test files has ONE `make-deftest` factory with prelude containing layered named helpers | grep |
| B | Each layer (0-4) has its OWN deftest using the factory's alias | grep |
| C | Final deftest bodies are ≤ 7 lines (assertion + helper call) | line count per body |
| D | No deftest body exceeds ~10 anonymous sequential bindings | manual review |
| E | All hermetic tests pass (no deadlocks) | cargo test |
| F | Workspace failure count drops by ≥ 12 from 2151/48 baseline | cargo test count |
| G | Top-down dependency graph: no helper references a helper defined LATER in the file | manual review |
| H | `cargo check --release` green | clean |
| I | Only 3 files modified | git status |
| J | Honest deltas surfaced | per FM 5 |

**10 rows.**

## Honest delta categories (anticipated)

1. **Non-unit `Thread<I,O>` services need recv-before-join** — per complectens § "Non-unit Thread output requires recv-before-join". The trio services return `Thread<nil, nil>` so this MAY not apply, but verify at slice time and surface.

2. **`make-bounded-channel` should NOT be factored** — per complectens § "Cross-function tracing — DO NOT factor `make-bounded-channel` into a helper". Inline the `(make-bounded-channel ...) + first/second` triplet inside the helper that USES both halves; don't abstract.

3. **Embedded literal exception** — if any test has an `(:wat::test::program ...)` literal AST as a fixture, exempt that part from line-count metrics via rune (`;; rune:complectens(embedded-program) — <reason>`). Unlikely here but flag if it comes up.

4. **Test bodies may surface OTHER bugs after restructure** — once each helper is named and individually proven, layer 2+ tests may fail in ways layer 0/1 don't. That's the discipline working — failures localize. Surface count.

## Predicted runtime

**120-240 min opus.** Reading complectens + CacheService.wat + restructuring 3 test files is substantive design work. Each file is ~5 helper layers + ~5 deftests + per-layer composition checks.

**Hard cap:** 480 min.

## Reference

- `.claude/skills/complectens/SKILL.md` — the spell (load-bearing)
- `crates/wat-lru/wat-tests/lru/CacheService.wat` — canonical worked example (load-bearing)
- `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` — six-step layered test (alternate worked example)
- `docs/arc/2026/05/130-cache-services-pair-by-index/REALIZATIONS.md` — the discipline's origin doc
- V1 BRIEF (STALE): `BRIEF-SLICE-1F-THETA.md`
- Predecessors: 1f-β-i/ii/iii — the slices that introduced these complectens-violating test bodies

## Path forward post-slice-1f-θ V2

1. Orchestrator scores; atomic-commits deliverable + SCORE; pushes
2. Verify leak resolved (root-cause fix); run workspace test; check orphan count
3. Remaining ~36 failures — split between retired-verb tests (sibling slice) + wat-cli echo + OOM-SIGKILL
4. Arc 170 INSCRIPTION — once baseline near-zero
