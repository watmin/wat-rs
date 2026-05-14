# Arc 170 Slice 4a-β BRIEF — sweep 32 legacy callers to canonical macros

**Task:** #313
**Phase:** Slice 4a-β — second stone of the corrected 4a → 4c chain (see `INTERSTITIAL-REALIZATIONS.md` § 2026-05-14 for the rescope rationale).
**Predecessors:** Slice 4a-α SHIPPED at `ddb3cad`. The mint is on disk:
- `:wat::test::run-thread` (Layer 1, thread transport, body-AST shape) at `wat/test.wat:688-698`
- `:wat::test::run-thread-driver` at `wat/test.wat:649-663`
- `:wat::test::failure-from-thread-died` at `wat/test.wat:622-638`
- Standalone deftest at `wat-tests/run-thread.wat` proves Ok-path + Err-path GREEN

## Goal

Sweep 32 active call sites of the LEGACY function-style wrappers to their canonical modern macros. This stone is PURE CALL-SITE SWEEP — no mint, no macro flip, no deletion. After this slice, the legacy `:wat::test::run` / `run-ast` / `run-hermetic-ast` defines in wat/test.wat have ZERO callers and become safe to delete in 4c-α.

## The destination split (critical — this is what changed from 5cf134d's BRIEF)

| Legacy macro | Mechanism | Destination |
|---|---|---|
| `:wat::test::run` (string entry; 5 sites) | spawn-program → THREAD | `:wat::test::run-thread` |
| `:wat::test::run-ast` (forms entry; 18 sites) | spawn-program-ast → THREAD | `:wat::test::run-thread` |
| `:wat::test::run-hermetic-ast` (forms entry; 9 sites) | fork-program-ast → PROCESS | `:wat::test::run-hermetic` |

**Total: 23 → `run-thread` + 9 → `run-hermetic` = 32 site migrations.**

The old BRIEF at 5cf134d swept all 32 to `run-hermetic`, conflating the two transports. The correction: thread is the default; hermetic is the explicit isolation marker. Each callsite migrates to the macro that matches its ORIGINAL transport, not to a single destination.

## Migration patterns (full detail; mirror old BRIEF's P-decomposition)

The legacy wrappers take **source-string** or **forms** and produce a `RunResult`. The modern macros take a **body AST**. The patterns and their corrected destinations:

### Pattern P1 — `(:wat::test::run "(literal-source-string)" stdin)` — 5 sites, → `run-thread`

The source is a STRING LITERAL in the test file. Parse the string contents as wat forms; inline them as the body of `run-thread`:

```scheme
;; Before
(:wat::test::run "(:user::main)" (:wat::core::Vector :wat::core::String))

;; After
(:wat::test::run-thread (:user::main))
```

Multiple top-level forms in the string wrap in `(:wat::core::do ...)`:

```scheme
(:wat::test::run-thread
  (:wat::core::do
    <form1>
    <form2>))
```

The `stdin :Vector<String>` parameter DROPS — Layer 1 children use ambient stdio via the three substrate services. If a test ACTUALLY needs to drive stdin (uses readln in the body), it migrates to Layer 2 (`run-hermetic-with-io` — process-side typed-channel IO) instead. Surface those as deltas; do NOT force-migrate to `run-thread` if the test depends on real stdin.

### Pattern P2a — `(:wat::test::run-ast literal-forms-vector stdin)` — most of 18 sites, → `run-thread`

The caller passes `forms :Vector<wat::WatAST>` as a literal `(:wat::core::Vector :wat::WatAST <form1> <form2>)` at the call site. Inline the forms as the body of `run-thread`:

```scheme
;; Before
(:wat::test::run-ast
  (:wat::core::Vector :wat::WatAST
    '(:user::main))
  (:wat::core::Vector :wat::core::String))

;; After
(:wat::test::run-thread (:user::main))
```

Multi-form vectors wrap in `(:wat::core::do ...)` same as P1.

### Pattern P2b — `(:wat::test::run-ast computed-forms stdin)` — 0-5 expected sites, → STOP and surface

