# Arc 170 slice 3 Gap F-2 BRIEF — resolver respects `forms`/quote/quasiquote boundaries

**Sonnet.** Third of the four Phase 2a gap slices (after Gap F-1 + F-3 land). Substrate correctness fix for Phase E V4 failure pattern 2 — `resolve_references` walks INTO `:wat::core::forms`-quoted content and treats inner call heads as live code references.

## Backstory

V4 failure pattern 2 (commit `f2de549` SCORE): ambient-stdio.wat uses helper `define` forms whose bodies contain `(:wat::test::run-hermetic-ast (:wat::test::program (:wat::core::define (:user::main -> ...) ...)))`. `(:wat::test::program ...)` expands to `(:wat::core::forms ...)` — a variadic data-capture form whose arguments are AST DATA, not live code.

The resolver currently descends INTO the `forms` arguments and finds `:user::main` as a call head in a nested form. `:user::main` is data here, not a real call head. UnresolvedReference fires anyway. Result: 25+ failures in V4 from one test file.

## The correctness bug

`resolve_references` / `check_form` walks every child of every list form, treating call-head positions as live references. This is correct for normal code but WRONG for quote-family forms:

- **`:wat::core::quote`** — entire argument is data. Don't recurse.
- **`:wat::core::quasiquote`** — template is data EXCEPT inside `:wat::core::unquote` / `:wat::core::unquote-splicing` escapes. Recurse only into unquote children.
- **`:wat::core::forms`** — variadic data-capture; all arguments are data. Don't recurse.

The resolver doesn't currently distinguish these. Path A fix closes the bug correctly across the quote family.

## Goal — resolver becomes quote-aware

Add quote-family arms to the recursive descent in `resolve_references` (or wherever `check_form` lives):

- `(:wat::core::quote x)` — do not recurse into `x`
- `(:wat::core::quasiquote tmpl)` — recurse into `tmpl`, BUT inside `tmpl` only recurse through `:wat::core::unquote` / `:wat::core::unquote-splicing` children (not arbitrary list children)
- `(:wat::core::forms ...)` — do not recurse into any argument

## Subtle case: nested quasiquote

In Clojure-style quasiquote, nested quasiquotes increment a "quote depth" counter, and unquote at depth>1 stays unquoted. wat's current implementation may or may not handle this; **out of scope** — if nested quasiquote isn't currently supported, defer; if it is, the resolver fix should respect existing semantics.

Surface in SCORE: how nested quasiquote is handled today; whether F-2 needs to address it.

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-E-V4-DEFTEST-REWRITE.md`** — V4 failure pattern 2 analysis (lines ~86-103)
2. **`docs/arc/2026/05/170-program-entry-points/RETIREMENT-THEATER-INVENTORY.md`** — priority queue + gap framing
3. **`src/resolve.rs`** — current `check_form` / `resolve_references` implementation; how it descends into list forms
4. **`src/special_forms.rs:225-233`** — quote-family registrations (quote, quasiquote, unquote, unquote-splicing, forms)
5. **`src/macros.rs`** — macro expansion logic; how quote-family forms are processed at expansion time (the resolver runs BEFORE macro expansion in some paths but may run AFTER for some forms)

## Implementation path

### Phase 1 — Audit current resolver behavior

Grep `check_form` / `resolve_references` for any existing quote-handling. Map the current behavior across the quote family. Surface in SCORE.

### Phase 2 — Probes (failing baseline)

Create `tests/probe_resolver_quote_awareness.rs` with:

```rust
#[test]
fn probe_forms_argument_is_data() {
    let src = r#"
        (:wat::core::define
          (:my::helper -> :wat::core::nil)
          (:wat::core::forms
            (:wat::core::define (:user::main -> :wat::core::nil)
              (:wat::core::nil))))
    "#;
    // The inner :user::main is inside forms-quoted data; resolver should NOT
    // fail on it as an unresolved reference.
    startup_from_source(src, ...).expect("freeze");
}

#[test]
fn probe_quote_argument_is_data() {
    // Same shape for :wat::core::quote
}

