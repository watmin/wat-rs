# Arc 130 — Cache Services Pair-by-Index — INSCRIPTION

## The closing

Arc 130 shipped over a week of substantive work. Drafted
2026-05-01; paused the same day after the slice 1 sweep was
killed mid-run; resumed in stages 2026-05-03 + 2026-05-06 across
4 sonnet sweeps + 1 atomic commit. Six discrete units of work
totaling ~110 min of sonnet wall-clock + the orchestrator
paperwork captured in 6 SCORE docs:

| Unit | Wall clock | Mode | Date |
|---|---|---|---|
| Slice 1 (substrate reshape, killed sweep) | ~4 hrs before kill | Mode B-discipline-failure | 2026-05-01 |
| Slice 1 (substrate consumer sweep — Vector/len → Vector/length) | ~100s | Mode B clean diagnostic | 2026-05-06 |
| Slice 1 (test file rebuild — wat-lru) | ~7 min | Mode A clean | 2026-05-06 |
| Slice 2 (HolonLRU substrate reshape) | ~5 min | Mode A clean | 2026-05-06 |
| Slice 2 (HolonLRU test rebuild + retire :should-panic) | ~7 min | Mode A clean | 2026-05-06 |
| Slice 3 (this paperwork) | small | A | 2026-05-06 |

**The substrate is cleaned up.** Both LRU services use
post-arc-130 pair-by-index discipline. The 9 LRU
`:should-panic("channel-pair-deadlock")` annotations that held
the deadlock-pattern's structure in place are RETIRED. Workspace
ships at **0 failed tests** with no panic crutches. Both rebuilt
test files are reference templates for future complectens-style
substrate-consumer test rebuilds.

## What this arc shipped

### Substrate (both crates)

`Handle = (ReqTx, ReplyRx)` client view + `DriverPair = (ReqRx, ReplyTx)`
driver view. spawn factory pre-allocates N (request-channel,
reply-channel) pairs at startup; HandlePool<Handle> for clients;
Vector<DriverPair> for the driver thread. select fires at index
`i` in the request-side; same index locates the matching ReplyTx
in the driver-side. **No per-call channel allocation.** Reply<V>
enum unifies GetResult + PutAck so both verbs share ONE reply
channel per slot.

Helper verbs `get` / `put` simplified from 3-channel-end signatures
to single-Handle signatures. Each helper does send-AND-recv
internally per arc 110's contract.

- `crates/wat-lru/wat/lru/CacheService.wat` (slice 1, 2026-05-01
  + post-arc-146 consumer sweep 2026-05-06): generic `<K,V>` shape;
  parametric typealiases.
- `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`
  (slice 2, 2026-05-06): concrete K=V=HolonAST shape; non-parametric
  typealiases.

### Tests (both crates)

Both test files rebuilt bottom-up using the `/complectens` spell:

- `crates/wat-lru/wat-tests/lru/CacheService.wat` — 5 layers + 3
  Level-3-taste sub-helpers, 325 LOC, factory prelude pattern.
- `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
  — 7 layers (5 baseline + 2 optional eviction/multi-client) + 5
  Level-3-taste sub-helpers, 501 LOC, factory prelude pattern.

Layer 0 lifecycle, Layers 1+ helper-verb scenarios composing to
multi-key + eviction + multi-client coverage. Each helper has its
own deftest where the parent invokes it; sub-helpers used in
exactly one place per the SKILL's Level-3-taste exemption.

Plus 2 arc-119 proof file updates (`step-A-spawn-shutdown.wat` +
`step-B-single-put.wat`) — mechanical edits to the new substrate
shape; step-B's `:should-panic("channel-pair-deadlock")` annotation
retired.

**All 9 LRU `:should-panic` annotations RETIRED:**
- 8 in `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
- 1 in `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat`

The only remaining `:should-panic` in the active workspace test
files is wat-sqlite arc-122's mechanism self-test — which is
testing should-panic's own correctness, NOT a deadlock crutch.

## The story — substrate-as-teacher cascade's most concrete demonstration

Arc 130's pause is the most fertile single pause in the
substrate's history. The week between 2026-05-01 (kill) and
2026-05-06 (cleanup) saw 12+ cascade arcs ship, each closing a
substrate gap arc 130 had surfaced:

### Round 1 — structural deadlock prevention (2026-05-01)

The killed slice 1 sweep had two failure modes that became
substrate-level fixes:

- **Arc 131 — HandlePool sender-bearing.** Sweep diagnosed: the
  agent's diagnostic work surfaced HandlePool not counting as
  Sender-bearing for arc 117's scope-deadlock check. Arc 131
  fixed the substrate; tests in scope-deadlock-prevention land.
