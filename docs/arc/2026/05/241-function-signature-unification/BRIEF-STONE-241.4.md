# BRIEF — Stone 241.4 — canonical `&` rest-binder extension + defclause opt-in; unblocks 237.8b Gate 1

You are sonnet (the Shadowdancer). Phase 1 capstone. The canonical `parse_argspec_triples` shipped vigilia-CONVERGED with `allow_rest_binder` as a per-site flag; this stone wires the `true` path; three future-fixture runes retire; defclause opts in; probe 237.8b Gate 1 flips green.

## What to do

### S1 — Canonical parser extension (`src/argspec/parse.rs`)

Un-prefix `_options: ParseOptions` → `options: ParseOptions` (line 73). Replace the current `&` branch body (lines 81-88) with the conditional + rest-binder triple parse:

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

### S2 — Extract `parse_triple` helper (struere — anticipate vigilia)

Both the fixed-param walker AND the rest-binder branch parse the same `name <- :T` shape. Extract a private helper:

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

Replace the fixed-param loop body's inline triple parse (current lines 90-122) with:

```rust
if args_vec.len().saturating_sub(idx) < 3 {
    return Err(ArgSpecError::IncompleteTriple {
        span: form_span.clone(),
        head: head.to_string(),
    });
}
let (name, ty) = parse_triple(&args_vec[idx..idx + 3], head)?;
fixed_params.push((name, ty));
idx += 3;
```

Place `parse_triple` between `parse_argspec_triples` and `parse_keyword_type` (logical order: main fn → triple helper → type helper → token predicate).

### S3 — Remove 3 future-fixture runes (their prediction's truth condition holds)

| File | Site | Action |
|---|---|---|
| `src/argspec/parse.rs:18-19` | `rune:purgare(future-fixture)` on `rest_param` field | DELETE both rune comment lines; field is now actively populated |
| `src/argspec/parse.rs:127-128` | `rune:purgare(future-fixture)` on `rest_param: None` initializer at no-rest Ok branch | DELETE both rune comment lines; this branch is the natural no-rest case |
| `src/argspec/error.rs:32-33` | `rune:purgare(future-fixture)` on `TrailingItems` variant | DELETE both rune comment lines; variant becomes reachable (contract_12 verifies) |

Net rune count post-stone: 0 in `src/argspec/*`. The future is the present.

### S4 — Update doc comments (intueri — anticipate vigilia)

`src/argspec/parse.rs`:

1. `ArgSpec.rest_param` doc (currently lines 16-17): rewrite to *"Rest parameter `(name, type)`, populated when `options.allow_rest_binder = true` AND the source includes `& name <- :T`. Otherwise `None`."* — remove "Always `None` in Stone 241.1" framing.

2. `ParseOptions.allow_rest_binder` doc (currently lines 30-37): rewrite to *"When `true`, the canonical parser parses `& name <- :T` as a rest-binder, populating `ArgSpec.rest_param`. When `false`, encountering `&` returns `ArgSpecError::RestBinderNotSupported`. `defclause` sites set this `true`; `defn`/fn-form sites set this `false`."* — remove "Stone 241.4 wires this field" future-tense.

3. `parse_argspec_triples` doc Algorithm section (currently lines 63-68): add step 3 *"On `&` (rest-marker): if `options.allow_rest_binder`, parse the following triple as the rest-binder and verify no trailing items; else return `RestBinderNotSupported`."* Update step 2's text — remove the "always reject `&` in Stone 241.1" wording.

### S5 — Defclause caller opts in (`src/runtime.rs:6827` `parse_defclause_args`)

A4's body becomes (signature changes — see D6 of DESIGN):