#[test]
fn probe_quasiquote_with_unquote_recurses_correctly() {
    let src = r#"
        (:wat::core::define
          (:my::actual-helper -> :wat::core::nil) (:wat::core::nil))
        (:wat::core::define
          (:my::macro-result -> :AST<wat::core::nil>)
          `(:wat::core::define (:user::main -> :wat::core::nil)
             (~:my::actual-helper)))
    "#;
    // Inside the quasiquote template, the inner :user::main is data;
    // BUT (~:my::actual-helper) is an unquote — `:my::actual-helper` IS a
    // real call head that must resolve. Verify both: outer freeze succeeds,
    // and :my::actual-helper is required to exist.
    startup_from_source(src, ...).expect("freeze");
}
```

(Sonnet adapts probe shape to actual resolver test conventions.)

### Phase 3 — Extend resolver

Add quote-family arms to `check_form` / recursive descent. Order:

1. Check if form's head is `:wat::core::forms` → don't recurse into children
2. Check if form's head is `:wat::core::quote` → don't recurse into argument
3. Check if form's head is `:wat::core::quasiquote` → recurse with depth-aware logic (only descend into unquote/unquote-splicing children)

### Phase 4 — Verify

- 3+ probes pass (forms, quote, quasiquote+unquote)
- All existing probes pass (no regression)
- Workspace stays at 0 failed
- Existing quote-using code in `wat/` and `wat-tests/` continues to work

## Scope (what's IN)

- Resolver quote-family arms (forms / quote / quasiquote)
- Quasiquote handles unquote escapes correctly
- 3+ new probes
- Workspace stays at 0 failed

## Scope (what's OUT)

- Nested quasiquote handling (defer if not currently supported; if currently supported, preserve existing semantics)
- Other resolver concerns (out-of-scope fixes)
- Gap F-1 / F-3 / G — separate slices
- Anything under `docs/arc/` (FM 11)
- `~/.claude/` memory system
- New quote-family forms

## Ship criteria (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | Resolver has `:wat::core::forms` arm (don't recurse) | grep + read |
| B | Resolver has `:wat::core::quote` arm (don't recurse) | grep + read |
| C | Resolver has `:wat::core::quasiquote` arm with unquote descent | grep + read |
| D | 3+ probes pass | cargo test |
| E | No regression in existing tests; workspace at 2209 + N + N' + 3 / 0 failed | full test |
| F | Existing quote-using code unchanged behavior | full test |

**6 rows.** All must PASS.

## Predicted runtime

**45-90 min sonnet.** More design-heavy than F-1/F-3 because of quasiquote-unquote semantics; otherwise mechanical resolver extension.

**Hard cap:** 180 min (2×).

## Constraints (hard)

- DO NOT modify quote-family form REGISTRATION (special_forms.rs entries unchanged)
- DO NOT modify macro EXPANSION logic (separate concern; resolver runs at different time)
- DO NOT add new quote-family forms
- DO NOT touch `docs/arc/` (FM 11)
- DO NOT commit (orchestrator atomic-commits)
- DO NOT use deferral language in SCORE
- DO NOT operate outside `/home/watmin/work/holon/wat-rs/`
- DO NOT touch `~/.claude/` memory system
- DO NOT extend to Gap F-1 / F-3 / G scope
- DO NOT use --no-verify or skip hooks
- DO NOT break existing quote-using code (quote/quasiquote/unquote semantics must be preserved)

## Honest delta categories (anticipated)

1. **Nested quasiquote handling** — what's the current behavior; does F-2 need to address it
2. **Unquote-splicing edge cases** — `~@list` semantics inside quasiquote
3. **Other resolver call sites** — is `check_form` the only place that walks list children, or are there sibling places that need the same arms?
4. **Existing quote-using code impact** — anything in wat/ that relied on the OLD (incorrect) resolver-walks-into-quote behavior?
5. **Anything unexpected** — particularly resolver-vs-macro-expansion ordering surprises

## Cross-references

- V4 SCORE (failure pattern 2): `SCORE-SLICE-3-PHASE-E-V4-DEFTEST-REWRITE.md`
- arc 172 (Scheme → Clojure macro flavor swap) — quasiquote syntax pivot history
- arc 173 (Clojure macro feature parity) — auto-gensym, &form/&env (separate concern)
- Gap F-1 (predecessor slice; struct/enum pregen)
- Gap F-3 (predecessor slice; closure type-registry inheritance)
- Gap G (next slice; Path E macro shape)
- Phase E V5 (unblocked after all 4 gaps)
