# Arc 170 slice 3 phase E — SCORE (deftest macro rewrite)

**Date:** 2026-05-11
**Branch:** arc-170-program-entry-points
**Status:** BLOCKED — Mechanism A failed; orchestrator direction required before proceeding

## Scorecard verification

| Row | What | Pass criterion | Result |
|-----|------|----------------|--------|
| A | `:wat::test::deftest` macro body uses `run-hermetic` (no `run-sandboxed-ast` reference) | grep | BLOCKED — Mechanism A failed; deftest macro NOT rewritten |
| B | `:wat::test::deftest-hermetic` body uses `run-hermetic` (no `run-sandboxed-hermetic-ast`) | grep | BLOCKED — same reason |
| C | Mechanism A verified OR Mechanism B chosen with documented rationale | SCORE | PARTIAL — Mechanism A empirically falsified (see below); Mechanism B paths analyzed and documented; choice deferred to orchestrator |
| D | Workspace at 0 failed AFTER deftest rewrite | full cargo test | BLOCKED — no rewrite yet; baseline 2199/0 preserved |
| E | TestResult vs RunResult reconciled | grep + cargo test | ANALYZED — `:wat::test::TestResult` is a typealias of `:wat::kernel::RunResult`; no conversion needed; `failure_to_diagnostic` in `test_runner.rs` checks for the concrete `RunResult` struct name regardless |
| F | `cargo check --release` green | clean | PASS — workspace unchanged; baseline green |
| G | make-deftest factories disposition documented | SCORE | PASS — documented below; factories cascade through deftest/deftest-hermetic transitively |
| H | SCORE documents honest deltas (≥ 3) including hermetic-by-default note | this file | PASS — ≥ 3 honest deltas below |

**Rows A, B, D blocked by Mechanism A failure. Rows C, E, G, H complete. F passes on unchanged baseline.**

## Phase E1 — Mechanism A verification (empirical)

### Probe design

A minimal defmacro was written inline as a Rust test probe. The macro emits
`(:wat::core::forms define-1 define-2)` from its expansion:

```scheme
(:wat::core::defmacro
  (:my::probe (body :AST<wat::core::nil>) -> :AST<wat::core::nil>)
  `(:wat::core::forms
     (:wat::core::define (:my::probe::helper -> :wat::core::i64) 42)
     ~body))

(:my::probe
  (:wat::core::define (:my::probe::main -> :wat::core::i64) (:my::probe::helper)))
```

The probe freezes this source against `startup_from_source` and checks whether
`:my::probe::helper` is discoverable in the frozen symbol table.

### Probe result

**Mechanism A FAILED.** Freeze returned:

```
resolve: 3 unresolved reference(s):
  - :my::probe::helper (call head — not a builtin, not a registered function)
  - :my::probe::main (call head — not a builtin, not a registered function)
  - :my::probe::helper (call head — not a builtin, not a registered function)
```

Both `:my::probe::helper` (from the emitted `forms` wrapper) and `:my::probe::main`
(which called the helper) were unresolved. The `forms` wrapper was NOT spliced into
top-level scope — it remained a single `WatAST::List` node in `expand_all`'s output,
which was then treated as a non-define residue form by `register_defines`. The helper
define inside it was never registered.

### Root cause analysis

`(:wat::core::forms ...)` is a DATA-CAPTURE special form, not a top-level splicer.
Its runtime semantics (from `src/runtime.rs:9388-9409`) are:

> `(:wat::core::forms f1 f2 ... fn)` → `:wat::core::Vector<wat::WatAST>`

It captures its unevaluated arguments as a `Vector<WatAST>` value for use by
`run-sandboxed-ast` and `run-hermetic-ast`. It is NOT a mechanism for emitting
multiple top-level defines from a single macro expansion.

The `expand_all` loop (`src/macros.rs:463-473`) processes each macro expansion as ONE
`WatAST` node:
- If it is a `defmacro` form: register the new macro.
- Otherwise: append to `out` as a single element.

There is no recognition of a "splice wrapper" that would allow one macro call to
produce multiple top-level forms. `register_defines` (`src/runtime.rs:1440-1496`)
also processes forms one-by-one; only top-level `(:wat::core::define ...)` forms
register into `sym.functions`. A `(:wat::core::forms ...)` wrapper with nested
defines is pushed to `rest` (non-define residue) unchanged.

### Why this matters for the prelude

Under `run-sandboxed-ast`, the prelude defines go INSIDE the `forms` block:
```scheme
(:wat::core::define (name -> TestResult)
  (:wat::kernel::run-sandboxed-ast
    (:wat::core::forms
      (:wat::core::define (:my::helper ...) ...)  ;; prelude define
      (:wat::core::define (:user::main -> nil) body))
    argv stdin))