```rust
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

Return type changes: `Vec<(String, TypeExpr)>` → `crate::argspec::ArgSpec`. The function name stays. Caller (`parse_defclause_clause` at runtime.rs:6947) consumes `spec.fixed_params` AND `spec.rest_param`.

Update doc comment above `parse_defclause_args` to reflect the new shape — *"Returns the canonical `ArgSpec` (fixed_params + rest_param) for the defclause's argspec; rest-binder allowed at defclause sites per Stone 241.4."*

### S6 — `parse_defclause_clause` integrates rest_param

Caller at `src/runtime.rs:6947` currently does:

```rust
let args = parse_defclause_args(args_vec, head, &form_span)?;
// ... uses args as Vec<(String, TypeExpr)>
```

After Stone 241.4, change to:

```rust
let spec = parse_defclause_args(args_vec, head, &form_span)?;
let args = spec.fixed_params;
let rest = spec.rest_param;
// ... thread rest into Clause's runtime structure
```

The Clause struct (also defined in src/runtime.rs around defclause infrastructure) likely needs a `rest_param: Option<(String, TypeExpr)>` field. Sonnet investigates:

- Find Clause struct (grep `struct Clause`)
- If it has rest_param: thread the value through
- If not: ADD `rest_param: Option<(String, TypeExpr)>` field; default to None where Clause is constructed; populate from `spec.rest_param` at parse_defclause_clause

**Scope check — STOP-6 trigger**: if integrating `rest_param` into Clause's RUNTIME DISPATCH (how clauses match arguments at runtime; how variadic args are bundled into Vector; type-checking of rest-binder; etc.) requires more than ~30 lines of mechanical wiring, SURFACE AS FINDING. Stone 241.4's scope is parser-level + structural-clause-storage; deeper runtime integration may belong to a follow-up stone.

The probe 237.8b Gate 1 (S7 below) is the load-bearing integration test: if Gate 1 passes after S5+S6, integration is sufficient for THIS stone. If Gate 1 still fails because runtime dispatch isn't wired, surface the gap.

### S7 — Unignore probe 237.8b Gate 1

In `tests/probe_arc237_8b_defclause_arithmetic.rs:85`, REMOVE the `#[ignore = "Stone 237.8b-prep: defclause `&` rest-binder support must be minted FIRST (substrate gap surfaced 2026-05-27); un-ignore after extension lands"]` line.

The test should now compile + RUN green. If it fails, surface the gap (STOP-10).

## Discipline

