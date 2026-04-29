# Arc 098 — BACKLOG

DESIGN settled 2026-04-29 (all design questions resolved by Q&A +
Q2/Q12 research). Slice order matches the DESIGN's slice plan.
Each slice is one substrate add; each ships with green tests before
the next opens.

## Slice 1 — Pattern grammar + classifier + type-check side — *ready*

- **Status:** ready.
- **Adds:**
  - New module `src/form_match.rs` with the shared pattern
    classifier (`classify_clause` + companion AST helpers).
  - `infer_form_matches` in `src/check.rs` — full type-check
    pipeline: subject type, struct lookup, clause walk, scope
    handling.
  - Special-form dispatch arm in `infer_call` for
    `:wat::form::matches?`.
  - The runtime-side `eval_form_matches` is stubbed to panic with
    "not yet implemented" — slice 1 is type-check only.
- **Guards (errors at expansion):** unknown struct type, unknown
  field, binding LHS not a `?var`, binding RHS not a `:keyword`,
  unrecognized constraint head, unbound `?var`, `where`-body fails
  to type-check or returns non-`:bool`.
- **Substrate touches:**
  - `src/form_match.rs` — new file (~150 lines: AST classifier +
    error types + ?var detection helper).
  - `src/check.rs` — special-form dispatch arm + infer function.
  - `wat-tests/std/form/matches-typecheck.wat` — error-class smoke
    tests; valid pattern accepted, each invalid pattern rejected.
- **Done when:** `cargo test --workspace` green; type-check side
  accepts the worked example from the DESIGN; rejects each invalid
  pattern category with a diagnostic.

## Slice 2 — Runtime walker — *ready when 1 lands*

- **Status:** ready once slice 1 ships.
- **Adds:**
  - `eval_form_matches` in `src/runtime.rs` — full runtime
    pipeline: subject eval, struct match, clause walk, AND of
    constraints.
  - Special-form dispatch arm in `eval_call`.
  - Reuses slice-1's classifier (no duplicate parsing logic).
- **Guards (false at runtime, no error):** subject is `:None`,
  non-Struct, or wrong struct type → `false`.
- **Substrate touches:**
  - `src/runtime.rs` — eval function + dispatch arm.
  - `wat-tests/std/form/matches.wat` — end-to-end coverage:
    PaperResolved Grace > 5.0 (worked example from arc 093),
    bindings + comparisons, and/or/not, where-escape, struct-of
    mismatch returns false, Option-None returns false, non-Struct
    subject returns false.
- **Done when:** `cargo test --workspace` green; the worked
  example evaluates `true` for matching subjects, `false` for
  every documented mismatch category.

## Slice 3 — INSCRIPTION + USER-GUIDE + arc 093 unblock — *ready when 2 lands*

- **Status:** ready when 2 ships.
- **Adds:**
  - INSCRIPTION.md sealing the arc.
  - USER-GUIDE.md appendix forms-table additions for
    `:wat::form::matches?` and the recognized clause vocabulary.
  - Arc 093's slice-4 example scripts now runnable; mark its
    Clara-matcher dependency resolved.
  - 058 FOUNDATION-CHANGELOG row in lab repo
    (`holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`).

## Cross-cutting fog

- **`?var` lexing** — Q12 research confirmed: wat's lexer accepts
  `?`-prefixed symbols natively. No lexer changes needed. The
  pattern walker recognizes them as logic-variable placeholders
  within the matcher's grammar.
- **Special-form vs defmacro** — Q2 research confirmed: user
  defmacros expand BEFORE type-checking and can't query the struct
  registry at expansion. `:wat::form::matches?` ships as a
  substrate-recognized special form (parallel walkers in check.rs
  and runtime.rs share the slice-1 classifier).
- **Shared classifier discipline** — both check and runtime walk
  the same grammar via `src/form_match.rs::classify_clause`. If
  the classifier's vocabulary needs to grow later (e.g., promoting
  `member` from a `where`-escape to a recognized head), it grows
  in one place and both sides see the change.
- **Bindings extraction (future arc).** A `:wat::form::match`
  variant returning `:Option<:HashMap<:Symbol, :Value>>` is
  out-of-scope for v1 but flagged in the DESIGN's "what this
  enables" list. `where` covers the predicate use case for now.
