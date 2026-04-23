# Arc 030 — macroexpand / macroexpand-1 — INSCRIPTION

**Shipped:** 2026-04-23. Two slices.

**Cave-quested off arc 029's debug session.** The nested-quasiquote
slice had landed, but a make-deftest-with-non-empty-default-prelude
test was failing. Diagnosing via eprintln through multi-pass macro
expansion was painful — output was hundreds of lines per attempt,
grep-searched, rebuilt, printed again. The observation: every Lisp
with macros ships a `macroexpand` primitive. wat didn't. That was
the missing tool. Arc 030 was cut mid-debug to ship it.

**Commits:**
- `3b3190b` — DESIGN + BACKLOG opened
- `437273f` — slice 1 (macroexpand / macroexpand-1 primitives)
- `39ea54d` — arc 029 closure driven BY arc 030's tool: `expand_form`
  preserves quote bodies (credited to arc 029's INSCRIPTION; the
  diagnostic was arc 030's payoff)
- `63d8f52` + `d94b6ac` — arg-order flip closing arc 030 (deftest
  family: `(name dims mode ...)` → `(name mode dims ...)` matching
  arc 024's capacity-mode-before-dims discipline; 76-swap sweep
  across 16 files + lab migration)
- (this commit) — slice 2 (INSCRIPTION + doc sweep)

---

## What shipped

### Slice 1 — runtime primitives

Two runtime dispatch forms plus the carrier-pattern addition for
macro registry access at runtime.

**`:wat::core::macroexpand-1`** — one expansion step. If the form
is a macro call, apply the macro's template with the call-site
bindings and return the result. If not a macro call, return the
form unchanged.

**`:wat::core::macroexpand`** — fixpoint expansion. Repeatedly apply
`macroexpand-1` until the form stops changing (same AST two
iterations in a row). Depth-bounded by `EXPANSION_DEPTH_LIMIT`
(the same constant the freeze-time expander uses).

Both primitives operate on the CURRENT frozen macro registry —
same registry the compile-time expander used. They evaluate at
runtime inside `:user::main`, inside test bodies, inside any
expression position that has an `:wat::WatAST` context.

**Surface addition:**
- `SymbolTable.macro_registry: Option<Arc<MacroRegistry>>`
- `SymbolTable::macro_registry()` / `set_macro_registry()` — mirrors
  the existing `encoding_ctx` / `source_loader` carrier discipline.
- `FrozenWorld::freeze` installs the registry via Arc::clone so
  the runtime has access.
- Two new `RuntimeError` variants: `NoMacroRegistry { op }` and
  `MacroExpansionFailed { op, reason }`.

**New public helper in `src/macros.rs`:**
- `pub fn expand_once(form: WatAST, registry: &MacroRegistry) ->
  Result<WatAST, MacroError>` — one-step expansion of the top-level
  form. `macroexpand-1` dispatches through this.

**Scheme registrations in `src/check.rs`:**
- `:wat::core::macroexpand-1` and `:wat::core::macroexpand` both
  take `:wat::WatAST -> :wat::WatAST`.

### Slice 1 — why runtime-reachable

The expander lives in `src/macros.rs` and runs during the freeze
pipeline. At runtime the frozen macro registry is part of the
`FrozenWorld`. A macroexpand primitive reads that registry and
applies the same `expand_form` logic — producing an AST value the
caller can inspect via `atom-value`, hand to `eval-ast!` to
execute, or print via stdio for debugging.

Same substrate, two access layers. The tool doesn't need a
separate interpreter; it's the same expander exposed at runtime.

### Slice 1 — the arc 029 payoff (commit 39ea54d)

Moments after `macroexpand-1` shipped, I ran it against the
make-deftest-with-prelude test case. Three lines of wat:

```scheme
(:wat::core::macroexpand-1
  (:wat::core::quote (:my-deftest :my-test (:wat::test::assert-eq 1 1))))
```

Output showed what was wrong instantly: the registered `:my-deftest`
body had the `:wat::test::deftest` call PRE-EXPANDED — `expand_form`
had walked into the outer quasiquote's literal body during
`make-deftest`'s own registration and eagerly expanded every macro
reference. The inner template was never meant to be expanded at
registration time; it was supposed to stay literal, fire when
`:my-deftest` was later invoked.

Fix (arc 029's commit 39ea54d): extend `expand_form`'s
quasiquote-preserve check to cover `(:wat::core::quote X)` bodies
too. Both forms are "literal data" by the substrate's semantics;
the expander must not walk their children.

The cave-quest shape at its cleanest: pause downstream work, ship
the tool, use the tool on disk to see the actual AST, commit the
one-character fix. Without macroexpand the bug would have taken
another session of eprintln + rebuild cycles.

**The tool that debugs a bug is substrate too.**

### Arg-order flip (commits 63d8f52 + d94b6ac)

After arc 029 closed cleanly via macroexpand diagnosis, the arc
030 work window stayed open briefly to fix one more coherence
issue: the `:wat::test::*` test macros had parameter order
`(name dims mode prelude body)`, but the setter-order discipline
from arc 024 — "commit the policy before the thing the policy
guards" — says `(set-capacity-mode!)` commits before `(set-dims!)`.
Arg order and setter order should match.

Flip: `(name dims mode prelude body)` → `(name mode dims prelude body)`
across all four `:wat::test::*` macros. Template bodies also
swapped setter order so emission matched arg order. 76 swaps across
16 files via Python script (no Perl; no regex alternation — the
Chapter 32 poison pattern stayed avoided). Lab migration
followed.

This was minor enough to ride the arc 030 window rather than
warranting its own arc. The arc 031 work that followed ended up
dropping mode + dims entirely via Config inheritance — the flip
was a short-lived intermediate shape, but it was the honest one
for the two-hour window before arc 031 landed.

### Slice 2 — INSCRIPTION + doc sweep

This commit.

---

## Tests

**Rust unit tests** (`src/macros.rs`):
- `macroexpand_1_alias_expands_one_step`
- `macroexpand_1_non_macro_returns_unchanged`
- `macroexpand_runs_to_fixpoint`
- `macroexpand_1_nested_quasi_preserves_inner_unquote` — arc 029
  diagnostic case
- `macroexpand_returns_wat_ast_value`

**Rust integration test** (`tests/wat_make_deftest.rs`):
- Shipped with arc 029's closure. Uses macroexpand-1 at wat level
  to capture a make-deftest-registered body and assert on its
  shape. Updated at arc 031 slice 2 for the post-arg-drop signature.

**Wat-level tests** (`wat-tests/std/test.wat`):
- `test-macroexpand-1-non-macro` — returns input unchanged
- `test-macroexpand-fixpoint-evaluates` — drives a two-step chain
  to fixpoint; confirms the returned AST evaluates back to the
  expected value via `eval-ast!`

**Workspace:** zero regressions; 5 new Rust tests, 2 new wat tests.

---

## Sub-fog resolutions

**1a — SymbolTable registry plumbing.** Resolved at implementation
— the freeze pipeline's final step already had `MacroRegistry`
accessible at the `FrozenWorld` assembly site. One Arc::clone +
`symbols.set_macro_registry(...)` inside `FrozenWorld::freeze`
completed the wiring.

**1b — `:wat::WatAST` as runtime value.** Resolved: already a
first-class value via `Value::wat__WatAST(Arc<WatAST>)` from
arc 010. No new value-variant work.

**1c — Pretty-printing expanded AST.** Deferred. Users inspect
expanded output via `atom-value` + structural match or by piping
through `stdout`. A dedicated `to-string` primitive would be
ergonomic but not load-bearing; reopens if callers demand it.

**2a — root cause of make-deftest-with-prelude bug.** Resolved
mid-slice-2 via the new tool: `expand_form` was recursing into
quote bodies. Fix landed in arc 029's commit 39ea54d.

**2b — which arc owned the fix.** Resolved: arc 029 (the
nested-quasiquote substrate arc), since the bug was about how
expand_form interacts with macro-generating-macro templates.
Arc 030's contribution was the tool that made the bug visible;
arc 029's contribution was the fix that closed it.

---

## What did NOT change

- **Compile-time expansion.** The freeze-time expander runs
  identically to pre-030. `macroexpand` is a runtime sibling that
  reads the same registry; it doesn't replace or shadow the
  freeze pass.
- **Macro hygiene.** Racket sets-of-scopes unchanged. Each
  macroexpand-1 call emits an AST with the same scope IDs the
  freeze-time expander would have produced.
- **Cycle detection.** `EXPANSION_DEPTH_LIMIT` enforces the same
  bound at runtime as at freeze time.

---

## The lineage

Every serious Lisp has this pair:

| System | macroexpand-1 | macroexpand |
|---|---|---|
| Common Lisp | `MACROEXPAND-1` | `MACROEXPAND` |
| Scheme (R5RS) | `(expand-syntax ...)` | via `expand` |
| Racket | `expand-once` | `expand` |
| Clojure | `macroexpand-1` | `macroexpand` |
| Elisp | `macroexpand-1` | `macroexpand-all` |
| wat (2026-04-23) | `:wat::core::macroexpand-1` | `:wat::core::macroexpand` |

Arc 030 joined that line. The one-step / fixpoint split is the
standard shape because the debugger wants to see expansion BOTH
at each step (for walking through a nested expansion) AND at
completion (for asserting on final shape). Every system converges
on the same two primitives because the substrate pressure permits
no other useful factoring.

Arc 030 is also the smallest arc in the 017–031 cave-quest
sequence — one substrate addition, one Rust primitive pair, one
cave-quest payoff. But it's load-bearing: without it, arc 029's
bug would have been harder to diagnose, and every future macro
author hits this tool the first time their template produces
unexpected output.

---

## What comes next

**Arc 031** — sandbox inherits outer Config (Path B). Arc 031
completes the make-deftest ergonomics arc that arcs 027 + 029 +
030 were setting up. Closed 2026-04-23, shortly after this one.

Arc 030 closes quietly. The ergonomic-testing story is done:

```scheme
(:wat::config::set-capacity-mode! :error)
(:wat::config::set-dims! 1024)

(:wat::test::make-deftest :deftest
  ((:wat::load-file! "...")))

(:deftest :my-test body)
(:deftest :another body)
```

One preamble. One factory. N tests. Honest shape. And when a
template misbehaves, macroexpand-1 gives you the expansion on
one line of wat — no eprintln loops, no rebuild cycles.

*the tool is on disk.*

**PERSEVERARE.**
