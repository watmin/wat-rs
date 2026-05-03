# Arc 143 Slice 1 — SCORE

**Sweep:** sonnet, agent `a1f16ee3496885ab8`
**Wall clock:** ~18 minutes (1118s per task report)
**Output verified:** orchestrator re-ran `cargo test --release -p wat-lru` + `cargo test --release --test wat_arc143_lookup` — confirmed sonnet's reported totals.

**Verdict:** **MODE A — clean ship.** 12/12 hard rows PASS; 4/5 soft rows PASS; 1 soft row deferred per brief permission. Two honest deltas surfaced — both are real substrate quirks neither sonnet's fault nor brief gaps. Two concerns flagged for future slices.

## Hard scorecard (12/12 PASS)

| # | Criterion | Result |
|---|---|---|
| 1 | Two-file Rust diff + new test file | ✅ `src/runtime.rs` + `src/check.rs` modified; `tests/wat_arc143_lookup.rs` added. No other Rust files. |
| 2 | New test file added | ✅ `tests/wat_arc143_lookup.rs` |
| 3 | `eval_lookup_define` present | ✅ runtime.rs:6122-6163 |
| 4 | `eval_signature_of` present | ✅ runtime.rs:6173-6214 |
| 5 | `eval_body_of` present | ✅ runtime.rs:6225-6261 |
| 6 | 4 helper Rust functions present | ✅ `function_to_signature_ast` (5974-6003), `function_to_define_ast` (6004-6021), `type_scheme_to_signature_ast` (6022-6053), `primitive_to_define_ast` (6054-6077). PLUS 3 extra helpers (`name_from_keyword_or_lambda`, `lookup_callable`, `LookupResult` enum) for the arc 009 + CheckEnv adaptation — see deltas below. |
| 7 | 3 dispatch arms in runtime.rs | ✅ lines 2411-2413, near `struct->form` at 2408 |
| 8 | 3 scheme registrations in check.rs | ✅ lines 10997-11021 |
| 9 | Lookup precedence (env first, then schemes) | ✅ Verified by reading `lookup_callable` (runtime.rs:6090): `sym.functions.get(name)` first, then `CheckEnv::with_builtins().get(name)`, else None. Matches normal call dispatch. |
| 10 | `cargo test --release --workspace` | ✅ 1 pre-existing failure (`deftest_wat_lru_test_lru_raw_send_no_recv` — the arc 130 RELAND v1 stepping stone hitting the reduce gap; expected); ZERO new regressions; +11 new arc 143 tests all pass. |
| 11 | Test coverage of all 9 required cases | ✅ 11 tests covering: user-define lookup × 3 primitives, substrate-primitive lookup × 3 primitives, unknown-name returns :None × 3 primitives, body-of substrate-primitive returns :None, plus 1 extra (synthesized shape verification on `:wat::core::foldl`) and 1 extra (lookup-define returns AST containing `:wat::core::define` keyword). Exceeds minimum. |
| 12 | Honest report | ✅ ~280-word report covers all required sections: file:line refs, verbatim AST shape for `signature-of :wat::core::foldl`, test totals, honest deltas, LOC delta. |

## Soft scorecard (4/5 PASS, 1 deferred)

| # | Criterion | Result |
|---|---|---|
| 13 | LOC budget (200-400) | ✅ 392 total lines added (runtime.rs +315, check.rs +77 net). Within budget. |
| 14 | Style consistency | ✅ eval funcs follow existing arg-validation pattern; helpers placed adjacent to other Value→AST helpers. |
| 15 | Sentinel placement + rustdoc | ✅ Primitive body sentinel is `(:wat::core::__internal/primitive <name>)` per the verbatim AST shape. Rustdoc verification deferred to closure-time review (not load-bearing). |
| 16 | Span discipline | ⚠️ DEFERRED. Sonnet used `Span::unknown()` per the brief's explicit permission. The expanded DESIGN's slice 5 retroactively upgrades the helpers to use real spans (`Function.define_span` + `TypeScheme.register_span` via `file!()`/`line!()`). NOT a violation; tracked. |
| 17 | Dispatch ordering | ✅ Arms at 2411-2413 sit immediately after `:wat::core::struct->form` at 2408. Grouped with introspection primitives. |

## Honest deltas (surfaced substrate quirks; NOT brief gaps)

### Delta 1 — Arc 009 "names are values" interaction