- **`src/argspec/mod.rs` UNCHANGED.** (Re-exports stay; no new public surface.)
- **`src/lib.rs` UNCHANGED.**
- **`src/check.rs` UNCHANGED.** (Fn-form parsers A1/A2/A3 don't change; they still set `allow_rest_binder: false`.)
- **A1/A2/A3 fn-form parsers UNCHANGED** (their `ParseOptions { allow_rest_binder: false }` stays — defn/fn sites NEVER want rest-binder).
- **Stone 241.2 + 241.3 probes UNCHANGED.** They test behaviors that Stone 241.4 preserves.
- **Stone 241.1 canonical probe EXTENDED** (already done by orchestrator; 9 → 15 contracts; verify the FIVE NEW happy-path/error-path contracts (10–14) flip green and contract_15 regression stays green).
- **A4's public signature CHANGES** (return type Vec → ArgSpec). Only A4 has this signature change; A1/A2/A3 stay (Vec<String>, Vec<TypeExpr>, TypeExpr) tuple.
- **3 runes RETIRE** (rest_param field; rest_param: None initializer; TrailingItems variant). DELETE the comment lines; do not preserve.
- **Vigilia gate APPLIES** to src/argspec/*. Sonnet ships SCORE-green; orchestrator casts vigilia Phase B before commit.
- **No `cargo run`; no wrapper scripts; just `cargo test/build/clippy`.**

## Test-assertion cascade likely larger than 241.2/241.3 (T9 from DESIGN)

A4 signature change may break tests that called A4 directly (private; should be zero); defclause tests previously asserting RestBinderNotSupported on `&` may now succeed; probe 237.8b Gate 1 is explicitly being unignored.

Per `docs/SUBSTRATE-AS-TEACHER.md`: substrate diagnostic cascade IS the migration brief. Iterate; document each update as HONEST DELTA.

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md` — FM catalog
2. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.4.md` — this doc
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.4.md` — D1-D10 + T1-T10 + STOP
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.1.fix.md` § Vigilia Convergence — canonical foundation; the three runes Stone 241.4 retires
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.3.md` — Phase 1 closure inscription; A4's current Vec-returning shape
6. `/home/watmin/work/holon/wat-rs/src/argspec/parse.rs` — current canonical body (struere extraction target)
7. `/home/watmin/work/holon/wat-rs/src/argspec/error.rs` — current variant list (TrailingItems rune to remove)
8. `/home/watmin/work/holon/wat-rs/src/runtime.rs` lines 6820-7000 — A4 + parse_defclause_clause + Clause-struct context
9. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone1_argspec_canonical.rs` — 15-contract canonical probe (10 PASS / 5 FAIL at HEAD; post-stone 15/15)
10. `/home/watmin/work/holon/wat-rs/tests/probe_arc237_8b_defclause_arithmetic.rs` lines 70-110 — Gate 1 source (currently `#[ignore]`'d)
11. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/EXPECTATIONS-STONE-241.4.md` — scorecard

## Implementation sketch

1. Read substrate + probes + DESIGN
2. Baseline check:
   - `cargo test --release --lib -p wat` (expect 834 PASS)
   - `cargo test --release --test probe_arc241_stone1_argspec_canonical` (expect 10 PASS / 5 FAIL at HEAD)
   - `cargo test --release --test probe_arc241_stone3_defclause_parser_migration` (expect 6 PASS)
3. **S1 + S2 + S3 + S4 (argspec/parse.rs)**: extract `parse_triple`; replace fixed-param inline parse; replace `&`-branch with conditional + rest-binder parse; remove rest_param field rune; remove rest_param: None initializer rune; update doc comments
4. **S3 (argspec/error.rs)**: remove TrailingItems rune comment lines
5. Run `cargo test --release --test probe_arc241_stone1_argspec_canonical` — expect 15/15 PASS
6. **S5 (runtime.rs A4)**: change A4 to return `ArgSpec`; set `allow_rest_binder: true`
7. **S6 (runtime.rs parse_defclause_clause + Clause struct)**: consume `spec.rest_param`; add Clause.rest_param field if needed; thread through
8. Run `cargo test --release --lib -p wat` — identify failing tests; update test assertions if they assert on old A4 return shape or on `RestBinderNotSupported` for defclause `&`
9. **S7 (probe_arc237_8b)**: remove `#[ignore]` from Gate 1; run it
10. Final verification:
    - `cargo test --release --lib -p wat` (≥834 PASS)
    - `cargo test --release --test probe_arc241_stone1_argspec_canonical` (15/15)
    - `cargo test --release --test probe_arc241_stone2_fn_parser_migration` (10/10 preserved)
    - `cargo test --release --test probe_arc241_stone3_defclause_parser_migration` (6/6 preserved)
    - `cargo test --release --test probe_arc237_8b_defclause_arithmetic gate_1` (passes; Gate 1 unblocked)
    - `cargo build --release --tests --workspace` (clean)
    - `cargo clippy --release` (≤ 905)
11. Write `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.4.md`
12. **DO NOT COMMIT.** Orchestrator commits after Phase B vigilia cast (gate applies; src/argspec/ touched).

## STOP triggers — REJECTION

1. **STOP-1** — Unexpected compile errors not traced to the changes
2. **STOP-2** — Lib baseline regression < 834 (after assertion updates)
3. **STOP-3** — 60 min elapsed
4. **STOP-4** — `holon-rs` touched
5. **STOP-5** — Files outside `src/argspec/{parse,error}.rs`, `src/runtime.rs`, `tests/probe_arc241_stone1_argspec_canonical.rs`, `tests/probe_arc237_8b_defclause_arithmetic.rs`, SCORE doc + test files with assertion updates. `src/lib.rs` + `src/argspec/mod.rs` + `src/check.rs` MUST stay unchanged.
6. **STOP-6** — Scope creep: Clause runtime dispatch >30 lines mechanical wiring (surface as future stone); A1/A2/A3 fn-form parsers touched; new ArgSpecError variants or ParseOptions fields; type-keyword helper unification
7. **STOP-7** — Extended canonical probe < 15/15 PASS
8. **STOP-8** — Stone 241.2/241.3/arc-237/arc-238 probes regress; OR Gate 1 doesn't pass after S7
9. **STOP-9** — Clippy > 905
10. **STOP-10** — Vigilia Phase B DIVERGES with findings sonnet can't quickly address (re-brief Stone 241.4.fix); or substrate change doesn't satisfy Gate 1 (deeper integration needed; surface)

## SCORE doc spec

Mirror SCORE-STONE-241.1.fix.md (since vigilia applies). Include:

- Header (Mode A/B; runtime; one-line summary)
- 15-row Phase A scorecard with verbatim results
- 10-row structural verification (runes removed; parse_triple present; rest-binder branch present; A4 returns ArgSpec; defclause opt-in landed; Gate 1 unignored + passing)
- Migration audit (per-file line deltas)
- Final post-stone code shapes (parse_triple verbatim; rest-binder branch verbatim; A4 final body)
- Test-assertion changes inventory
- Honest deltas (Clause integration depth; doc updates; Gate 1 status)
- Cascade depth note
- **NO Vigilia Convergence section** — orchestrator inscribes after Phase B re-cast

## Post-strike

Return with a one-paragraph status summary covering: argspec extension shipped, A4 signature evolved, defclause opt-in landed, Gate 1 status (green or surfaced gap), probe 15/15.

Phase 1 capstone. The runes retire; the future is the present. Strike clean.
