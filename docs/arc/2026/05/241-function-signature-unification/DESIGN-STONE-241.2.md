# DESIGN — Stone 241.2 — migrate A1/A2/A3 fn parsers through canonical

**Status:** READY (sub-DESIGN). Phase 1 second stone. Blocks Stone 241.3 per spawn-block winding. Mirrors Stone 241.1.fix discipline: SCORE-green is the floor; vigilia-convergence is the bar.

## Why this stone

Stone 241.1.fix shipped the canonical `parse_argspec_triples` with vigilia-convergence across all 8 spells. The home is exceptional. Three fn-form parsers (A1/A2/A3) carry duplicated triple-walking logic that the canonical parser now subsumes:

- **A1** `src/runtime.rs:6750` `parse_fn_signature` — `Result<(Vec<String>, Vec<TypeExpr>, TypeExpr), RuntimeError>`
- **A2** `src/check.rs:15205` `parse_fn_signature_for_check` — `Result<..., ()>` (silent-path)
- **A3** `src/check.rs:15258` `parse_fn_signature_for_check_diag` — `Option<...>` (errors pushed by-ref into `&mut Vec<CheckError>`)

All three consume the canonical `[name <- :T name <- :T ...] -> :Ret body` form. Their inner triple walkers duplicate ~15 lines each. Migration routes them through `parse_argspec_triples` for the args part; ret-clause parsing stays inline per site (each retains its own error semantics).

Per failure-engineering: the class to eliminate is *N parallel triple walkers*. Migrating A1/A2/A3 collapses it to 1 + 3 thin call sites; A4 (defclause) migrates in Stone 241.3; the class closes when 241.3 lands.

## What this stone delivers

Three migrations + behavioral parity verification. NO public API changes; A1/A2/A3 signatures stay identical; their callers (`eval_fn`, `try_parse_fn_shape_def`, silent-path `infer_fn`, diagnostic-path `infer_fn`) are UNTOUCHED.

### M1 — A1 `parse_fn_signature` at `src/runtime.rs:6750`

Replace the inline triple walker (~25 lines, the `while i < args_vec.len()` loop) with:

```rust
let spec = parse_argspec_triples(
    args_vec,
    ":wat::core::fn",
    args_vec_span,         // the Vector form's span
    ParseOptions { allow_rest_binder: false },
)?;
```

The `?` operator converts `ArgSpecError` → `RuntimeError` via the `From<ArgSpecError> for RuntimeError` impl shipped in Stone 241.1.fix. Repackage `spec.fixed_params` into `(Vec<String>, Vec<TypeExpr>)`:

```rust
let (names, types): (Vec<String>, Vec<TypeExpr>) = spec.fixed_params.into_iter().unzip();
```

### M2 — A2 `parse_fn_signature_for_check` at `src/check.rs:15205`

Same shape; A2 silences errors as `()`. The migration uses `.map_err(|_| ())`:

```rust
let spec = parse_argspec_triples(
    args_vec,
    ":wat::core::fn",
    args_vec_span,
    ParseOptions { allow_rest_binder: false },
).map_err(|_| ())?;
```

A2's existing `match args[i+1] { Symbol("<-") => {}, _ => return Err(()) }` pattern becomes implicit in the canonical parser's `MissingArrow` variant; the silencing is the same.

### M3 — A3 `parse_fn_signature_for_check_diag` at `src/check.rs:15258`

A3 pushes errors into `&mut Vec<CheckError>` and returns `None`. The `From<ArgSpecError> for CheckError` impl exists (Stone 241.1.fix). Pattern:

```rust
let spec = match parse_argspec_triples(
    args_vec,
    ":wat::core::fn",
    args_vec_span,
    ParseOptions { allow_rest_binder: false },
) {
    Ok(s) => s,
    Err(e) => {
        errors.push(e.into());
        return None;
    }
};
```

