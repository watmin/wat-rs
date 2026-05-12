# Arc 170 slice 3 phase E V2 — SCORE (deftest macro rewrite via top-level `do`)

**Date:** 2026-05-11
**Branch:** arc-170-program-entry-points
**Status:** BLOCKED — Phase 1 probe failed; `do`-splice gap confirmed; orchestrator direction required

## Scorecard verification

| Row | What | Pass criterion | Result |
|-----|------|----------------|--------|
| A | `:wat::test::deftest` macro body uses top-level `(:wat::core::do ~@prelude define)` + `run-hermetic` | grep — no `run-sandboxed-ast` in deftest expansion | BLOCKED — Phase 1 probe failed; deftest macro NOT rewritten |
| B | `:wat::test::deftest-hermetic` body rewritten (Layer 1 hermetic-by-default) | grep — no `run-sandboxed-hermetic-ast` | BLOCKED — same reason |
| C | Probe demonstrates top-level `do` splicing works for macro emissions | probe test passes | FAIL — probe produced same 3 unresolved refs as V1 `forms` probe (see Phase 1 below) |
| D | Workspace at 0 failed AFTER deftest rewrite | full cargo test | BLOCKED — no rewrite; baseline 2199/0 preserved |
| E | TestResult / RunResult reconciliation correct | grep + cargo test | INHERITED from V1 SCORE — typealias equivalence unchanged; no new finding |
| F | `cargo check --release` green | clean | PASS — workspace unchanged; baseline green |
| G | make-deftest factories cascade documented | SCORE | INHERITED from V1 SCORE — factories cascade transitively through deftest; still accurate |
| H | SCORE includes honest deltas + Phase F readiness check | this file | PASS — ≥ 3 honest deltas below; Phase F readiness count: 4 callers in `wat/test.wat` |

**Row C FAILED (probe). Rows A, B, D blocked. Rows E, G inherited from V1. F passes on unchanged baseline.**

---

## Phase 1 — Probe result (empirical)

### Probe design

A minimal defmacro was written inline as a Rust test `t18_top_level_do_splicing_from_macro_emission`
in `tests/wat_arc170_program_contracts.rs`. The macro emits `(:wat::core::do define-1 define-2)`
from its expansion:

```scheme
(:wat::core::defmacro
  (:my::probe (body :AST<wat::core::nil>) -> :AST<wat::core::nil>)
  `(:wat::core::do
     (:wat::core::define (:my::probe::helper -> :wat::core::i64)
       42)
     ~body))

(:my::probe
  (:wat::core::define (:my::probe::main -> :wat::core::i64)
    (:my::probe::helper)))
```

The probe froze this source against `startup_from_source` and checked whether
`:my::probe::helper` is discoverable in the frozen symbol table.

### Probe result

**Phase 1 FAILED.** Freeze returned identical error to V1's `forms` probe:

```
resolve: 3 unresolved reference(s):
  - :my::probe::helper (call head — not a builtin, not a registered function)
  - :my::probe::main (call head — not a builtin, not a registered function)
  - :my::probe::helper (call head — not a builtin, not a registered function)
