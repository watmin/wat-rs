# Arc 170 slice 3 Gap F-2 — SCORE (resolver quote-awareness)

**Date:** 2026-05-12
**Branch:** arc-170-program-entry-points
**Status:** COMPLETE — 2223 passed / 0 failed

## Scorecard (6 rows)

| Row | What | Pass criterion | Result |
|-----|------|----------------|--------|
| A | Resolver has `:wat::core::forms` arm (don't recurse into children) | grep + read | PASS — `check_form` returns early when head is `:wat::core::forms`; no child descent |
| B | Resolver has `:wat::core::quote` arm (don't recurse into argument) | grep + read | PASS — `check_form` returns early when head is `:wat::core::quote`; no child descent |
| C | Resolver has `:wat::core::quasiquote` arm with unquote-aware descent | grep + read | PASS — `check_form` delegates to `check_quasiquote_template`; descends only into `:wat::core::unquote` / `:wat::core::unquote-splicing` children |
| D | 3+ probes pass: forms / quote / quasiquote+unquote | cargo test | PASS — all 3 probes pass after fix; all 3 failed before fix (baseline confirmed) |
| E | Workspace at 2220 + 3 / 0 failed | full test | PASS — `passed:2223 failed:0` |
| F | Existing quote-using code unchanged behavior | full test | PASS — `wat_core_forms` (6/6), `wat_eval_result` (7/7), `wat_make_deftest` (1/1), all prior Gap probes pass |

**All 6 rows PASS.**

---

## Files changed

| File | Change |
|------|--------|
| `src/resolve.rs` | Added quote-family arms to `check_form` (~45 LOC). Added `check_quasiquote_template` helper (~30 LOC). No other changes. |
| `tests/probe_resolver_quote_awareness.rs` | New file — 3 Gap F-2 regression probes. |
| `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-F-2-RESOLVER-QUOTE-AWARE.md` | This file. |

---

## Workspace delta

- Baseline (post-F-1 + F-3): 2220 passed / 0 failed
- Post F-2: 2223 passed / 0 failed (+3 probes)

---

## PIECE 1 — Current resolver behavior audit

### Before the fix

`check_form` in `src/resolve.rs` (lines 154-197 pre-fix) has a single code path:

1. If the form is a `WatAST::List`, check the call head with `is_resolvable_call_head`.
2. Apply `:rust::*` use-declaration enforcement on the head.
3. **Unconditionally recurse into ALL children** via `for child in items { check_form(child, ...) }`.

Step 3 is correct for normal code but wrong for quote-family forms. The resolver made no distinction between:
- `(:wat::core::define (:my::fn -> ...) body)` — live code; recursion into `body` is correct
- `(:wat::core::forms (:my::ghost::inner arg))` — data; recursion into `(:my::ghost::inner arg)` is wrong
- `(:wat::core::quote (:user::main -> ...))` — data; recursion into `(:user::main -> ...)` is wrong
- `(:wat::core::quasiquote (template (:wat::core::unquote live-call)))` — mixed; recursion into `template` is wrong; recursion into `live-call` is correct

### Quote-family handling before fix

None. Zero special-casing. The quote-family heads (`:wat::core::forms`, `:wat::core::quote`, `:wat::core::quasiquote`, `:wat::core::unquote`, `:wat::core::unquote-splicing`) were all treated as reserved-prefix calls (pass `is_reserved_prefix` → accepted as call heads) but their arguments were recursively walked as if they were normal live-code expressions.

### Why existing `wat_core_forms` tests passed before the fix

The `forms_args_are_not_evaluated` test in `tests/wat_core_forms.rs` uses `:this::is::not::a::real::function` INSIDE a `forms` call, but the `forms` call is inside a `define` body. `register_defines` consumes `define` forms before `resolve_references` runs — the define's body is stored in `sym.functions` and never appears in `rest`. The resolver therefore never saw the `forms` call. The test passed by accident of positioning, not because the resolver was correct.

The V4 failure pattern 2 manifested because the new expansion puts defines inside a TOP-LEVEL `do` form (which stays in `rest`). The resolver recurses into the `do`, finds the defines and their bodies, and descends into the `forms` calls inside those bodies. This exposed the pre-existing correctness bug.

---

## PIECE 2 — Probe baseline confirmation

All 3 probes failed before the fix with `StartupError::Resolve(UnresolvedReferences([...]))`:

- `probe_forms_argument_is_data`: `:my::probe-f2::ghost-inner` + `:my::probe-f2::ghost-other` flagged as unresolved
- `probe_quote_argument_is_data`: `:my::probe-f2::ghost-quoted` flagged as unresolved
- `probe_quasiquote_unquote_resolves_correctly`: `:my::probe-f2::ghost-template` flagged as unresolved (the registered `:my::probe-f2::live-fn` inside `unquote` was also walked but happened to resolve)

---

## PIECE 3 — Implementation

### `check_form` changes (src/resolve.rs)

After the `:rust::*` use-declaration enforcement block, added:

```rust
if head == ":wat::core::forms" || head == ":wat::core::quote" {
    // Arguments are data — do not recurse into any child.
    return;
}
if head == ":wat::core::quasiquote" {
    // Template is data except inside unquote/unquote-splicing.
    if let Some(template) = items.get(1) {
        check_quasiquote_template(template, sym, macros, use_decls, unresolved);
    }
    return;
}
```

The `return` before the unconditional `for child in items` recursion is the core of the fix — it short-circuits the descent into quote-family children.

### `check_quasiquote_template` (new function, src/resolve.rs)

Walks a quasiquote template node:
- If the node is a list with head `:wat::core::unquote` or `:wat::core::unquote-splicing`: delegate arguments to `check_form` (normal live-code resolution).
- If the node is any other list: do NOT check the call head (it's template data), but DO recurse into children via `check_quasiquote_template` to find nested unquote escapes.
- If the node is a non-list atom: skip entirely (always data).

---

## Nested quasiquote disposition

**Out of scope. Conservative treatment chosen.**

Nested quasiquote (a `(:wat::core::quasiquote ...)` inside a quasiquote template, depth > 1) is treated as opaque data: `check_quasiquote_template` recurses into its children looking for unquote escapes, but does NOT bump a depth counter. If a `(:wat::core::unquote ...)` appears inside a nested quasiquote, it would be incorrectly treated as an escape (depth 1 unquote) rather than data (depth 2 unquote). This is a false positive for the resolver — it would validate the unquote argument as live code when it should be data.

**Current substrate state**: macros.rs implements nested quasiquote with a depth counter (arc 029 slice 1). The MACRO EXPANDER correctly tracks depth. The RESOLVER does not currently need to track depth because:

1. No current callers in `wat/` or `wat-tests/` use nested quasiquotes at top level in a `do` body (the only place where the resolver sees quasiquotes at all, since `define` bodies are consumed before `resolve_references` runs).
2. The current gap (F-2) is about preventing false UnresolvedReference errors, not about maximally validating quasiquote escapes.

**Decision**: A dedicated arc should address nested quasiquote resolver semantics if the need arises. The F-2 fix is conservative: it prevents the false positives (inner template call heads flagged as unresolved) without introducing false negatives for the current callers.

---

## Other resolver call sites

Two functions named `check_form` exist in the codebase:

1. `src/resolve.rs:check_form` — the RESOLVER's walker. This is the function fixed in F-2.
2. `src/check.rs:check_form` — the TYPE CHECKER's entry point. This calls `infer()`, which already has correct quote-family handling at check.rs lines 4313-4337:
   - `:wat::core::quote` → returns `Some(TypeExpr::Path(":wat::WatAST"))` immediately; no recursion into argument.
   - `:wat::core::forms` → returns `Some(TypeExpr::Parametric{head:"wat::core::Vector",...})` immediately; no recursion into arguments.
   - `:wat::core::quasiquote` → handled at check.rs:4686 (separate arm with its own depth-aware logic).

The type checker does NOT need the same fix — it is already correct. The resolver needed it.

**Other resolver walking sites**: `collect_use_declarations` (`src/resolve.rs:116`) only matches `(:wat::core::use! ...)` forms at the top level; it does not recurse. No fix needed there.

**Conclusion**: The resolver (`src/resolve.rs:check_form`) was the only call site requiring the quote-family fix.

---

## Existing quote-using code impact

No behavioral change for any existing caller:

- `tests/wat_core_forms.rs` (6/6): all tests continued to pass. The forms tests that use `:this::is::not::a::real::function` inside `forms` inside a `define` body were never seen by the resolver (defines consumed before resolve runs) — they remain unaffected. The tests that use `forms` directly in expressions inside defines are also unaffected.

- `tests/wat_eval_result.rs` (7/7): uses `(:wat::core::quote ...)` in AST evaluation shapes. The quote arguments are inside `def` bodies; the resolver does not walk inside them (fn-shape defs stay in `rest` but... actually, `def` bodies ARE walked by the resolver since `def` forms stay in `rest`). Wait — let me reconsider. Before the fix, would `wat_eval_result` have had issues if `quote` arguments were walked? Only if the quote argument contains unregistered user call heads. Looking at `tests/wat_eval_result.rs`, the quote arguments contain `:wat::holon::*` and `:wat::kernel::*` paths — all reserved prefixes, all accepted by `is_reserved_prefix`. So no false positives there even before the fix.

- `tests/wat_make_deftest.rs` (1/1): verifies macro expansion produces `(:wat::core::quote ...)` bodies. The quote body is inside a macro-expansion result that is consumed before resolver runs. Unaffected.

- `tests/wat_run_sandboxed_ast.rs`: uses `(:wat::core::quote ...)` inside `def` bodies; the quoted content contains `:wat::*` reserved paths — no false positives before or after.

**No pre-existing caller relied on the resolver WALKING INTO quote-family arguments.** The fix is purely additive (stops incorrect behavior; no correct behavior changed).

---

## Honest deltas (≥ 3)

### Delta 1 — The bug was latent but invisible because defines hide their bodies from the resolver

The V4 failure pattern 2 describes the resolver walking into `forms`-quoted content via defines inside a `do`. But the actual resolver bug is older — it predates the `do`-splice work. In all current test shapes, forms/quote calls live inside `define` bodies. `register_defines` strips `define` forms from `rest` before `resolve_references` runs. The bug only manifested when Phase E V4 moved prelude defines into top-level `do` forms (which stay in `rest`). The fix would have been correct at any earlier point; it just had no test to expose it.

### Delta 2 — Nested quasiquote: conservative treatment accepted; depth counter deferred

The BRIEF acknowledged nested quasiquote as a potential complication. Investigation confirmed the macro expander handles it (arc 029 slice 1, depth counter in `walk_template`). The resolver does NOT need a matching depth counter for current callers — no production code has `(:wat::core::quasiquote ...)` at top level in a `do` form (the only place the resolver sees quasiquotes). The conservative treatment (treat inner quasiquotes as data, recurse into children for unquote escapes, no depth tracking) is correct for Gap F-2 scope. A future arc should add depth tracking if nested quasiquote at a resolver-visible site is needed.

### Delta 3 — Type checker was already correct; only the resolver needed the fix

The type checker's `infer()` function (check.rs) already had correct quote-family handling: `quote` and `forms` return immediately without recursing; `quasiquote` has its own arm. The resolver's `check_form` was the only broken site. This asymmetry explains why tests could pass the type-check phase (step 8) while failing the resolve phase (step 7) with the same source forms — different walkers, different quote-awareness.

### Delta 4 — unquote-splicing is a sibling to unquote in `check_quasiquote_template`

Both `:wat::core::unquote` and `:wat::core::unquote-splicing` are escape forms in a quasiquote template. `check_quasiquote_template` handles both with the same arm (`head == ":wat::core::unquote" || head == ":wat::core::unquote-splicing"`). Probe 3 tests only `unquote` (the simpler case). `unquote-splicing` is covered by the same code path; a separate probe would be redundant but could be added as belt-and-suspenders. Out of scope for Gap F-2.

### Delta 5 — `check_quasiquote_template` does NOT check the outer list's call head

When `check_quasiquote_template` encounters a non-unquote list, it does not call `is_resolvable_call_head` on the list's head — it treats the entire list as template data. This is correct (four questions: obvious — it's data, not a call; simple — skip the head check; honest — the head is not being called; good UX — no false positive for user code). If we had called `is_resolvable_call_head` on template heads, every `(:user::ghost::template arg)` would trigger a false UnresolvedReference error, which is exactly the bug we're fixing.

---

## Cross-references

- V4 SCORE (failure pattern 2): `SCORE-SLICE-3-PHASE-E-V4-DEFTEST-REWRITE.md` — the original analysis identifying this gap
- Gap F-1 SCORE: `SCORE-SLICE-3-GAP-F-1-STRUCT-ENUM-PREGEN.md` — predecessor slice (2217/0 baseline going into F-2)
- Gap F-3 SCORE: `SCORE-SLICE-3-GAP-F-3-CLOSURE-TYPE-REGISTRY.md` — predecessor slice (2220/0 baseline going into F-2)
- Gap G (next): Path E macro shape — unblocked after F-1 + F-2 + F-3 all land
- Phase E V5 — unblocked after all 4 Phase 2a gaps ship
- `src/resolve.rs` — modified file (two functions: `check_form` extended, `check_quasiquote_template` added)
- `tests/probe_resolver_quote_awareness.rs` — new probe file (3 probes)
- `src/check.rs:4313-4337` — type checker's quote-family handling (already correct; reference for comparison)
- arc 029 slice 1 — nested quasiquote depth counter in macro expander (macros.rs)