If `forms` is a let-binding, function call, or otherwise computed at runtime (not a literal at the call site), the body-AST shape doesn't fit — `run-thread` and `run-hermetic` take a compile-time body, not a runtime `Vec<WatAST>`. **STOP for that site, surface in SCORE delta.** Don't force-migrate; the test likely needs a different shape entirely (perhaps Layer 2, perhaps a redesign).

Mid-sweep STOP threshold: > 5 P2b sites encountered → STOP the whole slice, surface; we'll handle those differently.

### Pattern P3 — `(:wat::test::run-hermetic-ast forms stdin)` — 9 sites, → `run-hermetic`

Same migration shape as P2a, but the destination is `run-hermetic` (process transport, body-AST shape) since the legacy form already implied hermetic semantics:

```scheme
;; Before
(:wat::test::run-hermetic-ast
  (:wat::core::Vector :wat::WatAST
    '(:user::main))
  (:wat::core::Vector :wat::core::String))

;; After
(:wat::test::run-hermetic (:user::main))
```

Same P2b sub-pattern applies: if forms is computed, STOP and surface.

### Scope parameter handling

All four legacy forms accept an `:Option<String>` `scope` parameter as the third arg. DROP it entirely per the modern macros' contracts — scope was leaked substrate plumbing; hermetic.wat:106-117 confirms it was never functional (returned Failure on `:Some`). If a test passes `(:wat::core::Some "scope-name")`, drop the arg; no replacement needed.

## Substrate edits — NONE in this slice

This slice is PURE CALL-SITE SWEEP. No edits to:

- `src/` Rust
- `wat/test.wat`'s LEGACY defines at lines 194/228/253 (stay; 4c-α deletes them)
- `wat/test.wat`'s `deftest` macro at line 294 (stays; 4a-γ flips its body)
- `wat/test.wat`'s `deftest-hermetic` macro at line 326 (stays)
- `wat/test.wat`'s modern run-thread / run-hermetic / run-thread-driver / run-hermetic-driver families (stay; the targets of the sweep)
- `wat/kernel/sandbox.wat` (stays; 4c-α deletes)
- `wat/kernel/hermetic.wat` (stays; 4c-α deletes)
- Past INSCRIPTIONs / SCORE-*.md / DEFERRAL-VIOLATIONS.md (immutable per `feedback_inscription_immutable`)
- The new `wat-tests/run-thread.wat` from 4a-α (stays; verification target, not sweep target)

## Edits in scope

Enumerate the 32 call sites:

```bash
grep -rE ":wat::test::(run|run-ast|run-hermetic-ast)\b" wat-tests/ tests/ crates/ examples/ \
  | grep -v "^Binary" \
  | grep -v -E "^[^:]+:[^:]+://" \
  | grep -v "\.md:"
```

Filter to active code, not comments or markdown. For each site:

1. Identify the pattern (P1, P2a, P2b, P3).
2. Apply the migration per the destination split above.
3. If P2b (computed forms) OR any other shape that doesn't fit: STOP for that site, surface in SCORE.

## Scorecard (8 rows, YES/NO with grep evidence)

| Row | What | Evidence |
|-----|------|----------|
| A | Zero active call sites of `:wat::test::run` remain | `grep -rE ":wat::test::run[^-A-Za-z]" wat-tests/ tests/ crates/ examples/` returns zero active (non-comment, non-.md) lines |
| B | Zero active call sites of `:wat::test::run-ast` remain | similar grep with `run-ast` returns zero hits |
| C | Zero active call sites of `:wat::test::run-hermetic-ast` remain | similar grep — note `run-hermetic` modern macro is allowed; only `run-hermetic-ast` retires |
| D | New thread-based call sites use `:wat::test::run-thread` | grep `run-thread` shows ~23 callers in the swept files (delta from baseline) |
| E | New hermetic call sites use `:wat::test::run-hermetic` | grep `run-hermetic` shows +9 callers in the swept files |
| F | `cargo build --release --workspace --tests` clean | build shows Finished; zero errors |
| G | Workspace test failure count ≤ pre-slice baseline (post-4a-α: 9 failures) | `cargo test --release --workspace --no-fail-fast` shows ≤ 9 failures; ideally MORE tests are in the PASSED set (former legacy callers now run as canonical-macro tests) |
| H | Any P2b or non-trivial migration site surfaced in SCORE | SCORE doc names each site that couldn't migrate cleanly + the reason; OR explicitly states "no P2b sites; 32/32 mechanical" |

