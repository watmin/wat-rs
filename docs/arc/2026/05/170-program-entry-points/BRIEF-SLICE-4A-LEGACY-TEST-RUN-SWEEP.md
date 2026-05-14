> **⚠ SUPERSEDED 2026-05-14** — This BRIEF's scope was wrong-direction. It would have swept all 32 legacy callers to `:wat::test::run-hermetic`, validating the arc 170 slice 3 phase C regression that collapsed both deftest forms into process-spawning. The user surfaced the conflation: *"non-hermetic test using a process or a thread? only hermetic should be a process."*
>
> The architectural correction (full account preserved in `INTERSTITIAL-REALIZATIONS.md` § 2026-05-14) rescoped this single slice into 5 stones:
>
> - **4a-α** (#308) — mint `:wat::test::run-thread` + `failure-from-thread-died` + `run-thread-driver` + standalone deftest. See `BRIEF-SLICE-4A-ALPHA-MINT-RUN-THREAD.md`.
> - **4a-β** (#313) — sweep 32 callers (23 → run-thread, 9 → run-hermetic).
> - **4a-γ** (#314) — flip deftest macro body to run-thread.
> - **4c-α** (#315) — delete legacy wat wrappers.
> - **4c-β** (#316) — rename `run-thread` → `run`; `run-thread-driver` → `run-driver`.
>
> Preserved as failure-engineering artifact per `feedback_inscription_immutable`. The conflation that drove this wrong-direction BRIEF — pattern-matching stdio-capture from process onto thread without reading runtime.rs — is named in INTERSTITIAL-REALIZATIONS.md as a discipline-failure record for future-me.

---

# Arc 170 Slice 4a BRIEF — sweep legacy `:wat::test::run*` → `:wat::test::run-hermetic`

**Phase:** Slice 4a (consumer sweep). First of the closure-paperwork sweep series (4a → 4b wat-cli Stone B → 4c substrate Rust deletion → 4d Phase H clippy → 4e INSCRIPTION).
**Predecessors:** All FD-multiplex Phases 1A–3 + amendment shipped at `61217c7..bed1a71`. Substrate-side spawn-process is deadlock-safe, FD-clean, shutdown-aware, lock-step canonical. The modern test macros `:wat::test::run-hermetic` (Layer 1) and `:wat::test::run-hermetic-with-io<I,O>` (Layer 2) ALREADY EXIST at wat/test.wat:574 + (Layer 2 below) and ALREADY route through spawn-process. deftest + deftest-hermetic ALREADY use them.

**Goal:** Sweep the 32 active call sites of the LEGACY function-style wrappers (`:wat::test::run`, `:wat::test::run-ast`, `:wat::test::run-hermetic-ast`) over to the modern `:wat::test::run-hermetic` macro. After this slice, the legacy wrappers in wat/test.wat have ZERO callers and can be deleted in slice 4c.

## The doctrine this enforces

Per `project_one_spawn_per_concern`:
> *"The substrate exposes exactly TWO spawn primitives. No others survive. `:wat::kernel::spawn-thread` — the ONLY way to make a thread. `:wat::kernel::spawn-process` — the ONLY way to make a process. Test convenience macros are allowed ON TOP — but every test site must use the macro."*

The legacy `:wat::test::run*` functions are the LAST consumers of `:wat::kernel::run-sandboxed*` (which ARE the legacy substrate primitives that wrap spawn-program / fork-program-ast). Killing the callers unblocks killing those substrate primitives in slice 4c.

## Migration patterns

The legacy wrappers take **source-string** or **forms** and produce a `RunResult`. The modern macro takes a **body AST**. Three migration shapes:

### Pattern P1 — `(:wat::test::run "(some-literal-source-string)" stdin)` (5 sites)

The source is a STRING LITERAL in the test file. Mechanical rewrite: parse the string contents as wat forms and inline them as the body of `run-hermetic`.

Before:
```scheme
(:wat::test::run "(:user::main)" (:wat::core::Vector :wat::core::String))
```

After:
```scheme
(:wat::test::run-hermetic (:user::main))
```

If the source-string contains MULTIPLE top-level forms, wrap them in `(:wat::core::do ...)`:

```scheme
(:wat::test::run-hermetic
  (:wat::core::do
    <form1>
    <form2>
    <form3>))
```

`stdin :wat::core::Vector<wat::core::String>` parameter DROPS entirely — Layer 1's child uses ambient stdio via `(:wat::kernel::println ...)` / `(:wat::kernel::readln)`; the test's stdin shape was for the legacy string-buffer substrate. If a test ACTUALLY needs to drive stdin, it migrates to Layer 2 (`run-hermetic-with-io`) instead — flag those for case-by-case review.

### Pattern P2 — `(:wat::test::run-ast forms stdin)` (18 sites)

The caller passes pre-built `forms :Vector<wat::WatAST>`. Two sub-shapes:

**P2a — forms is a literal `(:wat::core::Vector :wat::WatAST <form1> <form2> ...)` at call-site:**
Inline the forms into a do-block body:

```scheme
;; Before
(:wat::test::run-ast
  (:wat::core::Vector :wat::WatAST
    '(:user::main))
  (:wat::core::Vector :wat::core::String))

;; After
(:wat::test::run-hermetic (:user::main))
```

**P2b — forms is a let-binding or computed value (runtime Vec<WatAST>):**
This pattern doesn't trivially migrate — `run-hermetic` takes body as a compile-time AST, not a runtime Vec. STOP and surface; the test likely needs a different shape entirely. Mark these and report; don't force-migrate.

### Pattern P3 — `(:wat::test::run-hermetic-ast forms stdin)` (9 sites)

Same as P2 but the legacy call already implies hermetic semantics. Migration is identical to P2.

## Substrate edits — NONE in this slice

This slice is PURE CALL-SITE SWEEP. No edits to:
- src/ (Rust)
- wat/test.wat (legacy wrappers stay; deleted in slice 4c)
- wat/kernel/sandbox.wat / hermetic.wat (deleted in slice 4c)

## Edits in scope

Search-and-migrate the 32 call sites:

```bash
grep -rE ":wat::test::(run|run-ast|run-hermetic-ast)\b" wat-tests/ tests/ crates/ examples/
```

(Filter to actual code, not .md or commented lines.)

For each site:
1. Identify the pattern (P1, P2a, P2b, P3).
2. Apply the migration.
3. If P2b (computed forms) or any other shape that doesn't fit: **STOP for that site, surface in SCORE delta**. Don't delete the test; don't bandage; surface.

## Scorecard (8 rows)

| Row | What | Evidence |
|-----|------|----------|
| A | Zero active call sites of `:wat::test::run` remain | `grep -rE ":wat::test::run[^-A-Za-z]" wat-tests/ tests/ crates/ examples/` returns 0 active (non-comment, non-.md) lines |
| B | Zero active call sites of `:wat::test::run-ast` remain | similar grep, 0 hits |
| C | Zero active call sites of `:wat::test::run-hermetic-ast` remain | similar grep, 0 hits (note: `run-hermetic` modern macro is allowed — only `run-hermetic-ast` retires) |
| D | New call sites use `:wat::test::run-hermetic` (Layer 1) — body-form, no string-stdin | grep shows the migrated sites use the macro form |
| E | `cargo build --release --workspace --tests` clean | build output |
| F | Workspace test failure count ≤ pre-slice baseline (post-Phase-3: 11 failures) | `cargo test --release --workspace --no-fail-fast` shows ≤ 11 failures; ideally the test count INCREASES (former legacy callers now run as run-hermetic tests) |
| G | Any P2b or non-trivial migration site surfaced in SCORE honest deltas | SCORE doc names each site that couldn't migrate cleanly + the reason |
| H | wat/test.wat's LEGACY wrappers (`:wat::test::run`, `run-ast`, `run-hermetic-ast` define-functions at lines 194, 233, 253) untouched | grep wat/test.wat shows these defines still present (slice 4c deletes them) |

## Constraints

- DO NOT edit src/ Rust files.
- DO NOT edit wat/test.wat's legacy wrappers (deleted in slice 4c — leave them dangling-with-zero-callers post-this-slice).
- DO NOT edit wat/kernel/sandbox.wat or wat/kernel/hermetic.wat (deleted in slice 4c).
- DO NOT modify the modern `:wat::test::run-hermetic` macro at wat/test.wat:574+.
- Per `project_one_spawn_per_concern`: prefer Layer 1 (`run-hermetic`) over Layer 2 (`run-hermetic-with-io`). If a site genuinely needs typed-channel I/O, use Layer 2; otherwise body-form Layer 1.
- Per `feedback_inscription_immutable`: don't edit Slice C SCORE or historical INSCRIPTIONs.

## STOP-at-first-red

- `cargo build` fails mid-sweep → STOP at the breaking site; report.
- Workspace test failure count regresses (>11) → STOP. Surface the regression class.
- Encounter > 5 P2b sites (computed forms that can't migrate) → STOP. Surface; we'll handle them differently.

## On completion

Write `SCORE-SLICE-4A-LEGACY-TEST-RUN-SWEEP.md` as a sibling. 8 rows. Note honest deltas — especially:
- Any P2b sites (computed-forms callers) that couldn't migrate
- Any callers that needed Layer 2 instead of Layer 1
- Any tests that were better deleted than migrated (if any)

Do NOT commit. Orchestrator commits atomically after independent verification.

Per `feedback_simple_is_uniform_composition`: this is 32 uniform site changes. Predicted runtime in EXPECTATIONS.
