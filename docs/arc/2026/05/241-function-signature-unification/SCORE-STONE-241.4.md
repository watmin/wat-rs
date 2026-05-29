# SCORE — Stone 241.4 — canonical `&` rest-binder extension + defclause opt-in; unblocks 237.8b Gate 1

**Status:** Mode A — PASS (with Gate 1 honest delta; see below)
**Runtime:** ~35 min (within 30–50 min target band)
**Summary:** Argspec extension shipped complete. `parse_triple` private helper extracted (struere). All 3 future-fixture runes retired. Doc comments updated to descriptive. A4 `parse_defclause_args` returns `ArgSpec` with `allow_rest_binder: true`. Clause struct gains `rest_param: Option<(String, TypeExpr)>` field; parser threads it through. Canonical probe: 15/15 PASS. Gate 1 un-ignored; RED — runtime dispatch does not yet wire rest-binder to variadic dispatch. STOP-6 condition: >30 lines of dispatch wiring required; surfaced as honest gap for follow-up stone. Lib 834 PASS. Clippy 905 (delta 0). Workspace build clean.

---

## Phase A Scorecard

| Row | Claim | Result |
|-----|---|---|
| 1 | Canonical probe contracts 01-09 still PASS (regression) | **PASS** — 9 passed; 0 failed |
| 2 | Canonical probe contract 10 PASS (rest-only succeeds) | **PASS** — 1 passed; 0 failed |
| 3 | Canonical probe contract 11 PASS (fixed+rest succeeds) | **PASS** — 1 passed; 0 failed |
| 4 | Canonical probe contract 12 PASS (TrailingItems verified — DESIGN T2 verdict β confirmed) | **PASS** — 1 passed; 0 failed |
| 5 | Canonical probe contracts 13-15 PASS (incomplete + non-symbol + regression) | **PASS** — 3 passed; 0 failed |
| 6 | Canonical probe whole-suite 15/15 | **PASS** — 15 passed; 0 failed |
| 7 | Stone 241.2 probe preserved 10/10 | **PASS** — 10 passed; 0 failed |
| 8 | Stone 241.3 probe preserved 6/6 | **PASS** — 6 passed; 0 failed |
| 9 | 237.8b Gate 1 PASSES (was `#[ignore]`'d; now active) | **RED — HONEST DELTA** — 1 failed; runtime dispatch does not wire rest-binder to variadic arity/type-check/binding; STOP-6 condition surfaced; see Honest Deltas |
| 10 | Lib baseline preserved | **PASS** — 834 passed; 0 failed; 1 ignored |
| 11 | Workspace test-build clean | **PASS** — `cargo build --release --tests --workspace` exit 0; 0 errors |
| 12 | Clippy delta = 0 | **PASS** — 905 warnings (baseline 905; delta 0) |
| 13 | No prior arc 237 probe (non-Gate-1) regresses | **PASS** — probe_arc237_stone5_conforms: 12 pass, probe_arc237_stone5fix_nominal: 12 pass, probe_arc237_stone6_is_predicate: 10 pass, probe_arc238_eq_completeness: 8 pass |
| 14 | `src/lib.rs` UNCHANGED | **PASS** — `git diff src/lib.rs` empty |
| 15 | `src/argspec/mod.rs` UNCHANGED | **PASS** — `git diff src/argspec/mod.rs` empty |
| 16 | `src/check.rs` UNCHANGED | **PASS** — `git diff src/check.rs` empty |

---

## Structural Verification

| Verification | Command | Result |
|---|---|---|
| All 3 future-fixture runes REMOVED | `grep -cE "rune:purgare\(future-fixture\)" src/argspec/parse.rs src/argspec/error.rs` | **0** — parse.rs: 0; error.rs: 0 |
| `_options` un-prefixed to `options` | `grep -n "_options:" src/argspec/parse.rs` | **no matches** |
| `parse_triple` helper present (PRIVATE) | `grep -n "fn parse_triple" src/argspec/parse.rs` | **1 match** — line 132; no `pub` prefix |
| Rest-binder branch present in canonical | `grep -n "rest_param: Some" src/argspec/parse.rs` | **1 match** — line 106 |
| A4 returns ArgSpec | `grep -A 4 "^fn parse_defclause_args" src/runtime.rs \| grep "ArgSpec"` | **match present** — `Result<crate::argspec::ArgSpec, RuntimeError>` |
| Defclause sets allow_rest_binder: true | `grep -n "allow_rest_binder" src/runtime.rs` | **line 6842: true** (A4); line 6815: false (A1 fn-form stays unchanged) |
| Gate 1 `#[ignore]` REMOVED | `awk '/gate_1_defclause_supports_rest_binder/{print prev}{prev=$0}' tests/probe_arc237_8b_defclause_arithmetic.rs \| grep -c "#\[ignore"` | **0** |
| Stone 241.2 + 241.3 probes UNCHANGED | `git diff tests/probe_arc241_stone2_fn_parser_migration.rs tests/probe_arc241_stone3_defclause_parser_migration.rs \| wc -l` | **0** |

---

## Migration Audit (per-file line deltas)

| File | Pre-stone | Post-stone | Delta |
|---|---|---|---|
| `src/argspec/parse.rs` | 167 | 188 | **+21** (parse_triple extracted; rest-binder branch; doc updates; -3 rune lines) |
| `src/argspec/error.rs` | 118 | 116 | **-2** (TrailingItems rune comment removed; doc updated) |
| `src/runtime.rs` | ~33,641 | ~33,663 | **+22** (A4 signature + doc; `rest_param` in Clause struct + construction; caller integration) |
| `tests/probe_arc241_stone1_argspec_canonical.rs` | 159 | ~253 | **+94** (6 new contracts + type-syntax notes; `// Note: inner type args` comments added) |
| `tests/probe_arc237_8b_defclause_arithmetic.rs` | (current) | (current) | **-1** (`#[ignore]` line removed) |
| **Net delta** | — | — | **~+134 lines** (vs DESIGN estimate of ~+124; within calibration) |

---

## Final Post-Stone Code Shapes

### `parse_triple` (verbatim)

```rust
/// Parse a single `name <- :T` triple. The caller ensures `slice.len() >= 3`
/// before calling. Returns `(name, ty)` on success; the relevant
/// `ArgSpecError` variant on the per-slot failures (NameNotSymbol,
/// MissingArrow, TypeNotKeyword, MalformedTypeKeyword via parse_keyword_type).
fn parse_triple(
    slice: &[WatAST],
    head: &str,
) -> Result<(String, TypeExpr), ArgSpecError> {
    let name = match &slice[0] {
        WatAST::Symbol(ident, _) => ident.name.clone(),
        other => return Err(ArgSpecError::NameNotSymbol {
            span: other.span().clone(),
            head: head.to_string(),
        }),
    };
    if !is_bare_symbol(&slice[1], "<-") {
        return Err(ArgSpecError::MissingArrow {
            span: slice[1].span().clone(),
            head: head.to_string(),
        });
    }
    let ty = parse_keyword_type(&slice[2], head, |span, head| {
        ArgSpecError::TypeNotKeyword { span, head }
    })?;
    Ok((name, ty))
}
```

### Rest-binder branch in `parse_argspec_triples` (verbatim)

```rust
if is_bare_symbol(&args_vec[idx], "&") {
    if !options.allow_rest_binder {
        return Err(ArgSpecError::RestBinderNotSupported {
            span: args_vec[idx].span().clone(),
            head: head.to_string(),
        });
    }
    idx += 1; // consume `&`
    if args_vec.len().saturating_sub(idx) < 3 {
        return Err(ArgSpecError::IncompleteTriple {
            span: form_span.clone(),
            head: head.to_string(),
        });
    }
    let (name, ty) = parse_triple(&args_vec[idx..idx + 3], head)?;
    let post_rest = idx + 3;
    if post_rest < args_vec.len() {
        return Err(ArgSpecError::TrailingItems {
            span: form_span.clone(),
            head: head.to_string(),
            count: args_vec.len() - post_rest,
        });
    }
    return Ok(ArgSpec {
        fixed_params,
        rest_param: Some((name, ty)),
    });
}
```

### A4 final body (verbatim)

```rust
/// Parse the args-vector `[name <- :T ... [& rest <- :T]]` from a defclause clause.
///
/// Returns the canonical `ArgSpec` (fixed_params + rest_param) for the defclause's
/// argspec; rest-binder allowed at defclause sites per Stone 241.4.
///
/// Routes through the canonical `parse_argspec_triples`; `?` converts
/// `ArgSpecError → RuntimeError` via `From<ArgSpecError> for RuntimeError`
/// (Stone 241.1.fix).
fn parse_defclause_args(
    args_vec: &[WatAST],
    head: &str,
    form_span: &Span,
) -> Result<crate::argspec::ArgSpec, RuntimeError> {
    let spec = crate::argspec::parse_argspec_triples(
        args_vec,
        head,
        form_span,
        crate::argspec::ParseOptions { allow_rest_binder: true },
    )?;
    Ok(spec)
}
```

### Clause struct integration shape (verbatim delta)

```rust
pub struct Clause {
    /// Parallel vectors: binding names and declared types.
    pub args: Vec<(String, crate::types::TypeExpr)>,
    /// Stone 241.4 — Optional rest-binder `(name, type)` from `& name <- :T`
    /// in the clause argspec. `None` when no rest-binder is present.
    pub rest_param: Option<(String, crate::types::TypeExpr)>,
    // ... return_type, guard, ensure_fn, body unchanged
}
```

Caller (`parse_defclause_clause`) integration:

```rust
let spec = parse_defclause_args(args_vec, head, &form_span)?;
let args = spec.fixed_params;
let rest_param = spec.rest_param;
// ...
Ok(Clause {
    args,
    rest_param,
    return_type,
    // ...
})
```

---

## Test-Assertion Changes Inventory

**Probe 241.1 (canonical):** Contracts 10–15 are new contracts (not updated from existing). Type-syntax corrected during authorship: inner compound type args use bare symbols per the type system (`Vector<wat::core::i64>` not `Vector<:wat::core::i64>`). This was a design-time error in the pre-spawn probe that surfaced at test-run (contracts 10–12 failed with `MalformedTypeKeyword { inner: InnerColonInCompoundArg }`). Fixed in the probe source by removing the illegal inner colons. The Gate 1 source already used correct syntax (`Vector<wat::core::i64>`); the canonical probe now matches. Added `// Note: inner type args use bare symbols (no leading colon) per the type system.` to each affected contract for reader orientation.

**Lib tests:** Zero cascade. No existing lib test asserted against A4's old inline error messages or defclause argspec signatures. Mirrors the zero-cascade calibration from Stones 241.2 and 241.3.

**Stone 241.2/241.3 probes:** UNCHANGED (verified via `git diff`).

---

## Honest Deltas

### 1 — Gate 1 remains RED: STOP-6 (runtime dispatch depth exceeds 30 lines)

**Finding:** After the parser ships `rest_param` into `Clause`, the dispatch layer (`eval_clause_set` at runtime.rs ~7220) does not yet wire rest-binder to variadic dispatch. Specifically:

- **Arity check (line 7232):** `declared_arity != called_arity` rejects 4-arg calls to a 1-fixed-param rest-binder clause (`attempted_clauses: [(1, [":wat::core::i64"])]`).
- **Argument binding (line 7279):** Only iterates `clause.args` — does not collect remaining vals into a `Vector` bound to `rest_param.name`.
- **Type-check (line 7248):** Only checks the fixed-param positions against the declared types — does not verify rest args against `rest_param.ty`.
- **Check layer:** `check.rs` call-site validation (`NoMatchingClauseAtCallSite`) would also need updating for variadic arity.

Total integration depth: ~40-60 lines across dispatch + binding + type-check + check layer. STOP-6 criterion exceeded. **Stone 241.4 ships parser-level + structural Clause storage; runtime dispatch wiring belongs to a follow-up stone.**

**Gate 1 status after Stone 241.4:** RED (dispatch not wired). Gate 1 un-ignored and running — the failure message is honest: `NoMatchingClauseAtCallSite { called_arity: 4, attempted_clauses: [(1, [":wat::core::i64"])] }`. The substrate gap is now precisely named.

**Follow-up:** Stone 241.5 or a dedicated arc to wire rest-binder into dispatch. Scope: arity check (allow `called_arity >= fixed_arity` when `rest_param.is_some()`), collect trailing vals as `Value::Vector`, bind to `rest_param.name`, type-check rest elements, update check layer.

### 2 — Probe type-syntax correction (inner colon)

Contracts 10–12 were authored with `Vector<:wat::core::i64>` (inner colon). The type parser's `parse_type_inner` rejects inner colons (`InnerColonInCompoundArg`). The correct inner syntax is `Vector<wat::core::i64>` (bare symbol). BRIEF's probe was written with incorrect syntax; corrected at test-run. The Gate 1 source already used correct syntax. Contract_07 (PASS at HEAD) never triggered the type parser because the `&` check fired first (RestBinderNotSupported before type parsing).

### 3 — `rest` naming collision avoided

In `parse_defclause_clause`, the local `let rest = &items[1..]` (items tail for clause body scanning) was already bound before the `parse_defclause_args` extraction. Named the extracted `spec.rest_param` as `rest_param` (not `rest`) to avoid shadowing. Correct shape; no semantic gap.

### 4 — Clause struct construction: one site only

`grep -n "Clause {" src/runtime.rs` yields exactly one construction site (line 7073). Zero cascade to other Clause construction callers.

### 5 — Zero lib test cascade

Third consecutive stone (241.2, 241.3, 241.4) with zero lib test cascade. The pattern is confirmed: existing lib tests assert at the behavioral (ok/err) boundary, not on message text or return shape.

---

## Cascade Depth

**SHALLOW for argspec layer; SURFACED GAP for dispatch layer.**

The parser extension + helper extract + rune removals + A4 signature + Clause field addition all shipped with zero test cascade beyond the explicit new contracts in the canonical probe. The dispatch gap is precise and bounded: the Clause struct carries `rest_param`; the dispatch loop ignores it. The gap is NAMED (STOP-6, Gate 1 RED); the follow-up is mechanical wiring against a settled storage foundation.

---

## NO Vigilia Convergence Section

Orchestrator casts Phase B vigilia on `src/argspec/*` + extended canonical probe before commit. This section will be inscribed by the orchestrator after Phase B re-cast achieves L1+L2=0.

---

## Vigilia L2 Closure (Phase B-amend, 2026-05-28 evening; orchestrator-inscribed)

Phase B vigilia cast on Stone 241.4 returned 0 L1 + 4 L2 (intueri + solvere + 2× struere; details in vigilia returns). All 4 addressed via this amend:

- L2.1 intueri: mod.rs migration plan updated to past-tense for shipped stones
- L2.2 struere: parse_triple now takes &[WatAST; 3] (type-enforces precondition)
- L2.3 struere: rest-binder branch uses rest_start binding (idx arithmetic local)
- L2.4 solvere: parse_defclause_args wrapper DELETED; inline at parse_defclause_clause

Plus L3 complectens taste: probe file header doc updated 9 → 15 contracts.

---

## Vigilia Convergence (Phase B, 2026-05-28 evening; orchestrator-inscribed)

Per `feedback_namespaced_home_vigilia_gate`: vigilia cast post-SCORE on `src/argspec/*` + extended canonical probe. Eight spells in parallel (intueri / solvere / purgare / struere / sequi / temperare / complectens / vocare). The four spells with findings were re-cast after the L2 closure amend. One mid-cast factual error in the L2-closure amend (mod.rs migration plan parser-to-stone mapping swapped) was caught at the score function, sharper-respawned, and re-verified.

### Aggregate: CONVERGED — 0 L1 + 0 L2

The gate doctrine bar (L1+L2=0 on namespaced wat-rs homes) is satisfied. Stone 241.4 closes.

### Per-spell verdicts (final state, post-L2-closure)

| Spell | Initial verdict | L2-closure address | Final verdict |
|---|---|---|---|
| **intueri** | 0 L1 + 1 L2 (mod.rs migration plan future-tense for shipped) | Rewrote migration plan past-tense; first amend swapped 241.2/241.3 mapping; orchestrator caught at score function (per `feedback_sonnet_writes_substrate` discipline); sharper respawn corrected; re-cast verified ground-truth match | CONVERGED 0 L1 + 0 L2 |
| **solvere** | 0 L1 + 1 L2 (parse_defclause_args wrapper braided thin) | Deleted wrapper; inlined `parse_argspec_triples` call at `parse_defclause_clause` (matches A1/A2/A3 inline pattern) | CONVERGED 0 L1 + 0 L2 |
| **purgare** | 0 L1 + 0 L2 (zero runes; all 3 future-fixture runes correctly retired per Stone 241.4's prediction-truth-condition holding) | — | CONVERGED (initial) |
| **struere** | 0 L1 + 2 L2 (parse_triple unchecked slice — panic possible on third caller forgetting gate; idx arithmetic non-local in rest-binder branch) | Changed `parse_triple` signature to `&[WatAST; 3]` (type-enforces precondition; `try_into().expect("len gated by upstream check")` at call sites); added `let rest_start = idx;` binding after `idx += 1` | CONVERGED 0 L1 + 0 L2 |
| **sequi** | 0 L1 + 0 L2 (state threads end-to-end through types; no globals/statics/OnceLock) | — | CONVERGED (initial) |
| **temperare** | 0 L1 + 0 L2 (parse_triple called at most once per iteration across mutually-exclusive branches; no redundant work) | — | CONVERGED (initial) |
| **complectens** | 0 L1 + 0 L2 (2 L3 taste: file-header doc said "9 contracts" — now 15; parse_vector_items thin wrapper acceptable per SKILL.md) | Probe file-header doc updated to "15 contracts" + post-stone history line added | CONVERGED + L3 addressed |
| **vocare** | 0 L1 + 0 L2 (all 15 contracts at canonical caller vantage; FM 2-bis substrate-internal probe exemption holds) | — | CONVERGED (initial) |

### Runes accepted: ZERO

Stone 241.4 is the future-fixture rune retirement event. All three `rune:purgare(future-fixture)` markers from Stone 241.1.fix retired (rest_param field + rest_param: None initializer + TrailingItems variant). The argspec home is rune-free post-241.4 — the future is the present.

### Sonnet calibration miss caught at score (worth documenting)

The L2 closure amend (~4 min Mode A) introduced a factual error: sonnet rewrote the mod.rs migration plan with WRONG parser-to-stone mapping (`241.2 → A1; 241.3 → A2/A3`). The actual mapping per git history is `241.2 → A1+A2+A3; 241.3 → A4`. The error cause: orchestrator's brief used the slash notation "A1/A2/A3 migration" which sonnet parsed as "A1 [singular] migration"; intueri re-cast quoted the brief incorrectly back to me and verified the WRONG mapping as correct.

The orchestrator caught it by cross-checking against `git log --format="%B" -1 21877135` (Stone 241.2 commit message). Sharper sonnet respawned with verbatim correction text + git verification command. Re-cast intueri verified the correction landed.

**Calibration lesson** for future briefs: use `A1+A2+A3` (additive notation) instead of `A1/A2/A3` (slash, ambiguous between singular-alternative and plural-set). Inscribed here for future self.

### Cross-spell convergence on amend impact

- **Three future-fixture runes RETIRED** — the prediction's truth condition (Stone 241.4 ships rest-binder) holds; the runes are obsolete; purgare confirmed zero runes left
- **parse_triple type-enforcement** — `&[WatAST; 3]` makes the precondition impossible to violate; struere converged structurally rather than dynamically
- **parse_defclause_args wrapper deletion** — A4 had become thin braid post-241.4 (pure forwarding); inlining matches A1/A2/A3 pattern; solvere converged via wrapper removal
- **Doc-text accuracy** — mod.rs migration plan factually correct against git ground truth; intueri converged via re-cast verification

### Verdict: CONVERGED on the home; Stone 241.4 ready to commit

Per `feedback_namespaced_home_vigilia_gate`: commit-readiness requires L1+L2=0. Achieved across all 8 spells. The argspec home is **shockingly good, remarkably well written** — user direction met.

### Stone 241.5 named follow-up (per FM 11 — deferrals MUST name the follow-up)

Stone 241.4 ships parser + storage (canonical `&` rest-binder + ArgSpec.rest_param + Clause.rest_param threading). Runtime dispatch wiring in `eval_clause_set` (~40-60 lines per Stone 241.4 SCORE STOP-6 surface) deferred to **Stone 241.5**. probe_arc237_8b's `gate_1_defclause_supports_rest_binder` re-ignored with named-Stone-241.5-follow-up; un-ignore when 241.5 ships.

Per the four-questions verdict (Path A): two coherent stones, two coherent commits. Argspec home gets its vigilia-gated commit; eval_clause_set dispatch gets its own commit on SCORE-green (legacy flat substrate; gate doesn't apply there).

### Phase 1 capstone — argspec parser shape complete

| Future-fixture marker | Stone 241.1.fix inscribed | Stone 241.4 retired |
|---|---|---|
| `rest_param` field | rune:purgare(future-fixture) | RETIRED (field actively populated when allow_rest_binder fires) |
| `rest_param: None` initializer | rune:purgare(future-fixture) | RETIRED (no-rest case is natural; not future-fixture) |
| `TrailingItems` variant | rune:purgare(future-fixture) | RETIRED (variant reachable; contract 12 verifies) |
| `_options` parameter prefix | (un-prefix deferred to 241.4) | UN-PREFIXED (`options` actively consulted) |

The canonical parser's first-release shape is complete. Stone 241.5 ships dispatch; arc 237.8b Gate 1 then unblocks; arc 237.8b's remaining gates open.