- **Arc 132 — default 200ms time-limit.** Surfaced as part of the
  killed sweep's diagnostic context; addressed default test-runtime
  behavior. Later amended (arc 132 amend 2026-05-03) to 1000ms
  after the multi-binary cargo test contention surfaced.

### Round 2 — discipline + diagnostic substrate (2026-05-02 → 03)

The killed sweep's monolithic-deftest failure-mode birthed:

- **Arc 135 — complectens cleanup sweep.** The /complectens spell
  was named + applied to 9 files + 22 deftests across the
  workspace. Test-file composition became a failure-engineering
  primitive (REALIZATIONS.md preserves the discipline).
- **Arc 138 — errors carry coordinates.** The killed sweep's
  diagnostic blindness ("expected hit, actual <missing>" with
  no narrowing surface) was the trigger; arc 138 shipped
  coordinates project-wide on 8 error types + 6 substrate cracks.
- **Arc 139 — generic-T tuple return propagation.** Surfaced
  during arc 135's sweep as a substrate observation; closed
  same session.
- **Arc 140 — sandbox-scope leak teaching diagnostic.** Two-span
  diagnostic for the `:test::*` defines-don't-capture pattern;
  closed same session.
- **Arc 142 — runes cleanup.** Three spell-amendment + six-file
  sweep; canonical rune format for complectens / perspicere /
  vocare.

### Round 3 — substrate vocabulary correction (2026-05-02 → 03)

The killed sweep's runtime panic ("unknown function:
:wat::core::reduce") became:

- **Arc 143 — define-alias.** :wat::runtime::define-alias
  primitive + the introspection trio (lookup-callable / signature-of
  / body-of). The `:wat::list::reduce` alias unblocked arc 130's
  prior stepping stone.
- **Arc 144 — uniform reflection foundation.** Binding enum + the
  6 form-kinds reflectable uniformly; doc_string field paved-road
  for arc 141.
- **Arc 146 — Dispatch entity for container methods.** Surfaced
  arc 144 slice 3's polymorphic-handler anti-pattern as Mode B
  canary; arc 146 shipped Dispatch as a new entity kind. The
  `:wat::core::Vector/length` named in arc 146 became the next
  cascade link arc 130 needed.
- **Arc 148 — arithmetic + comparison correction.** Polymorphic-
  handler retirement for numeric ops; same Dispatch + per-Type-leaves
  pattern arc 146 established.
- **Arc 150 — variadic `:wat::core::define`.** Closed the
  asymmetry between defmacro (variadic since arc 029) and define
  (strict-arity); enabled arc 148's variadic arithmetic surface.

The cascade's ENTIRE compounding signature: arc 130's pause
forced the substrate to fix gaps in HOW IT DIAGNOSES + DEFERS +
NAMES + RETIRES anti-patterns. By the time we resumed cleanup
2026-05-06, every substrate primitive arc 130's tests would call
WAS callable, WAS reflectable, WAS structurally sound.

### Round 4 — the cleanup itself (2026-05-06)

With the substrate cascade arcs all shipped + working tree clean,
arc 130's three remaining work items closed in 4 sweeps:

1. **Substrate consumer sweep** (Mode B clean diagnostic) — fixed
   `:wat::core::Vector/len` → `:wat::core::Vector/length` consumer
   refs that arc 146's Dispatch entity rename hadn't swept. The
   sweep surfaced the cascade's NEXT chain link: arc 110's
   `silent-disconnect→panic-loud` discipline interacting with the
   test's deliberate drop-handle-pre-recv pattern.

2. **wat-lru test rebuild** (Mode A clean) — wiped the prior
   complectens-violating test file + rebuilt 5 layers
   bottom-up. Two adaptations surfaced + handled within scope:
   - **Factory prelude pattern** — sandbox-scope leak workaround
     (`(:wat::test::make-deftest :deftest-lru ...)` with helpers
     in the prelude). `:test::*` defines don't capture into deftest
     sandboxes; the factory pattern is the canonical shape.
   - **Tuple-out for scope-deadlock check** — `(driver, value)`
     returned from inner let* so spawn/pool/handle drop before
     outer Thread/join-result, satisfying arc 117/126's check.

3. **HolonLRU substrate reshape** (Mode A clean) — mechanical
   pattern-application from wat-lru's template; concrete-typing
   adaptation (K=V=HolonAST; no `<V>` heads on typealiases).
   Substrate file structurally valid; consumer tests fail with
   TYPE-MISMATCH errors as predicted (resolved by the next sweep).

4. **HolonLRU test rebuild + :should-panic retirement** (Mode A
   clean) — both adaptations from the wat-lru rebuild transferred
   cleanly across the crate boundary. 7 layers (5 baseline + 2
   optional for eviction/multi-client). Plus 2 arc-119 proof
   files updated in place. All 9 LRU `:should-panic` annotations
   retired.

