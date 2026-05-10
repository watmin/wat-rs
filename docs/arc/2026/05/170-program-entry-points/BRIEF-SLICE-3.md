# Arc 170 slice 3 — consumer sweep + testing-lib three-layer rebuild

## Goal

Sweep the 545 RED failures slice 2 produced, AND rebuild the
testing tooling on the typed-channel three-layer API per
TIERS.md. The wat-level surface from slice 2 (spawn-process,
:user::main 4-arg + ExitCode, walkers) is the platform; slice 3
is where consumers reach polished form on it.

This slice ships in two phases, atomic-committed at the end per
recovery doc § 7 atomic-commit pattern:

- **Phase A (opus)** — design + land testing-lib platform: three-layer
  macros (`run-hermetic`, `run-hermetic-with-io`), `wat/test.wat`
  deftest macro rebuild, `wat/std/hermetic.wat` rebuild on typed-
  channel API, `wat/std/sandbox.wat` migration. **DO NOT COMMIT;
  leave dirty tree.**
- **Phase B (sonnet)** — mechanical sweep: ~277 test fixture
  migrations remaining after phase A lands (3-arg main → 4-arg,
  fork-program* → spawn-process, spawn-program* → spawn-process or
  spawn-thread). **DO NOT COMMIT; leave dirty tree.**
- **Orchestrator** atomically commits both phases as ONE atomic
  commit when workspace = 0-failed.

## Read first (in order)

1. `docs/arc/2026/05/170-program-entry-points/BRIEF-SLICE-3.md` — this doc
2. `docs/arc/2026/05/170-program-entry-points/EXPECTATIONS-SLICE-3.md` — the scorecard
3. `docs/arc/2026/05/170-program-entry-points/TIERS.md` — three-layer testing API spec; section "Today's `wat/std/hermetic.wat` is a tier-2 wrapper — gets rebuilt in arc 170 slice 3" is THIS slice's spec
4. `docs/arc/2026/05/170-program-entry-points/DESIGN.md` — full arc; slice 3 section "Tooling rebuild — testing-lib three-layer API"
5. `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-2.md` — workspace fail breakdown is your input map (268 deftest_* + 277 other)
6. `docs/arc/2026/05/170-program-entry-points/REALIZATIONS-SLICE-1.md` — six framing passes; pass 6 bandaid-bounded
7. `docs/COMPACTION-AMNESIA-RECOVERY.md` § 6 (FM 5/9/11/12/16) + § 7 atomic-commit pattern

## Slice 2 context (workspace state)

Workspace post-slice-2: **1594 passed / 545 failed**.

Failure breakdown:
- **268 deftest_*** — `wat/test.wat`'s deftest macro generates legacy
  3-arg `:user::main`; walker fires per `expanded_user` scoping
- **277 other** — test fixtures embedding 3-arg `:user::main` source
  + legacy `fork-program*` / `spawn-program*` callsites + 1 lib
  test + 2 arc112 typed-channel scheme probes

Stdlib silently survives (per `freeze.rs:599-619` user-source-only
walker scoping); `wat/std/{sandbox,hermetic}.wat` use legacy
verbs internally and continue to work through legacy dispatch
arms. Slice 3 rebuilds them; slice 4 retires the legacy arms.

Phase A targets the 268 deftest_* failures (rebuild deftest
macro to emit new contract shape). Phase B targets the 277 other.

## Branch + commit policy

- **Active branch**: `arc-170-program-entry-points`
- Phase A: spawn opus; opus DOES NOT COMMIT; orchestrator
  inspects dirty tree
- Phase B: spawn sonnet against dirty tree (sees phase A's
  changes); sonnet DOES NOT COMMIT
- Orchestrator atomically commits BOTH phases as ONE atomic
  commit when workspace = 0-failed
- DO NOT push to main; arc-170-program-entry-points only
- DO NOT edit SCOREs 1, 1B, 1C, 2 (immutable)

---

## Phase A — testing-lib three-layer rebuild (opus)

### Scope

#### A.1 Mint testing-lib three-layer macros

Per TIERS.md three-layer testing API:

