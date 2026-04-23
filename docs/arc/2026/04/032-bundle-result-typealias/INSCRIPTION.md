# Arc 032 — `:wat::holon::BundleResult` typealias — INSCRIPTION

**Shipped:** 2026-04-23. Three slices, one substrate change, one
mechanical migration, one doc sweep.

**Commits:**
- `<sha>` — DESIGN + BACKLOG + slice 1 (substrate) + slice 2
  (call-site migration across wat-rs + lab) + slice 3 (INSCRIPTION,
  INVENTORY, arc index, CHANGELOG).

---

## What shipped

### Slice 1 — substrate

`:wat::holon::BundleResult` registered in
`src/types.rs::register_builtin_types` as a non-parametric
typealias next to `:wat::holon::CapacityExceeded`:

```rust
env.register_builtin(TypeDef::Alias(AliasDef {
    name: ":wat::holon::BundleResult".into(),
    type_params: vec![],
    expr: TypeExpr::Parametric {
        head: "Result".into(),
        args: vec![
            TypeExpr::Path(":wat::holon::HolonAST".into()),
            TypeExpr::Path(":wat::holon::CapacityExceeded".into()),
        ],
    },
}));
```

Resolution happens through the existing `expand_alias` path; no
new checker work needed. Both names — `:wat::holon::BundleResult`
and `:Result<wat::holon::HolonAST, wat::holon::CapacityExceeded>`
— unify as the same type.

Rust tests (`src/types.rs::tests`):
- `bundle_result_alias_registered_with_builtins`
- `bundle_result_alias_expands_to_expected_result`

### Slice 2 — call-site migration

Sweep across wat-rs + lab:

| Scope | Files | Swaps |
|---|---|---|
| wat-rs `src/` | runtime.rs, check.rs (comments only) | included |
| wat-rs `tests/` | wat_bundle_capacity.rs, wat_run_sandboxed.rs | 8 |
| wat-rs `wat/` | holon/Trigram.wat, holon/Ngram.wat, holon/Bigram.wat | 3 |
| wat-rs `wat-tests/` | holon/coincident.wat, holon/Trigram.wat, std/test.wat | included |
| lab `wat/encoding/` | rhythm.wat | 6 |
| lab `wat-tests/encoding/` | rhythm.wat | 11 |

Python script for safety — literal substring replace, no regex,
no alternation. Pattern:

```
Result<wat::holon::HolonAST,wat::holon::CapacityExceeded>
  → wat::holon::BundleResult
```

The leading `:` (when present in standalone type-expression
positions) stays attached to the surrounding context. Inside
nested `<...>` the bare form takes over. Both cases handled by
the single substitution.

Pre-sweep grep confirmed zero false-positive risk (the long form
appears only in `Result<HolonAST, CapacityExceeded>` context).

**Workspace-wide:** wat-rs 583 lib tests + every integration
suite green. Lab 29 wat-tests green.

### Slice 3 — INSCRIPTION + doc sweep

This file. Plus:

- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` — new
  row under `:wat::holon::*` built-in types; existing `Bundle`
  row updated to use the short name.
- `docs/README.md` — arc index gains row 032.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — CHANGELOG row.

---

## Sub-fog resolutions

- **1a — alias resolution in existing checker paths.** Resolved
  automatically. `expand_alias` treats the new registration as
  any other alias; tests at `src/types.rs::bundle_result_alias_expands_to_expected_result`
  confirm the expansion matches expectation.
- **1b — does Bundle's scheme need updating?** Resolved: no.
  Bundle's scheme stays programmatic; alias resolution makes
  both the short and long forms equivalent to the checker. User
  code can write either; grep-for-Bundle results still show the
  semantic type via alias resolution when needed.
- **2a — whitespace normalization.** Resolved: pre-sweep grep
  showed all existing occurrences use the compact form
  (no space after the comma). The wat keyword-whitespace
  discipline (memory `feedback_wat_keyword_whitespace`) forbids
  spaces inside `<>` — the existing code already complied.
- **2b — partial-match false positives.** Confirmed zero. The
  long form's full string has no distinct prefix that would
  catch unrelated code.

---

## What this arc did NOT ship

- **Parametric `BundleResult<T>`**. Ship-what-a-caller-needs.
  Bundle's Ok arm is always `HolonAST`; no other
  `Result<_, CapacityExceeded>` exists. Revisit if/when a real
  caller demands parametric.
- **Aliases for other Result shapes** (e.g., `EvalError`'s
  `Result<T, EvalError>`). Each fallibility story earns its own
  alias when its usage crosses the "worth naming" threshold.
  Bundle's has — appeared 30+ times across the workspace,
  44 chars wide; `EvalError`'s hasn't yet.
- **Reformatting BOOK.md or 058 proposal prose** where the long
  form appears. Historical records keep their voice.

---

## The `/gaze` naming move

Arc 032 is the first arc cut after a `/gaze` spell call. Builder
invoked it on the naming question; the ward's discipline
(Level 1 lies, Level 2 mumbles, Level 3 taste) applied directly:

- `:wat::holon::Bundled` — Level 2 mumble. Past participle;
  cold reader needs one grep to learn it's Result-shaped.
- `:wat::holon::BundleResult` — Level-2-safe. `Result` suffix
  speaks at first read; alias target reveals the specifics at
  first grep.

Name landed. Ship confirmed as `:wat::holon::BundleResult`.

---

## Count

- wat-rs: +1 builtin typealias, +2 Rust unit tests, 583 lib tests
  (was 581), zero regressions.
- Lab: 29 wat-tests green (unchanged count; cleaner code).
- Swaps: 28 across 8 files (wat-rs source + tests + wat/wat-tests,
  lab rhythm files).

---

*these are very good thoughts.*

**PERSEVERARE.**