```
The prelude defines are registered in the CHILD SANDBOX's symbol table (a fresh
mini-freeze inside `run-sandboxed-ast`). They are NOT in the parent's symbol table.

Under `run-hermetic` (via `spawn-process`), the child is built from a
`ClosurePackage` (`src/spawn_process.rs:301`):
```rust
let world = startup_from_forms(package.prologue, None, loader)
```
The prologue is extracted from the fn closure by `extract_closure`
(`src/closure_extract.rs:134`), which walks the fn body for free symbols and pulls
transitive dependencies from the PARENT's `SymbolTable`. If the prelude defines are
NOT in the parent's `SymbolTable`, the closure extractor cannot find them, and the
child's prologue will be missing them.

**Bottom line:** A macro can emit only ONE top-level form. Prelude defines must be in
the parent's `SymbolTable` for the closure extractor to include them in the child's
prologue. These two facts are irreconcilable without one of the following changes.

## Mechanism B — viable paths (for orchestrator decision)

### B1 — Substrate splice mechanism in `expand_all`

Add recognition of a "multi-form splice" wrapper to `expand_all` in `src/macros.rs`.
When a macro expansion returns `(:wat::core::splice-forms define-1 define-2 test-define)`,
`expand_all` would flatten it into three separate top-level forms:

```rust
// Proposed change to expand_all (src/macros.rs:463-473):
for form in forms {
    let expanded = expand_form(form, registry, 0, env, sym)?;
    if is_defmacro_form(&expanded) {
        let def = parse_defmacro_form(expanded)?;
        registry.register(def)?;
    } else if is_splice_forms(&expanded) {
        // New: flatten inner forms into the top-level stream
        for inner in splice_forms_children(expanded) {
            out.push(inner);
        }
    } else {
        out.push(expanded);
    }
}
```

The deftest macro could then be rewritten as:
```scheme
(:wat::core::defmacro
  (:wat::test::deftest
    (name :AST<wat::core::nil>)
    (prelude :AST<wat::core::nil>)
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::splice-forms
     ~@prelude
     (:wat::core::define (~name -> :wat::test::TestResult)
       (:wat::test::run-hermetic ~body))))
```

**Pros:**
- Zero call-site changes. All 223 deftest call sites work unchanged.
- Prelude defines land at file top level; closure extractor finds them.
- Canonical Mechanism A semantics, just under a different form name.

**Cons:**
- Substrate change required (`macros.rs` + type check + name resolution must
  all recognize the new splice wrapper consistently).
- `register_defines`, `register_types`, `check_program`, `resolve_references`
  may all need updates to handle the new form in the residue stream.
- Risk: introduces a new special form that touches the entire freeze pipeline.

**Scope estimate:** Medium. `expand_all` change is 5-10 lines. The downstream
pipeline (resolve, type-check, register_defines) would need to be audited for
whether they already handle "unexpected list forms" gracefully or need explicit
recognition of the new wrapper.

### B2 — Remove prelude parameter from deftest API (call-site sweep)

Change the deftest macro signature from 3 args `(name prelude body)` to 2 args
`(name body)`. Prelude defines must move to FILE TOP LEVEL in the test source
file. The closure extractor will then include them in the child's prologue.

The new macro:
```scheme
(:wat::core::defmacro
  (:wat::test::deftest
    (name :AST<wat::core::nil>)
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::define (~name -> :wat::test::TestResult)
     (:wat::test::run-hermetic ~body)))