```

The probe test was added and removed (net zero change). `tests/wat_arc170_program_contracts.rs`
remains at 1207 lines. Baseline preserved at 2199 passed / 0 failed.

---

## Root cause analysis — do-splice gap

### Why the BRIEF assumed do would work

The V2 BRIEF anchored on:
- `src/check.rs:6848` — `collect_splice_defs_ctx` arm for `:wat::core::do` with `is_top = true`
- Arc 157 § Scope Q1 error message (check.rs:715) — "def is only legal at top-level position:
  (1) direct file top-level, (2) inside a top-level `(:wat::core::do ...)`"

Both references are correct — BUT they document the type-checking pass's behavior for
`:wat::core::def` forms, NOT the function-registration pass's behavior for
`:wat::core::define` forms.

### What actually happens in the freeze pipeline

**Step 4 — `expand_all`** (`src/macros.rs:446`):
The macro fires, producing `(:wat::core::do (:wat::core::define :my::probe::helper ...) (:wat::core::define :my::probe::main ...))` as ONE `WatAST::List` node. This node is pushed to `out` as a single element — no unwrapping.

**Step 6 — `register_defines`** (`src/runtime.rs:1443`):
Iterates forms one-by-one. Checks `is_define_form` (requires the form's head to be `:wat::core::define`). The `do` wrapper's head is `:wat::core::do` — check fails. Checks `try_parse_fn_shape_def` (requires `:wat::core::def` shape) — also fails. The `do`-wrapped form falls to `rest.push(form)`. The nested defines inside the `do` are NEVER processed.

**Step 7 — `resolve_references`** (`src/resolve.rs:84`):
Calls `check_form` recursively. Inside the `do` children, encounters the `(:my::probe::main)` call body which calls `:my::probe::helper`. `is_resolvable_call_head(":my::probe::helper", sym, macros)` → `sym.get(":my::probe::helper")` returns None (never registered) → `UnresolvedReference` emitted.

### The structural gap

`register_defines` does NOT handle `(:wat::core::do ...)` wrappers. The `do` form ends
up in `residue` unchanged; nested `define` forms inside it are invisible to `sym.functions`.
`discover_tests` (test runner) iterates `sym.functions` — tests inside `do` wrappers would
not be discovered even if resolve/check somehow passed.

### Why `do` works for `def` but not `define` at freeze time

The type checker's `collect_splice_defs_ctx` (check.rs:6805) handles `do` for `def` forms
because it is a recursive walker called FROM `check_program`'s sequential loop. It recurses
into `do` children looking for `:wat::core::def` forms.

The function-registration step (`register_defines`) is a flat iterator — it processes each
element of `forms` once, without recursion into wrapper forms. `def` forms have their own
registration path via `register_runtime_defs` (which DOES handle `do` — see runtime.rs:2018).
But `define` forms have no such recursive registration path.

### What substrate change would close the gap

`register_defines` needs a `do`-unwrap arm identical to `register_runtime_defs`'s `do` arm
(runtime.rs:2018-2023):

```rust
// Proposed addition to register_defines (src/runtime.rs ~line 1491):
} else if is_do_form(&form) {
    // Splice: register nested defines; keep non-define children in rest.
    let items = do_children(form);
    let child_rest = register_defines_recursive(items, sym)?;
    // Reconstruct do wrapper with only non-define children remaining:
    if !child_rest.is_empty() {
        rest.push(reconstruct_do_form(child_rest, form.span()));
    }
    // define children already consumed by recursive call above.
```

The symmetric companion change: `register_stdlib_defines` (runtime.rs:1503) would need
the same arm for consistency (stdlib macros that emit `do`-wrapped defines).

Additionally `resolve_references`'s `check_form` (resolve.rs:154) recurses into all
children already — it does NOT need a change. The gap is ONLY in `register_defines`.

---

## Distinct failure modes: forms vs do

V1 used `(:wat::core::forms ...)` — a data-capture form that returns `Vector<WatAST>`.
V2 used `(:wat::core::do ...)` — a sequential-evaluation form.

Both fail with identical errors because `register_defines` treats both as "not a define
and not a def-with-fn" — both fall to `rest` without recursing into their children.
The error messages are indistinguishable at the resolve step (3 unresolved refs, same paths).

The root cause IS different: `forms` was the wrong CONCEPT (data-capture, not splicing).
`do` is the right concept (sequential, top-level-splice-eligible per check.rs) but
`register_defines` has not been extended to recognize it.

---

## Phase F readiness — remaining run-sandboxed-* callers

In `wat/test.wat` (the 4 live callers blocking Phase F):

1. `run-ast` wrapper (line 238) — calls `:wat::kernel::run-sandboxed-ast`
2. `run-hermetic-ast` wrapper (line 258) — calls `:wat::kernel::run-sandboxed-hermetic-ast`
3. `deftest` macro (line 311) — calls `:wat::kernel::run-sandboxed-ast`
4. `deftest-hermetic` macro (line 338) — calls `:wat::kernel::run-sandboxed-hermetic-ast`

In `wat/kernel/hermetic.wat` (line 100): calls `:wat::kernel::run-sandboxed-hermetic-ast<I,O>` —
this is the stdlib implementation; not a deftest caller; Phase F must audit this separately.

**Zero callers in `wat-tests/`.** All `run-sandboxed-*` references in test files are
routed through `deftest`/`deftest-hermetic` macros (items 3 and 4 above).

Phase F readiness count: **4 callers in `wat/test.wat` remain** (same as V1 SCORE).
Phase E is a prerequisite for migrating items 3 and 4. Items 1 and 2 (`run-ast` /
`run-hermetic-ast` wrappers) are independent of Phase E and could be addressed by Phase F
separately IF the substrate verbs are not retired wholesale.

---

## Honest deltas

### Delta 1 — do-splice is not absent from the substrate; it is absent from register_defines specifically

V1 Delta 1 said "Mechanism A is structurally absent." This was framed as a concept-level
gap (no top-level splicer existed). V2 reveals the gap is narrower: `:wat::core::do` IS
the top-level splicer semantically (check.rs:6848, runtime.rs:2018), but `register_defines`
— the function-registration step — was never extended to recurse into `do` wrappers.
The gap is one function, not the entire concept.

### Delta 2 — The check.rs reference in the BRIEF was accurate but layer-misidentified

`src/check.rs:6848` proves `do` splicing exists for the type-check pass's `def` handling.
The BRIEF reasoned this implied `define` registration would also work. The layers are:
- Type checker (`collect_splice_defs_ctx`) — handles `do` for `def`; uses recursive descent
- Function registrar (`register_defines`) — handles only direct `define`; no `do` unwrap
- Runtime def registrar (`register_runtime_defs`) — handles `do` for `def` (runtime.rs:2018)

The V2 framing was right about the concept; wrong about which layer had the gap.

### Delta 3 — The substrate fix is small and well-bounded

V1 proposed B1 as a new special form (`splice-forms`). V2's root-cause analysis shows
the fix is NOT a new form — it is extending `register_defines` to recurse into existing
`do` wrapper forms. The `do` form already handles this pattern in `register_runtime_defs`
(5 lines, runtime.rs:2018-2023). The symmetric fix for `register_defines` is similarly
scoped: ~10 lines + the `register_stdlib_defines` companion.

### Delta 4 — No call-site changes become necessary once the substrate fix ships

Once `register_defines` handles `do` wrappers, the V2 deftest expansion:
```scheme
(:wat::core::do
  ~@prelude
  (:wat::core::define (~name -> :wat::kernel::RunResult)
    (:wat::test::run-hermetic ~body)))
```
will register `~name` in `sym.functions`, `discover_tests` will find it by signature,
and all 223 existing call sites work unchanged. The 54 non-empty prelude sites also
work: their prelude defines are spliced into the `do` alongside the test define —
all land in `sym.functions` at freeze time, visible to the closure extractor.

### Delta 5 — Performance note inherited, now confirmed moot

V1 Delta 4 predicted ~1.1s overhead from 223 tests × ~5ms fork. This remains accurate
for when Phase E ships. Not measured in V2 (Phase E blocked before rewrite).

---

## Files modified

| File | Change |
|------|--------|
| `tests/wat_arc170_program_contracts.rs` | Probe test added and removed (net zero change). Baseline preserved at 1207 lines. |

**No other files modified.** `wat/test.wat` is unchanged. Workspace at 2199 passed / 0 failed.

---

## Orchestrator direction needed

**One substrate change unblocks Phase E V3:**

Extend `register_defines` (and `register_stdlib_defines`) in `src/runtime.rs` to recurse
into `(:wat::core::do ...)` wrapper forms, registering nested `(:wat::core::define ...)` forms
in `sym.functions` and keeping non-define children in `rest`. The pattern mirrors
`register_runtime_defs_form`'s `do` arm (runtime.rs:2018-2023) exactly.

Once that change ships:
- Phase E V3: rewrite deftest + deftest-hermetic to use `(:wat::core::do ~@prelude define)`
  — no call-site changes, all 223 sites work, prelude defines land in parent `sym.functions`
  (visible to closure extractor for `spawn-process`)
- Phase F: retire `run-sandboxed-ast` / `run-sandboxed-hermetic-ast` callers

The substrate change does NOT require a new special form. `:wat::core::do` already has the
correct semantics; only the registration pass needs to recognize it.