A3 gains BETTER error fidelity than today (the canonical parser emits per-triple error variants where A3's current walker emits one generic message per failure).

### Ret-clause parsing stays inline (per-site)

The `match args[1] { Symbol("->") => {}, ... }` + `match args[2] { Keyword(k) => parse_type_*(k), ... }` blocks at each site stay UNCHANGED. Three reasons:

1. **Per user verdict 2026-05-28**: "args have nothing to do with ret type." Ret-clause is fn-form-parser concern; lives at the fn-form parser, not in argspec.
2. **Per-site error semantics differ**: A1 (`RuntimeError`), A2 (`()`), A3 (push `CheckError`). A shared helper would either uniformly Err (forcing per-site `.map_err`/match shim) or branch internally (god-helper). Both are uglier than 3 short inline blocks.
3. **The ret-clause inline is small** (~6 lines per site × 3 = 18 lines total). Three identical-but-different (different error class) sites is fine. Stone 241.2 minus the deeper refactor.

## Locked decisions

### D1 — A1/A2/A3 PUBLIC API UNCHANGED

`parse_fn_signature`, `parse_fn_signature_for_check`, `parse_fn_signature_for_check_diag` keep their existing signatures (return types, parameter types, fn name). Their callers don't know the migration happened.

### D2 — Migration is INTERNAL substrate refactor

The diff is confined to:
- `src/runtime.rs` — body of A1 only
- `src/check.rs` — body of A2 + A3 only

NO other files touched in 241.2. NO new exports. NO new types. NO new helpers minted (parse_ret_clause is NOT minted; see "Ret-clause parsing stays inline" above).

### D3 — Error-message regression is expected and documented

After migration, the error messages produced via `From<ArgSpecError>` impls (Stone 241.1.fix shipped) are domain-neutral: "name slot must be a plain symbol (not a keyword, literal, or nested form)", "triple must be `name <- :T`; `<-` arrow not found at slot 1", etc.

These will DIFFER from A1's current inline messages (e.g., A1 currently emits "Expected Symbol at slot N" or similar). Tests asserting against the OLD message strings will break.

**The migration is honest**: the messages improve. Per `feedback_substrate_diagnostics_are_brief` (and `docs/SUBSTRATE-AS-TEACHER.md`): the substrate's diagnostic stream IS the migration brief. Sonnet finds, reads, fixes test message assertions one by one until baseline holds.

### D4 — Form span sourcing

The canonical parser needs `form_span: &Span` — the args-vector's own span. A1 currently destructures the Vector as `WatAST::Vector(items, span)` — that `span` IS the form_span. Pattern:

```rust
let (args_vec, args_vec_span) = match args_vec_node {
    WatAST::Vector(items, span) => (items, span),
    other => { return Err(...); }
};
```

A1/A2/A3 destructure the Vector identically; sourcing the span is mechanical.

### D5 — `.into_iter().unzip()` for fixed_params repackaging

`spec.fixed_params: Vec<(String, TypeExpr)>` → `(Vec<String>, Vec<TypeExpr>)` via `.into_iter().unzip()`. Type-annotated:

```rust
let (names, types): (Vec<String>, Vec<TypeExpr>) = spec.fixed_params.into_iter().unzip();
```

The explicit type annotation prevents inference ambiguity. Standard Rust idiom.

### D6 — `spec.ret_type` is `None` (the field doesn't exist post-241.1.fix)

Stone 241.1.fix STRIPPED `ret_type` from `ArgSpec`. The canonical parser returns ONLY `fixed_params` + `rest_param`. Ret-clause parsing happens AFTER the canonical call, inline at each A1/A2/A3 site, before returning the (names, types, ret_type) tuple.

### D7 — `spec.rest_param` should ALWAYS be `None` in 241.2

`ParseOptions { allow_rest_binder: false }` means `parse_argspec_triples` rejects any `&` it encounters via `RestBinderNotSupported`. So `spec.rest_param.is_none()` after a successful parse — always. A1/A2/A3 don't need to handle Some(rest_param); the field is permanently None at this caller-site until Stone 241.5 ships defclause opt-in (and even then, fn-form sites stay `allow_rest_binder: false`).

**A debug_assert** in M1/M2/M3 could enforce this: `debug_assert!(spec.rest_param.is_none(), "fn-form sites never opt into rest-binder");`. OPTIONAL — surface as candidate; the parser already guarantees it; the assertion is documentation, not enforcement.

### D8 — No regression tests added by 241.2; existing tests are the regression suite

The migration is internal; existing tests covering fn-form parsing (eval_fn paths, infer_fn paths, defn macro expansion, etc.) ARE the regression suite. The FM 2-bis probe at `tests/probe_arc241_stone2_fn_parser_migration.rs` will be MINIMAL — perhaps just a behavioral parity check on a handful of inputs (good fn-form parses identically; bad fn-form produces semantically-equivalent errors). The bulk of regression confidence comes from existing tests staying green.

If TestX asserts against the old error message verbatim, TestX needs its assertion UPDATED to match the new canonical-domain-neutral message. This is mechanical; the substrate-as-teacher cascade surfaces each TestX one by one.

### D9 — Vigilia-gate doctrine applies

Per `feedback_namespaced_home_vigilia_gate`: Stone 241.2 touches `src/runtime.rs` (NOT a namespaced home — pre-existing flat substrate). The vigilia gate is for namespaced wat-rs files (`src/<noun>/`); `src/runtime.rs` is pre-ward-doctrine legacy per `feedback_ward_zone_comms_only`.

**Therefore Stone 241.2 commits on SCORE-green** without a vigilia cast. The cast is reserved for namespaced homes. Stone 241.2's substrate edits live in legacy flat files; the gate doesn't fire.

If Stone 241.2 surfaces an opportunity to NAMESPACE (e.g., a `src/fn_form/` home for fn-signature parsing later in 241.10), that future stone gets the gate; not 241.2.

### D10 — Lib baseline + arc 237 + arc 238 + probe regression checks

After Stone 241.2:
- `cargo test --release --lib -p wat` = 834+ PASS / 0 FAIL (must hold ≥834)
- `cargo test --release --test probe_arc241_stone1_argspec_canonical` = 9/9 PASS (unchanged)
- `cargo test --release --test probe_arc241_stone2_fn_parser_migration` = N/N PASS (new probe; ~4-6 contracts)
- `cargo build --release --tests --workspace` clean
- `cargo clippy --release` ≤ 905
- Arc 237/238 regression: stone5_conforms 12, stone5fix_nominal 12, stone6_is_predicate 10, eq_completeness 8

---

## Trap-door audit

### T1 — Error-message-asserting tests will break

A1's inline messages don't match the canonical-domain-neutral messages. Tests like `assert!(err.to_string().contains("Expected Symbol at args_vec"))` will fail. The substrate-as-teacher cascade reveals them; sonnet updates each assertion to match the new canonical message. EXPECTED behavior; not a STOP trigger; mechanical work.

Mitigation: surface as honest delta in SCORE; enumerate affected tests + the assertion updates.

### T2 — A2's silent `()` path may hide error-quality differences

A2 silences errors as `()`. After migration, the silenced error class CARRIES the canonical's richer error info (which is then discarded by `.map_err(|_| ())`). The silenced path's behavior is UNCHANGED externally — it still falls through to None at the caller.

Net effect: zero observable difference at A2's caller boundary. Honest.

### T3 — A3's diagnostic push gains BETTER error fidelity

A3's existing walker emits one generic message per failure shape ("expected Symbol at slot N", "expected -> at slot M"). The canonical parser emits per-triple error variants with precise spans + the canonical domain-neutral wording.

Tests that assert against A3's exact old messages may fail; the IMPROVED messages should be accepted. Document the upgrade in SCORE.

### T4 — Form span sourcing nuance

A1's current arity error uses `Span::unknown()` (defensive fallback). The migration's argspec call uses the args-vector's own span, which IS the real form_span. Per-element errors get per-element spans from the canonical parser; arity errors stay using `Span::unknown()` or the args[0] span.

Cleanup opportunity: A1's arity error could use a better span (e.g., the form head's span if accessible) but that's OUT OF 241.2 SCOPE.