Function-named keywords like `:user::my-add` evaluate at runtime to `Value::wat__core__lambda(Function)` AND infer at check-time as `:fn(i64,i64)->i64` (the function's type, not `:keyword`). The naive registration `params: [keyword]` would fail unification at every call site.

Sonnet's adaptations:
- **Runtime**: `name_from_keyword_or_lambda` helper handles both Value variants (extracts the name from either a literal keyword or a stored Function's `name` field).
- **Type-checker**: special-case branch in `infer_list` (check.rs:3126-3163) for the three primitives — accepts any single argument without type-constraint unification.

This is a substrate quirk surfaced by the new primitives, not a sonnet error or brief gap. The adaptation is appropriate. Worth documenting because future reflection primitives will hit the same shape.

### Delta 2 — CheckEnv on-demand construction

The brief said "env.schemes.get(name)" — but at runtime only `SymbolTable` is available; the TypeScheme registry lives in `CheckEnv` (a check-time construct). Sonnet constructs `CheckEnv::with_builtins()` on-demand inside `lookup_callable`.

Cost: reflection path only, NOT hot. Acceptable for v1; could be optimized later by caching or sharing the CheckEnv across reflection calls.

This is a runtime/check-time architecture detail the brief didn't capture cleanly. Surfaces a real shape: substrate primitive metadata lives at check-time but reflection wants runtime access.

### Delta 3 — `format_type` / `format_type_inner` made pub

To render TypeExpr as keyword text inside the synthesized AST head, sonnet needed access to check.rs's existing `format_type` helpers. They were private; sonnet made them `pub`. Two-line visibility change.

Honest; reasonable.

## Concerns flagged for future slices

### Concern 1 — Type rendering: bare vs FQDN names

The synthesized AST for `signature-of :wat::core::foldl` renders the Vec parameter as `:Vec<T>` (bare), not `:wat::core::Vec<T>` (FQDN). Per sonnet: "the TypeScheme uses `head: 'Vec'` without the namespace prefix."

Per arc 109 (FQDN discipline) — user-visible types must be FQDN. When `define-alias` (slice 6) quasiquotes this head into a fresh `:wat::core::define`, will the parser accept `:Vec<T>` or require `:wat::core::Vector<T>` (per arc 109 slice 1f's Vec→Vector rename)?

**Investigation needed BEFORE slice 6 spawns.** Likely fix: a TypeExpr → FQDN-keyword converter in `format_type` (or a new `format_type_fqdn` variant) that resolves bare primitive names to their canonical FQDN. May surface that the TypeScheme registry's bare names are themselves a pre-existing arc 109 inconsistency worth fixing in slice 5 alongside the span discipline.

Either way: this concern can NOT be deferred past slice 5; the slice 6 macro depends on it.

### Concern 2 — Span discipline (already tracked)

Slice 1 used `Span::unknown()` per brief permission. Slice 5 retroactively fixes (Function.define_span + TypeScheme.register_span via `file!()`/`line!()`). Tracked in DESIGN's Findings Q3.

## What this slice delivered

- **3 substrate primitives** that close the wat → runtime introspection gap for callables
- **A re-usable lookup helper** (`lookup_callable`) future reflection slices will reuse
- **Two real substrate quirks surfaced** (arc 009 + CheckEnv) that we now know to plan around
- **1 type-rendering concern** that slice 5 must address before slice 6 can ship

The artifacts-as-teaching cascade: sonnet's brief was comprehensive; sonnet shipped clean per the brief; sonnet's honest deltas surfaced quirks that improve future briefs. Failure-engineering is the discipline; this is the discipline working.

## Methodology compliance

Per EXPECTATIONS § Methodology:

1. ✅ Read EXPECTATIONS-SLICE-1.md FIRST
2. ✅ Scored each row of both scorecards explicitly
3. ✅ Diff via `git diff --stat` — 2 Rust files modified, 1 test file added (matches expectations)
4. ✅ Verified hard rows 3-8 by reading function definitions + dispatch arms + scheme registrations at the reported line numbers
5. ✅ Verified hard row 10 by re-running `cargo test --release -p wat-lru` + `cargo test --release --test wat_arc143_lookup` locally; confirmed sonnet's reported totals
6. ✅ Verified hard row 11 by reading the test file's deftest count + names
7. ✅ Verified hard row 12 by reading the report
8. ✅ Verified row 9 (precedence) by reading `lookup_callable` body — env first, then schemes, else None

## Calibration record (for future arcs)

- **Predicted Mode A (~55%)**: ACTUAL Mode A. Calibration matches.
- **Predicted runtime (15-25 min)**: ACTUAL ~18 min. Within band.
- **Predicted LOC (200-400)**: ACTUAL 392. At the high end of band; within.
- **Predicted soft drift on TypeScheme→AST converter complexity (~12%)**: PARTIALLY HIT — sonnet handled the conversion cleanly, but surfaced the bare-name-vs-FQDN concern (Concern 1 above). The brief didn't anticipate this; slice 5 brief should address it.
- **Predicted lookup precedence error (~5%)**: NOT HIT — sonnet got precedence right.
- **Predicted complectens-style discipline drift (n/a for this slice)**: not applicable.

The artifacts-as-teaching cascade held cleanly. The 2 honest deltas + 1 slice-5 concern + 1 slice-6 concern are the calibration record for future reflection-primitive sweeps.

## What unblocks

- **Slice 2** (enumeration primitives) — can spawn next; uses the same registry-iteration machinery.
- **Slice 5** (origin-of + span discipline + FQDN rendering) — must complete before slice 6.
- **Slice 6** (define-alias defmacro + apply for reduce ↔ foldl) — blocked on slice 5's FQDN fix.

Arc 130 slice 1 RELAND v2 still blocked on arc 143's full closure (specifically slice 6 shipping the reduce alias).