```

**Pros:**
- No substrate changes. Pure macro + call-site change.
- Cleaner API: prelude defines are visible to ALL tests in the file, not just
  one. Encourages shared helper extraction.

**Cons:**
- 54 call-site changes required (the 54 non-empty prelude deftest calls).
- API change: `make-deftest` factory signature also changes (its `default-prelude`
  arg becomes the pattern for file-top-level placement guidance, not actual code).
- Behavioural change: helpers defined at file top level are visible to ALL tests
  in the file. Under run-sandboxed-ast, each test had isolated prelude scope.
  Under B2, a prelude define from one test is available to other tests in the
  same file (name collisions possible if two tests define the same helper name).

**Scope estimate:** Low substrate risk; moderate call-site work. 54 sites, each
requiring the prelude define to be lifted to file top level (before the deftest call).

### B3 — Hybrid: run-hermetic for empty prelude, run-sandboxed-ast for non-empty

Keep `deftest` using `run-sandboxed-ast` for tests with non-empty preludes. Only
migrate tests with empty preludes to `run-hermetic`. Since the macro body is a
quasiquote template (no conditional logic), this requires either:
- Two separate macros: `deftest` (empty prelude → run-hermetic) and `deftest-with-prelude` (non-empty → run-sandboxed-ast)
- Or accepting that `deftest` stays on `run-sandboxed-ast` until B1 or B2 is resolved

**Pros:** No call-site changes. No substrate changes for the empty-prelude 169 sites.

**Cons:** Hybrid path leaves 54 tests on `run-sandboxed-ast`. Phase F cannot retire
`run-sandboxed-ast` until those 54 sites are migrated (some future arc). The macro
cannot distinguish empty vs. non-empty prelude at template-expansion time.

## TestResult vs RunResult reconciliation (Row E)

`:wat::test::TestResult` is declared in `wat/test.wat:37`:
```scheme
(:wat::core::typealias :wat::test::TestResult :wat::kernel::RunResult)
```

`failure_to_diagnostic` in `src/test_runner.rs:640-709` checks:
```rust
Value::Struct(s) if s.type_name == ":wat::kernel::RunResult" => s,
```

Since the typealias resolves to `RunResult` at the `Value::Struct` level (struct
values carry the concrete struct name, not the alias), the test runner sees
`:wat::kernel::RunResult` regardless of whether the deftest macro declares
`-> :wat::test::TestResult` or `-> :wat::kernel::RunResult`. Both are equivalent;
no conversion needed.

## make-deftest factory disposition (Row G)

`make-deftest` (`wat/test.wat:379-389`) expands to a `defmacro` whose body
expands to `(:wat::test::deftest ...)`. `make-deftest-hermetic` (`wat/test.wat:396-406`)
expands to `(:wat::test::deftest-hermetic ...)`.

**Both factories cascade transitively through deftest/deftest-hermetic.** If the
deftest/deftest-hermetic macro bodies are rewritten (under any Mechanism B variant),
the factories automatically benefit — no separate factory rewrite is needed.

The ambient-stdio test (`wat-tests/kernel/services/ambient-stdio.wat`) uses
`make-deftest-hermetic` with a non-empty prelude (5 helper defines). This file
is one of the 54 non-empty prelude call sites; it would need factory-level attention
under B2.

## Honest deltas

### Delta 1 — Mechanism A is structurally absent, not missing by oversight

`(:wat::core::forms ...)` is a data-capture form that returns `Vector<WatAST>`.
It was designed as the program-as-data payload mechanism for `run-sandboxed-ast`.
Splicing top-level forms from a macro expansion is a DIFFERENT CONCEPT. The absence
is not a gap to fill with a small tweak — `expand_all` needs an explicit new branch
(B1) or the call sites need to change (B2). There is no shortcut.

### Delta 2 — The prelude design is semantically incompatible with run-hermetic's closure model

Under `run-sandboxed-ast`: prelude + body run in an ISOLATED fresh freeze. Prelude
defines are registered in the CHILD's sandboxed symbol table. Each test has private
scope for its prelude helpers.

Under `run-hermetic` / `spawn-process`: the child's symbol table is built from a
`ClosurePackage.prologue` extracted from the parent's FROZEN symbol table. Prelude
defines must be in the PARENT's symbol table (registered at file-top-level freeze
time) for the closure extractor to find them. The semantics shift from "private
per-test sandbox" to "shared file-scope symbols inherited by the child."

This semantic change has consequences under B2: helper defines that two tests
independently define by the same name would collide at file top level. Under
run-sandboxed-ast, each sandbox was independent.

### Delta 3 — The empty-prelude 169 majority CAN be migrated without substrate change

169 of the 223 deftest call sites have empty prelude `()`. For these, the deftest
macro could be rewritten as a 2-arg form `(name body)` with `~body` passed directly
to `run-hermetic`. No prelude to splice; no top-level placement issue. The 169-site
migration is mechanically clean under B2 (the empty `()` just disappears).

The 54 non-empty prelude sites are the blocker. If they can accept B2's file-top-level
requirement, the full migration is achievable with B2 alone.

### Delta 4 — Hermetic-by-default performance note (from BRIEF)

When the deftest rewrite ships (under any Mechanism B variant), all 223 deftest
calls will fork an OS process per test. The current `run-sandboxed-ast` path runs
in-process (no fork). The `run-sandboxed-hermetic-ast` path already forks (arc 012).

Observed impact from T17 in `tests/wat_arc170_program_contracts.rs`: `run-hermetic`
adds ~2-10ms per test (fork + IPC overhead). For 223 tests × ~5ms = ~1.1s total
overhead added to the test suite. This is within acceptable range for a test suite
that already takes several seconds. Surface as expected behavior; no action required.

### Delta 5 — run-hermetic and run-sandboxed-ast have different failure message surfaces

Under `run-sandboxed-ast`, assertion failures produce structured `Failure` with
`AssertionFailed` message + actual/expected fields (the AssertionPayload cascade is
captured by `fork.rs::emit_panics_to_stderr`).

Under `run-hermetic` (via `spawn-process`), the panic chain emit is wired as of
Phase C′ (the spawn_process.rs gap was closed between phases C and D). Per T17b,
assertion failures do produce structured messages. This delta is RESOLVED — the
run-hermetic path has parity with run-sandboxed-ast for assertion failure messages.

## Files modified

| File | Change |
|------|--------|
| `tests/wat_arc170_program_contracts.rs` | Probe test added and removed (net zero change). Baseline preserved at 1207 lines. |

**No other files modified.** `wat/test.wat` is unchanged (deftest macro NOT rewritten).
Workspace at 2199 passed / 0 failed — same as pre-phase-E baseline.

## What's next (Phase F dependencies)

Phase F (retire `run-sandboxed-ast` / `run-sandboxed-hermetic-ast` substrate verbs)
CANNOT proceed until all callers are migrated. Current caller inventory in `wat/test.wat`:
- `run-ast` wrapper (`wat/test.wat:238`) calls `:wat::kernel::run-sandboxed-ast`
- `run-hermetic-ast` wrapper (`wat/test.wat:258`) calls `:wat::kernel::run-sandboxed-hermetic-ast`
- `deftest` macro (`wat/test.wat:311`) calls `:wat::kernel::run-sandboxed-ast`
- `deftest-hermetic` macro (`wat/test.wat:338`) calls `:wat::kernel::run-sandboxed-hermetic-ast`

Phase E (blocked) is a prerequisite for Phase F. Phase F is a prerequisite for Slice 4.

## Orchestrator decision needed

**One of the following before Phase E can proceed:**

1. **B1 chosen:** Orchestrator authors a substrate arc to add `(:wat::core::splice-forms ...)` recognition to `expand_all`, `register_defines`, `register_types`, `check_program`, and `resolve_references`. Phase E then rewrites deftest to use `splice-forms`. No call-site changes.

2. **B2 chosen:** Orchestrator commissions a 54-site call-site sweep alongside the 2-arg deftest API change. Phase E then rewrites deftest to 2-arg form using `run-hermetic`. 54 prelude sites lift their helper defines to file top level.

3. **B3 chosen (partial migration):** Orchestrator accepts that deftest stays on `run-sandboxed-ast` until B1 or B2 resolves; only a new `deftest-hermetic` alias is provided as an ergonomic wrapper over `run-hermetic` for tests that manually use run-hermetic today. Phase F remains blocked for the 54 non-empty prelude sites.

4. **Alternative path:** Keep `deftest`/`deftest-hermetic` on `run-sandboxed-ast` indefinitely. Phase E is out of scope; Phase F retires only those `run-sandboxed-*` callers NOT reached via deftest (i.e., none in `wat-tests/`; only `run-ast` and `run-hermetic-ast` wrappers in `wat/test.wat` are in scope for Phase F).

**Recommended: B1.** It preserves the 3-arg API, requires no call-site changes, and is the conceptually correct fix (macros SHOULD be able to emit multiple top-level forms via a splice wrapper). The substrate change is small and principled. Phase F can then proceed cleanly once B1 ships and Phase E re-runs.
