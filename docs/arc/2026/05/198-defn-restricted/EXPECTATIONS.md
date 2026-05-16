# Arc 198 EXPECTATIONS

**BRIEF:** `BRIEF.md`

## Independent prediction

**Runtime band:** 60-90 minutes sonnet.

Reasoning:
- Substrate primitive (parser + AST + eval + CheckEnv storage extension): ~80-120 LOC
- Walker check (new fn pattern; Stone B's `validate_join_result_user_namespace` is the template): ~30-50 LOC
- Defmacro sugar in `wat/core.wat`: ~5-10 LOC
- 5 new tests: ~120-180 LOC
- Stone B already shipped a working walker pattern that arc 198 generalizes; sonnet has clear precedent

**Time-box:** 150 min hard stop.

## SCORE methodology

6 rows YES/NO per BRIEF; per-row evidence patterns:

- **Row A** (def-restricted primitive): grep `def.restricted\|DefRestricted` in src/ shows parser handling + AST variant/extension + eval impl + CheckEnv field.
- **Row B** (walker check + hooked): grep shows new check fn + call site in `check_program` (likely adjacent to Stone B's hook at `src/check.rs:1939`).
- **Row C** (defn-restricted defmacro): grep `defn-restricted` in `wat/core.wat` shows the defmacro definition.
- **Row D** (5 tests pass): `cargo test --release -p wat --test wat_arc198_def_restricted` → all green.
- **Row E** (build clean): cargo build Finished.
- **Row F** (workspace baseline maintained): cargo test summed failed ≤ 4 (Stone B baseline).

## Honest deltas to watch for

- **Parser handling of Vec-of-keyword positional arg.** `def-restricted` takes (name, Vec<Keyword>, value) — three positional args. Sonnet may discover the parser needs slight extension for this shape. Stone B's walker already handles Vec literals; the parser layer is the unknown.

- **AST representation choice.** Two paths: (a) new `WatAST::DefRestricted { name, prefixes, value }` variant, (b) extend `WatAST::Def { name, value, restriction: Option<Vec<String>> }`. Sonnet decides based on how `Def` is currently structured; option (b) is more honest if it doesn't bloat `Def`.

- **CheckEnv storage.** Where do per-binding metadata items live? Likely a sibling field to fn-signatures (the type registry). Sonnet to discover.

- **Defmacro expansion arg-shape.** The user's sketch: `(defn-restricted name [prefixes] sig body)`. The expansion: `(def-restricted name [prefixes] (fn sig body))`. The `~@rest` splice in the existing `defn` defmacro is the precedent. May need a slight reshape if `defn-restricted` has extra args in a different position.

- **Error message format.** Stone B established `JoinResultUserNamespace` as the variant. Arc 198 needs a `DefRestrictedCallerNotAllowed` (or similarly named) variant. The Display impl should be teaching-flavored: name the callee, name the caller, name the whitelist, suggest where the caller can move to (if applicable).

- **Empty whitelist semantics.** `[]` could mean:
  - "No callers allowed" — error always (every call fails)
  - "Default to allow none, must opt in" — same as above
  - "Substrate-only" — sonnet's interpretation if the substrate convention has a default

  Sonnet to decide and document.

- **Restricted-binding lookup at call site.** The walker needs to ask: "is this callee restricted? what's its whitelist?" — that lookup needs to be efficient (HashMap, not linear scan). Verify against existing CheckEnv shape.

## Workspace baseline (commit `2a071f0`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 4 pre-existing target failures (lifeline flake, t6 unquote, totally_bogus, startup_error)

Post-arc-198 target:
- ≥ baseline + 5 passed (5 new tests)
- ≤ 4 failed (no regressions; arc 198 is purely additive)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 60-90 min | TBD |
| Scorecard rows | 6/6 PASS | TBD |
| Workspace fail count | ≤ baseline (4) | TBD |
| New test count | 5 | TBD |
| AST representation chosen | (a) new variant OR (b) extended Def | TBD |
| Empty-whitelist semantics | TBD | TBD |
| Substrate-discovery surprises | 0-2 | TBD |
| Mode | Additive | TBD |