The 2026-05-06 cleanup work atomic-committed (substrate + tests
+ SCORE docs) per `feedback_no_broken_commits.md` — the working
tree was deliberately dirty between sweep 2a and sweep 2b
because sweep 2b's test rebuild required sweep 2a's substrate;
orchestrator committed both atomically when workspace = 0 failed.

## Foundation principles established (carry forward)

1. **Pair-by-index is the canonical service-substrate channel
   discipline.** When N clients need request-reply with a service,
   pre-allocate N pairs at spawn; clients pop a Handle; driver
   holds matching DriverPair at the same index. NO per-call
   channel allocation. Mirrors Console's existing pattern.

2. **Reply<V> enum unifies multi-verb reply channels.** When a
   service has multiple verbs (Get + Put), one Reply enum with
   per-verb variants on a single shared reply channel per slot
   is cleaner than per-verb channel families.

3. **Helper-verb signatures take Handle, not channel-ends.** Bury
   the send-AND-recv lifecycle inside the helper per arc 110's
   contract. Callers don't reckon with disconnect; the helper
   panics-loud with a meaningful message.

4. **The complectens spell + factory prelude + tuple-out are the
   canonical test-file-shape patterns for substrate-service tests.**
   Both LRU rebuilt test files demonstrate the discipline:
   bottom-up layered structure, per-helper deftests, top-down
   dependency graph, deftest body 3-7 lines, factory prelude
   for sandbox-scope-leak avoidance, tuple-out for scope-deadlock
   compliance. Templates for future substrate-consumer test
   rebuilds.

5. **Atomic-commit-across-coordinated-sweeps** discipline (added
   to recovery doc Section 7 today). When sweep B logically
   requires sweep A's output, working tree stays dirty between
   sweeps; orchestrator commits atomically when workspace =
   0-failed; commit message names both sweeps. Extends
   `feedback_no_broken_commits.md`.

## What this arc closes — counted

- **2 substrate files reshaped** to pair-by-index discipline
  (wat-lru CacheService.wat + wat-holon-lru HologramCacheService.wat)
- **2 substrate-side helper-verb signature simplifications**
  (3-channel-ends → single-Handle, in both crates)
- **2 substrate Reply enums introduced** (parametric Reply<V> in
  wat-lru; concrete Reply in wat-holon-lru)
- **6 OLD typealias families retired** in HolonLRU (GetReplyTx,
  GetReplyRx, GetReplyPair, PutAckTx, PutAckRx, PutAckChannel,
  ReqTxPool); equivalent typealias retirements in wat-lru shipped
  2026-05-01
- **2 test files rebuilt complectens-style** with factory prelude
  + tuple-out patterns
- **2 arc-119 proof files updated** (step-A-spawn-shutdown,
  step-B-single-put)
- **9 LRU `:should-panic("channel-pair-deadlock")` annotations
  RETIRED**
- **The deadlock pattern's structural foundation removed from
  the substrate** — arc 126's compile-time check no longer fires
  on cache-service consumers (no per-call channel allocation
  remaining)

## What this arc unlocks