### T5 — `args[0]` is always the args-vector at A1/A2/A3

The fn-form is `(:wat::core::fn [args...] -> :Ret body)`. After head-stripping, args[0] is the args-vector, args[1] is `->`, args[2] is ret-keyword, args[3] is body. All three sites assume `args.len() == 4`. This is the fn-form's CANONICAL outer shape — locked since arc 109 retirement.

### T6 — `parse_type_keyword` vs `parse_type_expr` vs `parse_type_expr_with_span`

A1 uses `parse_type_keyword` (no span); A2/A3 use `parse_type_expr` (no span). The canonical argspec parser uses `parse_type_expr_with_span` internally. AFTER migration:
- The argspec args are parsed via `parse_type_expr_with_span` (canonical)
- The ret-clause keyword (inline at each site) still uses each site's original helper

This is an INCONSISTENCY within the same fn-form parser — args use one type-keyword parser; ret uses another. Surface as Honest Delta.

Possible Stone 241.2.fix follow-up: unify the type-keyword helper across args + ret. OUT OF 241.2 SCOPE; queue for a future arc.

### T7 — `parse_argspec_triples` consumes `args_vec: &[WatAST]` — borrow lifetime fit

The canonical parser takes a slice reference; A1/A2/A3 destructure `WatAST::Vector(items, _)` to get `items: &Vec<WatAST>`. Passing `items` as `&[WatAST]` works via Vec's Deref<Target=[T]>. No lifetime ceremony.