```scheme
;; LAYER 1 — the 90% case
;; Macro hides ALL fn ceremony; user writes body directly
(:wat::test::run-hermetic
  (:wat::core::assert-eq 42 (my-test-helper)))
;; Expansion: (spawn-process (fn [] -> :wat::core::nil body))
;;   → drains result → returns RunResult { failure :Option<Failure> }
;; If body panics, harness catches via spawn-process child
;; protocol; surfaces structured failure

;; LAYER 2 — the 9% case (typed-channel I/O tests)
;; Macro introduces rx + tx as bindings in body scope
(:wat::test::run-hermetic-with-io<I,O> inputs
  (... body sends/receives via rx and tx ...))
;; Expansion: (spawn-process (fn [rx <- :Receiver<I> tx <- :Sender<O>] -> :wat::core::nil body))
;;   → feeds inputs via parent-side tx; drains parent-side rx;
;;   returns parsed Vec<O> + failure
```

Place these in `wat/test.wat` (where deftest lives) or split
into `wat/test/run-hermetic.wat`. Agent picks file organization.

#### A.2 Rebuild `wat/test.wat` deftest macros

Today's deftest expansion generates legacy 3-arg `:user::main`.
Rebuild the expansion to emit the new contract shape — either:
- (a) deftest expands to `(run-hermetic body)` directly (cleanest;
  uses A.1's Layer 1 macro); the harness handles the test
  wrapping; existing deftest callsites unchanged at the call
  site
- (b) deftest expands to a 4-arg `:user::main` + ExitCode shape
  matching the new contract; harness does its own wrapping

Pick (a) if the macro expansion can route through run-hermetic
cleanly; pick (b) if the deftest's prelude/load-mechanism doesn't
fit run-hermetic's scope. Surface choice + reasoning.

deftest-hermetic + make-deftest + make-deftest-hermetic get the
same treatment.

#### A.3 Rebuild `wat/std/hermetic.wat`

Today's hermetic.wat is the SPECIFIC, wat-level wrapper for
fork-program-ast (string source → forked process). Rebuild on
spawn-process(fn) + the three-layer API:

- Old: `(:wat::kernel::run-sandboxed-hermetic-ast forms stdin scope) -> RunResult`
- New: thin wrapper over `(spawn-process fn)` + drain →
  RunResult. OR: hermetic.wat retires entirely, replaced by the
  three-layer macros from A.1 (run-hermetic IS the new
  hermetic).

Agent picks: rebuild as Layer 1/2 macros (replaces hermetic.wat)
OR keep hermetic.wat with new internals. Surface choice.

#### A.4 Migrate `wat/std/sandbox.wat`

Today: uses spawn-program / spawn-program-ast internally.
Slice 4 retires those. Sandbox.wat needs to migrate to
spawn-process(fn) or spawn-thread(fn) per the substrate's
two-mode taxonomy.

Investigate sandbox's actual semantics (in-thread? forked? both?)
and pick the right migration.

#### A.5 No mechanical sweep in phase A

Phase A is platform work only. The 277 other failures (test
fixtures) are phase B's territory. After phase A lands, the
deftest_* 268 should be GREEN (because deftest emission changed);
the remaining 277 still RED. That's phase B's input.

### Phase A commit policy

**DO NOT COMMIT.** Leave dirty tree for phase B to consume.
Phase B sweeps the residual; orchestrator commits both as one
atomic commit when workspace = 0-failed.

### Phase A predicted runtime

90-180 min opus.

### Phase A honest deltas (if surfaced, report; don't bridge)

- **deftest expansion shape (a) or (b)** — agent picks; reasons
- **hermetic.wat rebuild vs retirement** — agent picks; reasons
- **sandbox.wat migration shape** (spawn-process vs spawn-thread)
- **Slice 1b honest delta B reprise (match-arm pattern bindings)** —
  if rebuilding deftest body using match patterns, may trip
  this; surface
- **deftest's prelude/load-mechanism integration** with run-hermetic
- **FM 5 trap** — TODOs verboten

### Phase A reporting

Phase A agent reports to chat with:
- Dirty-tree state summary (files changed; lines)
- Choices made (deftest shape, hermetic disposition, sandbox
  shape)
- Workspace state via `./scripts/cargo-test-summary.sh`
  (expected: 268 deftest_* now green; 277 other still RED)
- Honest deltas surfaced
- Wall-clock minutes

After phase A reports, orchestrator inspects + briefs phase B.

---

## Phase B — mechanical sweep (sonnet)

### Scope

Mass mechanical migration of remaining ~277 test fixtures to
the new contract shapes. Categories (per slice 2 SCORE):

#### B.1 3-arg `:user::main` → 4-arg `:user::main`

Test fixtures embedding legacy 3-arg main shape:

```scheme
;; OLD
(:wat::core::define
  (:user::main (stdin :IOReader stdout :IOWriter stderr :IOWriter) -> :nil)
  ...body...)

;; NEW (per arc 170 contract)
(:wat::core::defn :user::main
  [stdin  <- :wat::io::IOReader
   stdout <- :wat::io::IOWriter
   stderr <- :wat::io::IOWriter
   argv   <- :wat::core::Vector<wat::core::String>]
  -> :wat::kernel::ExitCode
  ...body...
  (:wat::core::u8 0))  ;; explicit ExitCode return
```

If the test body returned implicit `:nil`, append explicit
`(:wat::core::u8 0)` as final form.

#### B.2 `fork-program*` → `spawn-process(fn)`

Test fixtures calling legacy fork-program / fork-program-ast:

```scheme
;; OLD
(:wat::kernel::fork-program src-string :None)
(:wat::kernel::fork-program-ast forms)

;; NEW
(:wat::kernel::spawn-process worker-fn)
;; where worker-fn satisfies :user::process contract:
;; [rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>] -> :wat::core::nil
```

Migration shape depends on the test's intent. Surface ambiguity
to orchestrator; don't guess.

#### B.3 `spawn-program*` → `spawn-process(fn)` OR `spawn-thread(fn)`

Per DESIGN's two-mode taxonomy:
- spawn-process(fn) — real OS-process fork (memory boundary; tier 2)
- spawn-thread(fn) — parent's-world thread (memory shared; tier 1)

Investigate each callsite's semantics; pick the right mode.

#### B.4 1 lib test + 2 arc112 probes

`runtime::tests::assert_eq_failure_renders_actual_and_expected` —
embedded 3-arg main fixture; downstream of B.1.

2 arc112 typed-channel scheme probes that depended on legacy main
shape — investigate; migrate or remove.

### Phase B commit policy

**DO NOT COMMIT.** Leave dirty tree for orchestrator atomic
commit.

### Phase B predicted runtime

60-180 min sonnet.

### Phase B honest deltas

- **Ambiguous migration shape** — surface to orchestrator;
  don't guess
- **Test intent unclear** — surface; some tests may need
  redesign rather than mechanical migration

### Phase B reporting

Phase B agent reports to chat with:
- Files changed (count; lines)
- Categories swept (B.1, B.2, B.3, B.4 counts)
- Workspace state via `./scripts/cargo-test-summary.sh`
  (expected: 2107 passed 0 failed — all 545 swept; net +15
  arc170 contract tests = 2122 expected; verify)
- Ambiguities surfaced (if any)
- Wall-clock minutes

After phase B reports, orchestrator verifies + atomically
commits both phases.

---

## Critical syntax shapes (across both phases)

Per arc 167 + arc 109 + arc 153:

- fn-form: `(:wat::core::fn [name <- :T ...] -> :Ret body)`
- defn: `(:wat::core::defn :name [params] -> :Ret body)`
- Type names: `:wat::core::nil` (NOT bare `:nil`); FQDN for all
  substrate types
- No whitespace inside `<>`, `:(...)`, `:fn(...)`, `:[...]`
- No inner colon before generic

## Branch state at slice 3 start

```
$ git log --oneline -3
9879e3b  arc 170 slice 2: SCORE — 19/19 rows pass, Mode A clean, ~180 min
09d7b04  arc 170 slice 2: wat-level surface — spawn-process + ExitCode + walkers
e7bbf95  arc 170 slice 2: BRIEF + EXPECTATIONS REDRAFTED — full settled foundation
```

`./scripts/cargo-test-summary.sh` baseline: `passed: 1594 failed: 545`

Post-phase-A expected: ~1862/277 (268 deftest_* green; 277 other still RED)
Post-phase-B expected: 2122/0 (all 545 swept; +15 arc170 contract tests)

## SCORE artifact

After both phases ship + atomic commit, orchestrator writes
SCORE-SLICE-3.md documenting both phases + atomic-commit pattern
+ honest deltas + calibration.