- **Arc 119 closure** (#209) — its parent dependency on arc 130's
  substrate work is satisfied. Closure paperwork tractable.
- **Arc 109 K.holon-lru slice** (#195) — was waiting on arc 130's
  HolonLRU substrate reshape. Tractable.
- **Arc 109 v1 closure trajectory** — major chain link closes;
  the impeccable-foundation milestone the user has been chasing
  approaches by another major link's worth.
- **Future substrate-service rebuilds** — the wat-lru +
  HolonLRU test files are reference templates; their
  complectens + factory + tuple-out patterns transfer cleanly.

## What this arc does NOT close

- **Arc 119's INSCRIPTION** — separate arc's paperwork; not
  arc 130's deferral.
- **Arc 109 K.holon-lru rename + arc 109 K.thread-process rename**
  — separate arc 109 slices; not arc 130's deferrals.
- **Arc 109 v1 INSCRIPTION** — major future ship; not arc 130's
  scope.

These are downstream arc trajectories, NOT deferrals OF arc 130.
Arc 130's substrate work + test work + `:should-panic` retirement
are all DONE per the DESIGN's "What's still owed" section.

## The honest record — what the killed sweep taught

Per `feedback_inscription_immutable.md` and `project_failure_engineering.md`:
the killed slice 1 sweep (2026-05-01) is preserved as evidence,
not erased. Two artifacts capture the failure:

- **`docs/arc/2026/05/130-cache-services-pair-by-index/complected-2026-05-02/`**
  — sonnet's working state at kill time (substrate.wat + test.wat
  + README.md). Per user direction 2026-05-02 ("we need to know
  what bad looks like to make good"), this directory is a
  permanent calibration set for the /complectens spell.
- **`REALIZATIONS.md`** — names test-file composition as a
  failure-engineering primitive. The principles cited there
  ("the deftest body is short BECAUSE the layers exist") drove
  the wat-lru + HolonLRU test rebuilds 4 days later.

The killed sweep + its preservation = the substrate-as-teacher
cascade's foundational lesson. It's why the cascade arcs
shipped. It's why the cleanup completed. The cost of getting
this right (the user's emotional bandwidth, the session time
across the week, the 12+ cascade arcs of substrate work) is the
artifact.

## Cascade arcs (cross-references)

Arcs that shipped during arc 130's pause, each closing a substrate
gap arc 130 had surfaced:

- arc 131 (HandlePool sender-bearing for scope-deadlock)
- arc 132 + amend (deftest default time-limit 200→1000ms)
- arc 135 (complectens cleanup sweep — 9 files / 22 deftests)
- arc 138 (errors carry coordinates project-wide; 8 error types
  + 6 substrate cracks)
- arc 139 (generic-T tuple return propagation)
- arc 140 (sandbox-scope leak teaching diagnostic)
- arc 142 (runes cleanup — canonical spell-rune format)
- arc 143 (`:wat::runtime::define-alias` + reflection trio)
- arc 144 (uniform reflection — Binding enum, 6 form-kinds)
- arc 146 (Dispatch entity for container methods)
- arc 148 (arithmetic + comparison polymorphic-handler retirement)
- arc 150 (variadic `:wat::core::define`)

12 arcs cascade-shipped. Each has its own INSCRIPTION; each
references arc 130's pause as the trigger. The substrate-as-teacher
cascade's most concrete demonstration in the project's history.

## Calibration record

- **Slice 1 (substrate) total**: ~4 hrs killed work (2026-05-01) +
  ~7 min successful rebuild (2026-05-06)
- **Slice 1 (consumer sweep)**: ~100s — the substrate-vocabulary
  fix that the cascade made trivial
- **Slice 2 substrate**: ~5 min — pure pattern-application from
  wat-lru's template
- **Slice 2 tests**: ~7 min — patterns transferred cleanly across
  crate boundaries
- **Total successful sonnet wall-clock 2026-05-06**: ~24 min
  across 4 sweeps
- **Mode A on 3/4 sweeps; Mode B clean diagnostic on 1/4** —
  exactly the discipline mode the substrate-as-teacher pattern
  predicts when foundation gaps are surfacing
- **Mutual-agreement protocol**: held end-to-end on 4/4 sweeps
  + the atomic commit per `feedback_no_broken_commits.md`

## Cross-references

- **Inside arc 130**: DESIGN.md, REALIZATIONS.md, FOLLOWUPS.md;
  per-sweep BRIEFs (slice-1, slice-1-RELAND, slice-2, substrate-
  consumer-sweep, test-file-rebuild, holon-lru-substrate-reshape,
  holon-lru-test-rebuild); per-sweep SCOREs; the calibration set
  at `complected-2026-05-02/`
- **Cascade arcs (12)**: 131, 132, 135, 138, 139, 140, 142, 143,
  144, 146, 148, 150 — see each arc's INSCRIPTION
- **Discipline**: `feedback_no_broken_commits.md` (atomic commit
  pattern), `feedback_inscription_immutable.md` (calibration set
  preserved unchanged), `project_failure_engineering.md` (failure
  as data + artifacts as teaching), `feedback_test_file_composition.md`
  (top-down dependency graph in one file), COMPACTION-AMNESIA-RECOVERY.md
  § 7 (atomic-commit-across-coordinated-sweeps amendment today),
  § FM 5 (workaround-vs-stop, exercised + held), § FM 11
  (deferral discipline, nothing deferred from arc 130's scope)
- **Spell library**: `.claude/skills/complectens/SKILL.md` —
  shipped during arc 130's cascade; both rebuilt LRU test files
  are reference templates for it

## Status

**Arc 130 closes here.** Both LRU services use post-arc-130
pair-by-index discipline. All 9 LRU `:should-panic` crutches
retire. Workspace at 0 failed. The week's work is complete.

**Arc 109 v1 closure trajectory clearer.** Arc 130's chain link
closes; arc 109's K.holon-lru + K.thread-process slices become
tractable; arc 119's parent dependency satisfied.

The methodology IS the proof. The cascade compounded — sequentially
(arc 130 → 12 cascade arcs → arc 130 cleanup) AND laterally
(wat-lru patterns → HolonLRU patterns via clean transfer). The
discipline propagates via artifacts. The arc record is the
project's memory.

---

*the killed sweep taught the cascade. the cascade closed the
foundation gaps. the cleanup proves the foundation. forward
progress only. what is inscribed is inscribed.*

**PERSEVERARE.**