### T8 — A2's `i + 2 >= args_vec.len()` becomes implicit

A2's `if i + 2 >= args_vec.len() { return Err(()) }` becomes `ArgSpecError::IncompleteTriple` via the canonical parser; `.map_err(|_| ())` silences to `()`. Semantically identical; the silenced-path-fall-through behavior preserved.

### T9 — Behavioral parity: identical inputs produce identical outputs

For ANY well-formed `[name <- :T name <- :T ... -> :Ret]` input, A1/A2/A3 post-migration MUST produce the same `(names, types, ret_type)` tuple as pre-migration. The FM 2-bis probe verifies this on representative inputs.

### T10 — `eval_fn`, `try_parse_fn_shape_def`, `infer_fn` (silent + diag) callers UNTOUCHED

A1/A2/A3 are called from:
- `eval_fn` (runtime.rs:6696) → A1
- `try_parse_fn_shape_def` (runtime.rs:3913, 4000) → A1
- silent-path `infer_fn` (check.rs:9592) → A2
- diagnostic-path `infer_fn` (check.rs:15154) → A3

These callers DON'T CHANGE. The `?` propagation + `.map_err(|_| ())` + match-on-Err patterns stay at the site boundary; the callers see the same tuple result.

---

## STOP triggers (REJECTION — not permission to defer)

1. **STOP-1** — Unexpected compile errors not traced to the migration call sites
2. **STOP-2** — Lib baseline regression (current: 834; must hold ≥834 — unless message-string assertions update is in-flight; in that case, message changes are HONEST DELTAS, not regressions)
3. **STOP-3** — 60 min elapsed (broader scope than 241.1.fix; ~30 lines per site × 3 sites + test-assertion updates)
4. **STOP-4** — `holon-rs` touched (substrate is frozen)
5. **STOP-5** — Rust files outside `src/runtime.rs`, `src/check.rs`, and ONE new probe file (`tests/probe_arc241_stone2_fn_parser_migration.rs`) touched. `src/argspec/*` MUST stay unchanged.
6. **STOP-6** — Scope creep:
   - Migrating A4 (defclause) — that is 241.3
   - Minting `parse_ret_clause` — out of scope per D2
   - Changing A1/A2/A3 PUBLIC signatures — D1 violation
   - Unifying type-keyword helpers (T6 finding) — queue for follow-up
7. **STOP-7** — Probe doesn't reach N/N PASS (where N is determined by the probe's contract count, ~4-6)
8. **STOP-8** — Any prior arc 237 probe regresses
9. **STOP-9** — Clippy warnings increase above 905
10. **STOP-10** — Migration produces SUBTLY different behavior (not just different error messages — different parsed result for valid inputs); surface as finding

Each STOP is REJECTION criteria.

---

## FM 2-bis evidence

`tests/probe_arc241_stone2_fn_parser_migration.rs` (NEW). Minimal contracts proving behavioral parity:

| # | Contract | Path |
|---|---|---|
| 1 | A1 happy path: `(fn [x <- :i64 y <- :i64] -> :i64 body)` parses to expected names/types/ret | via `:wat::core::fn` form |
| 2 | A1 NameNotSymbol: `(fn [:kw <- :i64] -> :i64 body)` produces `RuntimeError::MalformedForm` with canonical-domain-neutral message | via `:wat::core::fn` form |
| 3 | A1 MissingArrow: `(fn [x = :i64] -> :i64 body)` produces `RuntimeError::MalformedForm` | via `:wat::core::fn` form |
| 4 | A2 silent-path happy: silent infer_fn parses identical fn-form cleanly (existing test pattern) | via check.rs path |
| 5 | A3 diagnostic-path bad: malformed fn-form pushes CheckError; returns None | via check.rs diagnostic path |
| 6 | A3 diagnostic-path happy: well-formed fn-form returns Some((names, types, ret_type)) | via check.rs diagnostic path |

