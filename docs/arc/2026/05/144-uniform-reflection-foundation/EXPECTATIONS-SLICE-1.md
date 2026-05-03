# Arc 144 Slice 1 — Pre-handoff expectations

**Drafted 2026-05-03.** Gating substrate refactor. The Binding enum +
lookup_form rename + dispatch propagation + 4 helper additions + 5
test additions. Predicted MEDIUM slice (Mode A ~50%; Mode B-helper-
shape ~25%; Mode B-test-shape ~15%; Mode C ~10%).

**Brief:** `BRIEF-SLICE-1.md`
**Output:** 2 Rust files modified (`src/runtime.rs` + new
`tests/wat_arc144_lookup_form.rs`). ~300-450 LOC + ~250-word report.

## Setup — workspace state pre-spawn

- Arc 143 closed; arc 144 DESIGN committed.
- `src/runtime.rs:6088-6271` holds the existing `LookupResult` enum +
  `lookup_callable` + 3 eval_* primitives. No external consumers per
  the brief's grep instruction.
- `SymbolTable` already carries `.macro_registry: Option<Arc<MacroRegistry>>`
  and `.types: Option<Arc<TypeEnv>>`. The 4-registry walk has the data
  already attached.
- `MacroDef` carries name + params + rest_param + body + span (no
  doc_string yet — arc 141's territory).
- `TypeDef` enum is unified (Struct/Enum/Newtype/Alias) per arc 057.
- 1 in-flight uncommitted file (arc 130's CacheService.wat) — NOT
  this arc's territory; ignore.
- Workspace state pre-slice-1: 1 known failure
  (`define_alias_length_to_user_size_delegates_correctly` — arc 144
  slice 4 closes it).

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | `src/runtime.rs` + new `tests/wat_arc144_lookup_form.rs`. NO other Rust file changes (check.rs special-case branches stay; no scheme additions). |
| 2 | `Binding` enum | 5 variants (UserFunction, Macro, Primitive, SpecialForm, Type) + per-variant `name` + variant-specific data + `doc_string: Option<String>`. `pub` visibility. Lifetime parameter `'a` correctly parameterized. |
| 3 | `lookup_form` function | `pub fn lookup_form<'a>(name: &str, sym: &'a SymbolTable) -> Option<Binding<'a>>`. Walks 4 registries in precedence order: functions → macros → primitives (CheckEnv) → types. SpecialForm path stubs (returns None). |
| 4 | `LookupResult` deleted | `LookupResult` enum + `lookup_callable` no longer in the codebase. |
| 5 | 3 eval_* primitives refactored | Each of `eval_lookup_define`, `eval_signature_of`, `eval_body_of` uses `lookup_form` and matches on Binding's 5 variants. UserFunction + Primitive arms preserve existing behavior verbatim. Macro + Type arms call new helpers. SpecialForm arm emits sentinel / signature directly. |
| 6 | 4 NEW helpers | `macrodef_to_define_ast(&MacroDef) -> WatAST`, `macrodef_to_signature_ast(&MacroDef) -> WatAST`, `typedef_to_define_ast(&TypeDef) -> WatAST`, `typedef_to_signature_ast(&TypeDef) -> WatAST` — placed near existing helpers (5970-6082). |
| 7 | New test file | `tests/wat_arc144_lookup_form.rs` exists with 5+ tests covering Macro lookup (define + signature + body), Type lookup (define + signature; body returns None), no-regression for UserFunction + Primitive paths, unknown name returns None across all 3 primitives. |
| 8 | **Existing arc 143 tests still green** | `cargo test --release --test wat_arc143_lookup` ALL PASS (zero behavior change for arc 143 slice 1). `cargo test --release --test wat_arc143_manipulation` ALL PASS. `cargo test --release --test wat_arc143_define_alias` 2/3 (length canary unchanged; foldl + unknown still pass). |
| 9 | **`cargo test --release --workspace`** | Same failure profile as pre-slice-1: only the slice 6 length canary fails. ZERO new regressions. |
| 10 | Honest report | ~250-word report covers: Binding enum verbatim, lookup_form summary, the 3 dispatch updates per kind, 4 helper signatures, 5+ new tests with assertions, test totals, clippy results, honest deltas. |

**Hard verdict:** all 10 must pass. Rows 5 + 8 + 9 are the load-
bearing rows (Binding dispatch fans out to all consumers AND
preserves arc 143's behavior).

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget | Total slice diff: 300-500 LOC (runtime.rs + new test file). >600 LOC = re-evaluate. |
| 12 | Style consistency | New code follows existing patterns: arg-validation pattern in eval_*; helper placement adjacent; Span::unknown() in synthesized ASTs. |
| 13 | clippy clean | `cargo clippy --release --all-targets` shows no new warnings (especially around `Binding<'a>` lifetime + reference borrows in the variants). |
| 14 | Sentinel honesty | Macro signature uses `:AST<wat::WatAST>` for params (the brief's "honest sentinel" — params ARE WatAST values; specific T isn't tracked in MacroDef today). Type signature emits head-only (no field rendering). Both match the brief's "honest sentinel beats half-rendered." |

## Independent prediction

- **Most likely (~50%) — Mode A clean ship.** The brief is pre-
  flighted; sonnet executes the refactor mechanically. ~25-40 min
  wall-clock (the longest slice in the arc 144 plan; subsequent
  slices are smaller).
- **Surprise on helper shape (~25%) — Mode B-helper.** Sonnet
  finds that one of the 4 helpers needs a different signature
  (e.g., `typedef_to_signature_ast` needs `&TypeEnv` for typealias
  expansion). Adapts; minor delta surfaced honestly.
- **Surprise on test shape (~15%) — Mode B-test.** Sonnet finds
  the test needs a wat helper (e.g., the type lookup test needs to
  declare a struct with a specific shape). Adapts; the test setup
  costs more than the brief anticipated.
- **Sweep gets stuck on lifetime / borrow checker (~10%) — Mode C.**
  The `Binding<'a>` enum's lifetime propagation through
  `lookup_form` and the 3 eval_* primitives may surface
  borrow-check friction. If sonnet hits this, the report should
  surface it cleanly with the specific compile error; orchestrator
  decides whether to scope down to Owned variants or push through.
- **Mode B-LookupResult-public (~5%, IF GREP HITS).** Sonnet's
  brief-mandated grep finds an unexpected `LookupResult` consumer
  outside the 3 primitives. If found, STOP and report — orchestrator
  scopes the refactor accordingly.

**Time-box: 60 min cap (2× upper-bound 30 min).**

## Methodology

After sonnet returns:
1. Read this file FIRST.
2. Score each row.
3. Diff via `git diff --stat` — 1 Rust file modified + 1 new test
   file expected.
4. Read the Binding enum definition + lookup_form body to confirm
   the 4-registry walk + SpecialForm stub.
5. Read each of the 3 refactored eval_* primitives to confirm the
   5-variant dispatch.
6. Run `cargo test --release --test wat_arc143_lookup` — confirm
   ALL pass (no regression).
7. Run `cargo test --release --test wat_arc143_define_alias` —
   confirm 2/3 (length canary unchanged).
8. Run `cargo test --release --test wat_arc144_lookup_form` —
   confirm new tests pass.
9. Run `cargo test --release --workspace` — confirm only the
   length canary fails; tally regression count.
10. Run `cargo clippy --release --all-targets` — confirm no new
    warnings.
11. Score; commit `SCORE-SLICE-1.md`.

## What this slice unblocks

- **Slice 2** — special-form registry can populate the
  SpecialForm Binding variant; the dispatch is already in place.
- **Slice 3** — hardcoded primitive TypeScheme registrations
  become visible to `lookup_form` immediately (via the existing
  CheckEnv walk).
- **Slice 4** — verification can re-run arc 143's slice 6 length
  test; once slice 3 ships the length scheme, the test turns green
  and arc 144's load-bearing canary is closed.
- **Future arc 141 (docstrings)** — populates the `doc_string`
  field on each Binding variant; no enum refactor needed.
- **Future REPL `(help X)` form** — composes lookup_form +
  signature-of + body-of + doc-string-of (when arc 141 ships) into
  a uniform help form.

The "nothing is special" principle is honored at the substrate
boundary: any consumer of `lookup_form` sees the same 5-variant
union for any known wat form.