## Constraints (HARD)

- DO NOT commit. Orchestrator commits atomically after independent verification.
- DO NOT edit `src/` Rust.
- DO NOT touch wat/test.wat's legacy defines (lines 194/228/253) — leave them dangling-with-zero-callers post-sweep; 4c-α deletes them.
- DO NOT touch wat/test.wat's `deftest` macro (line 294) — 4a-γ flips its body.
- DO NOT modify `wat/test.wat`'s modern macros (run-thread / run-thread-driver / failure-from-thread-died / run-hermetic / run-hermetic-driver / run-hermetic-with-io). They're the targets of the sweep, not subjects.
- DO NOT edit `wat/kernel/sandbox.wat` or `wat/kernel/hermetic.wat`. 4c-α deletes them.
- DO NOT delete the new `wat-tests/run-thread.wat` from 4a-α; it's verification.
- DO NOT touch past INSCRIPTIONs / SCORE-*.md / DEFERRAL-VIOLATIONS.md.

## STOP-at-first-red

- `cargo build` fails mid-sweep → STOP at the breaking site; report the file + line + error.
- Workspace test failure count REGRESSES (>9, the post-4a-α baseline) → STOP; surface the regression class (which tests went red).
- Encounter > 5 P2b sites (computed forms) → STOP the whole slice; surface; we'll handle differently.
- Encounter a site that genuinely needs Layer 2 (`run-hermetic-with-io`) — surface; don't force-migrate to run-thread.

## Implementation protocol

Per `feedback_simple_is_uniform_composition`, this is 32 uniform site changes. Per `feedback_iterative_complexity`, build small stepping stones:

1. **Enumerate first.** Run the grep; get the 32 sites in a list; classify by pattern (P1/P2a/P2b/P3). Note any P2b or Layer-2-escalation sites BEFORE starting edits.
2. **Migrate by pattern.** Do all P1 first (5 sites). Run cargo build. If clean, do all P2a (18). Build. If clean, do all P3 (9). Build. Then run the full workspace test suite.
3. **STOP-at-first-red** between batches. A clean build between batches means the previous batch is settled.
4. **Surface P2b in SCORE before final commit.** Each P2b site gets its own delta entry: file:line, the computed-forms expression, and the recommended alternative shape (Layer 2 migration, redesign, etc.).

## Test-first note

This slice is a SWEEP, not a feature mint. Test bodies are inherited from the legacy call shapes; no new tests to write before the migrations. The `wat-tests/run-thread.wat` from 4a-α already proves the macro functional; the sweep verifies it works at scale across the existing test fleet.

If a migrated test goes RED that was previously GREEN, that's substrate-level evidence of a regression — STOP and surface; don't paper over.

## On completion

Write `SCORE-SLICE-4A-BETA-SWEEP-LEGACY-CALLERS.md` as a sibling. 8 rows. Each YES/NO with grep evidence. Honest deltas surfaced — especially:

- Any P2b sites (computed-forms callers) that couldn't migrate; their file:line + the computed expression + recommended alternative
- Any callers that needed Layer 2 (`run-hermetic-with-io`) instead of Layer 1; same detail
- Any tests that were better deleted than migrated (DO NOT delete; flag for orchestrator judgment)
- Per-file site count for the sweep (e.g., "tests/wat_arc113_cross_fork_cascade.rs: 2 sites; migrated 2 P2a → run-thread")
- Calibration record filled per the EXPECTATIONS template

Do NOT commit. Orchestrator commits atomically.

## Per-file decomposition guidance

The 32 sites span multiple test files. Per `feedback_test_file_composition`, complex test files compose named helpers in one file. When migrating a file with N sites, do them all in one pass per file. Don't bounce between files.

If multiple sites in one file share a helper function that wraps the legacy macro, MIGRATE THE HELPER — the call sites pointing at the helper don't change. Surface in SCORE: "file X: 1 helper migration covered N call sites."