Contracts 1-3 exercise A1 via a high-level wat form parse; contracts 4-6 exercise A2/A3 via the check.rs entry points. The probe doesn't need direct fn-pointer access (A1/A2/A3 are private); it routes through `eval_fn` / `infer_fn` / etc.

**Pre-stone**: probe contracts 1-3 ALREADY pass (the old A1 path); contract patterns 4-6 also pass via existing parsers. The probe doesn't compile-fail at HEAD; it proves the substrate post-migration RETAINS the behavior.

**FM 2-bis discipline**: the probe is a BEHAVIORAL PARITY check, not a diagnostic isolation probe. This is a different use of FM 2-bis than Stone 241.1 (which used compile-fail isolation). Both are valid; parity probes catch regression rather than gate substrate existence.

---

## SCORE doc spec

`docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.2.md` (NEW). Mirror SCORE-STONE-241.1.fix.md structural shape:

- **Header**: status (Mode A/B); runtime; one-line summary
- **Phase A scorecard** ~12-15 rows covering lib, probe, workspace, clippy, file discipline, arc 237/238 regression, behavioral parity confirmation
- **Migration audit** — per-site diff summary (A1/A2/A3 line counts before/after)
- **Final code shape** at each site (verbatim post-migration body)
- **Error-message changes inventory** — every test assertion updated; before/after string
- **Honest deltas** — T6 type-keyword inconsistency; T4 form-span sourcing nuance; any other surfaces
- **NO Vigilia Convergence section** — Stone 241.2 commits on SCORE-green per D9 (flat-file substrate; gate doesn't apply)

---

## Calibration

**Target band:** 40–60 min Mode A.
**Upper bound:** 90 min (STOP-3 at 60 + buffer for test-assertion updates).

**Surface estimate (net delta ~-30 to -50 lines from migration; +50 from probe; +N from test-assertion updates):**

| File | Pre-migration | Post-migration | Delta |
|---|---|---|---|
| `src/runtime.rs` (A1 body) | ~50 lines | ~30 lines | **-20** |
| `src/check.rs` (A2 body) | ~45 lines | ~28 lines | **-17** |
| `src/check.rs` (A3 body) | ~75 lines | ~50 lines | **-25** |
| `tests/probe_arc241_stone2_fn_parser_migration.rs` (NEW) | 0 | ~80 | **+80** |
| (various test files with error-msg assertions) | N | N | depends on assertion count |
| **Net delta** | — | — | **~+18 lines + N test-assertion updates** |

**Confidence: HIGH.** Mechanical migration; locked decisions; no new types; no public API change. The error-message-asserting tests are the main risk; mitigated by treating message changes as HONEST DELTAS.

**Per `feedback_stone_briefs_cite_prior_score`**: BRIEF cites `SCORE-STONE-241.1.fix.md` for:
- Canonical parser surface (signatures, error variants, classify())
- Vigilia-validated home (argspec module is exceptional; migration callers route through it)
- `From<ArgSpecError>` impls available for RuntimeError / CheckError / TypeError

---

## What this unblocks

Stone 241.3 — migrate A4 (defclause) parser at `runtime.rs:6880`. Same shape as A1/A2/A3 but `include_ret_type` is FALSE (defclause has no `-> :Ret`). After 241.3, all 4 fn/defclause parsers route through canonical; the class is closed.

Stone 241.4 — extend canonical with `&` rest-binder; A4's caller (`parse_defclause_clause`) opts in via 241.5; probe 237.8b Gate 1 flips green.

---

## Cross-references

- `SCORE-STONE-241.1.fix.md` § Vigilia Convergence — the validated foundation Stone 241.2 builds on
- `AUDIT.md` — per-site invariant matrix; the migration shape
- `DESIGN.md` § Scope expansion 2026-05-28 — arc-level framing
- `feedback_namespaced_home_vigilia_gate` — gate doctrine (D9: doesn't apply here; legacy flat substrate)
- `feedback_ward_zone_comms_only` — wards-optional for broader codebase
- `feedback_stone_briefs_cite_prior_score` — BRIEF cites Stone 241.1.fix SCORE for structural shape
- `feedback_substrate_diagnostics_are_brief` + `docs/SUBSTRATE-AS-TEACHER.md` — error-message changes surface as cascade; sonnet follows
- `feedback_sonnet_writes_substrate` — orchestrator briefs + scores; sonnet writes
- COMPACTION-AMNESIA-RECOVERY § FM 2-bis — behavioral-parity probe shape (different from 241.1's isolation probe)
