# BRIEF — Stone 241.1.fix — vigilia-convergence + scope correction on `src/argspec/*`

You are sonnet (the Shadowdancer). You strike on a two-layer amend pass against the home Stone 241.1 just laid. Layer 1 (vigilia amends) is already on disk uncommitted from a prior strike — KEEP those changes. Layer 2 (scope correction) is NEW — strip ret-clause concerns out of argspec entirely.

The deeper context: Stone 241.1 was scope-confused (carried ret-type semantics in argspec per a stale AUDIT.md framing). Vigilia surfaced a solvere L2 (`RetTypeNotKeyword` conflation); the user verdict locked Path Y — *"args have nothing to do with ret type."* Argspec parses ONLY the canonical `[name <- :T name <- :T ... [& rest <- :T]]` triple form. Ret-clause (`-> :Ret`) belongs to fn-form parsers (defn, fn, etc.); those callers (Stone 241.2 territory) compose argspec + ret-clause at form level.

## What to do

### Layer 1 — Vigilia amends (already on disk; KEEP)

The substrate at HEAD has uncommitted changes from a prior strike implementing:
- `classify()` extraction on `ArgSpecError` (error.rs)
- `parse_keyword_type<F>` extraction (parse.rs)
- `// rune:purgare(future-fixture)` on `unreachable!` arm + `rest_param` field (parse.rs)
- Probe helper renames (`argspec_inputs` → `parse_vector_items`; `invoke` → `parse_triples`)
- Owned `(Vec<WatAST>, wat::span::Span)` return replacing opaque `impl Deref<Target=Span>`
- Tautological guard removal at parse.rs (post-loop ret-arrow check)
- `saturating_sub` rewrite at parse.rs:99
- WHAT-comment deletion at parse.rs:98

Verify these exist via `git diff src/argspec/ tests/probe_arc241_stone1_argspec_canonical.rs`. KEEP them all.

### Layer 2 — Scope correction (NEW; strip ret-clause concerns)

#### S1 — Remove `ret_type` field from `ArgSpec`

In `src/argspec/parse.rs`, delete the field from the struct definition:

```rust
// BEFORE
pub struct ArgSpec {
    pub fixed_params: Vec<(String, TypeExpr)>,
    pub rest_param: Option<(String, TypeExpr)>,
    pub ret_type: Option<TypeExpr>,        // ← DELETE this line
}

// AFTER
pub struct ArgSpec {
    pub fixed_params: Vec<(String, TypeExpr)>,
    // rune:purgare(future-fixture) — Stone 241.4 populates rest_param via allow_rest_binder
    //                                path; field exists in 241.1 for API stability.
    pub rest_param: Option<(String, TypeExpr)>,
}
```

#### S2 — Remove `include_ret_type` from `ParseOptions`

```rust
// BEFORE
pub struct ParseOptions {
    pub include_ret_type: bool,           // ← DELETE
    pub allow_rest_binder: bool,
}

// AFTER
pub struct ParseOptions {
    /// Whether a `& name <- :T` rest-binder is permitted in the arg-vector.
    /// Always `false` in Stone 241.1. Stone 241.4 adds rest-binder logic;
    /// `defclause` callers set this `true` via 241.5.
    pub allow_rest_binder: bool,
}
```

#### S3 — Remove `MissingRetArrow` and `RetTypeNotKeyword` from `ArgSpecError`

In `src/argspec/error.rs`, delete the two variants from the enum. The final enum:

```rust
pub enum ArgSpecError {
    NameNotSymbol { span: Span, head: String },
    MissingArrow { span: Span, head: String },
    TypeNotKeyword { span: Span, head: String },
    MalformedTypeKeyword { span: Span, head: String, inner: Box<TypeError> },
    TrailingItems { span: Span, head: String, count: usize },
    IncompleteTriple { span: Span, head: String },       // ← renamed from IncompleteSignature
    RestBinderNotSupported { span: Span, head: String },
}
```

Also: **RENAME `IncompleteSignature` → `IncompleteTriple`**. Per S3-rename rationale: "signature" implies fn-form (which we just stripped); "triple" is honest about what this parses.

#### S4 — Update `classify()` arms

Remove arms for `MissingRetArrow` and `RetTypeNotKeyword`. Rename `IncompleteSignature` arm to `IncompleteTriple`. Final 7 arms:

```rust
impl ArgSpecError {
    fn classify(self) -> (Span, String, String) {
        match self {
            ArgSpecError::NameNotSymbol { span, head } => (
                span, head,
                "name slot must be a plain symbol (not a keyword, literal, or nested form)".into(),
            ),
            ArgSpecError::MissingArrow { span, head } => (
                span, head,
                "triple must be `name <- :T`; `<-` arrow not found at slot 1".into(),
            ),
            ArgSpecError::TypeNotKeyword { span, head } => (
                span, head,
                "type slot must be a keyword (e.g. `:wat::core::i64`); got a non-keyword".into(),
            ),
            ArgSpecError::MalformedTypeKeyword { span, head, inner } => (
                span, head,
                format!("type keyword is malformed: {inner}"),
            ),
            ArgSpecError::TrailingItems { span, head, count } => (
                span, head,
                format!("{count} trailing item(s) beyond the expected argspec shape"),
            ),
            ArgSpecError::IncompleteTriple { span, head } => (
                span, head,
                "triple is incomplete; expected `name <- :T` but ran out of items".into(),
            ),
            ArgSpecError::RestBinderNotSupported { span, head } => (
                span, head,
                "`&` rest-binder is not supported at this binding site".into(),
            ),
        }
    }
}
```

#### S5 — Strip ret-clause logic from `parse_argspec_triples` body

In `src/argspec/parse.rs`, remove the entire `if options.include_ret_type {...}` post-loop block. The function body becomes:

```rust
pub fn parse_argspec_triples(
    args_vec: &[WatAST],
    head: &str,
    form_span: &Span,
    options: ParseOptions,
) -> Result<ArgSpec, ArgSpecError> {
    let mut idx = 0usize;
    let mut fixed_params: Vec<(String, TypeExpr)> = Vec::new();

    // Walk triples (name <- :T) until rest-marker `&` or end-of-slice.
    while idx < args_vec.len() {
        // Check for `&` rest-marker.
        if is_bare_symbol(&args_vec[idx], "&") {
            if !options.allow_rest_binder {
                return Err(ArgSpecError::RestBinderNotSupported {
                    span: args_vec[idx].span().clone(),
                    head: head.to_string(),
                });
            }
            // rune:purgare(future-fixture) — Stone 241.4 ships allow_rest_binder=true
            //                                logic; 241.1 path unreachable by design.
            unreachable!("allow_rest_binder is always false in Stone 241.1");
        }

        // Check remaining items can form a triple.
        if args_vec.len().saturating_sub(idx) < 3 {
            return Err(ArgSpecError::IncompleteTriple {
                span: form_span.clone(),
                head: head.to_string(),
            });
        }

        // Slot 0: name — must be a Symbol.
        let name = match &args_vec[idx] {
            WatAST::Symbol(ident, _) => ident.name.clone(),
            other => {
                return Err(ArgSpecError::NameNotSymbol {
                    span: other.span().clone(),
                    head: head.to_string(),
                })
            }
        };

        // Slot 1: arrow — must be bare Symbol "<-".
        if !is_bare_symbol(&args_vec[idx + 1], "<-") {
            return Err(ArgSpecError::MissingArrow {
                span: args_vec[idx + 1].span().clone(),
                head: head.to_string(),
            });
        }

        // Slot 2: type — route through parse_keyword_type with the fixed-param error ctor.
        let ty = parse_keyword_type(&args_vec[idx + 2], head, |span, head| {
            ArgSpecError::TypeNotKeyword { span, head }
        })?;

        fixed_params.push((name, ty));
        idx += 3;
    }

    Ok(ArgSpec {
        fixed_params,
        // rune:purgare(future-fixture) — Stone 241.4 populates rest_param via
        //                                allow_rest_binder path; 241.1 always None.
        rest_param: None,
    })
}
```

**Key changes from current**:
- The break on `is_bare_symbol(args_vec[idx], "->")` GOES AWAY (no ret-arrow recognition in argspec)
- The entire post-loop `if options.include_ret_type {...}` block GOES AWAY
- The final `if idx < args_vec.len() { TrailingItems }` check GOES AWAY (loop consumes full slice unless rest-marker fires; until Stone 241.4 ships rest-binder logic, the loop walks to end)
- The `ArgSpec` construction drops `ret_type` field
- `IncompleteSignature` → `IncompleteTriple`

#### S6 — Update doc comments

`src/argspec/parse.rs` doc on `ArgSpec`: remove ret_type mentions. Doc on `parse_argspec_triples`: remove ret-related algorithm steps; clarify scope.

`src/argspec/mod.rs` module doc: strip ret-related framing. Add:

```rust
//! ## Scope
//!
//! Argspec parses ONLY the canonical `[name <- :T name <- :T ... [& rest <- :T]]`
//! triple form. The ret-clause (`-> :Ret`) is NOT argspec's concern — fn-form parsers
//! (defn, fn, fn type-signature) compose argspec + ret-clause at the form level.
//! Per `FORM-COLLAPSE-NOTES.md` line 184:
//!
//! > Arc 241's `parse_argspec_triples` parses the canonical 3-slot triple uniformly
//! > across all binding sites. Form-level parsers decode the per-binding metadata map
//! > separately and associate by symbol.
```

#### S7 — Strip ret-related contracts from probe

In `tests/probe_arc241_stone1_argspec_canonical.rs`, DELETE these contracts:
- `contract_03_multiple_fixed_params_with_ret` (current 03 — used `include_ret_type=true`)
- `contract_04_ret_only_signature` (current 04 — ret-only is no longer an argspec concept)
- `contract_08_missing_ret_arrow_when_expected` (current 08 — MissingRetArrow gone)
- `contract_09_trailing_items_after_ret` (current 09 — TrailingItems uses different semantics now)
- `contract_12_ret_type_not_keyword` (current 12 — RetTypeNotKeyword gone)

REPLACE current `contract_03` with a NEW contract testing multiple fixed params WITHOUT ret:

```rust
#[test]
fn contract_03_multiple_fixed_params() {
    // [x <- :wat::core::i64 y <- :wat::core::i64] — two fixed params, no ret.
    let result = parse_triples("[x <- :wat::core::i64 y <- :wat::core::i64]", false);
    let spec = result.expect("multi-param argspec parses cleanly");
    assert_eq!(spec.fixed_params.len(), 2, "two fixed params");
    assert_eq!(spec.fixed_params[0].0, "x", "first name is 'x'");
    assert_eq!(spec.fixed_params[1].0, "y", "second name is 'y'");
    assert!(spec.rest_param.is_none(), "rest_param should be None");
}
```

REMAINING contracts after deletion + replacement (renumber to 01-08):
| New # | Test | Source form |
|---|---|---|
| 01 | empty argspec | `[]` |
| 02 | single fixed param | `[x <- :wat::core::i64]` |
| 03 | multiple fixed params (replacement) | `[x <- :wat::core::i64 y <- :wat::core::i64]` |
| 04 | non-Symbol at name slot | `[:keyword-not-symbol <- :wat::core::i64]` |
| 05 | missing `<-` arrow | `[x = :wat::core::i64]` |
| 06 | non-Keyword at type slot | `[x <- "string-not-keyword"]` |
| 07 | `&` rest-marker rejected | `[x <- :wat::core::i64 & rest <- :wat::core::Vector<:wat::core::i64>]` |
| 08 | malformed type keyword (`:Any`) | `[x <- :Any]` |

Rename:
- Current contract_05 → contract_04 (non-Symbol name)
- Current contract_06 → contract_05 (missing arrow)
- Current contract_07 → contract_06 (non-keyword type)
- Current contract_10 → contract_07 (rest-binder rejected)
- Current contract_11 → contract_08 (malformed type keyword)
- Current contract_13 → DELETE? No — `IncompleteTriple` is a renamed variant; the contract should test the renamed variant.

Wait — `contract_13_incomplete_signature` tests `[x <-]` which triggers `IncompleteSignature` (being renamed to `IncompleteTriple`). KEEP this contract; rename to `contract_NN_incomplete_triple`; update the match arm.

Updated FINAL contract list (9 contracts):
| # | Test name | Source form | Variant |
|---|---|---|---|
| 01 | contract_01_empty_argspec | `[]` | (success) |
| 02 | contract_02_single_fixed_param | `[x <- :wat::core::i64]` | (success) |
| 03 | contract_03_multiple_fixed_params | `[x <- :wat::core::i64 y <- :wat::core::i64]` | (success) |
| 04 | contract_04_non_symbol_at_name_slot | `[:kw <- :wat::core::i64]` | NameNotSymbol |
| 05 | contract_05_missing_arrow_token | `[x = :wat::core::i64]` | MissingArrow |
| 06 | contract_06_non_keyword_at_type_slot | `[x <- "string"]` | TypeNotKeyword |
| 07 | contract_07_rest_binder_rejected | `[x <- :i64 & rest <- :Vector<:i64>]` | RestBinderNotSupported |
| 08 | contract_08_malformed_type_keyword | `[x <- :Any]` | MalformedTypeKeyword |
| 09 | contract_09_incomplete_triple | `[x <-]` | IncompleteTriple |

So 9 contracts, not 8. The probe shrinks 13 → 9. (My earlier count missed IncompleteTriple.)

`parse_triples` helper signature simplifies (drop `include_ret_type` param):

```rust
fn parse_triples(
    src: &str,
    allow_rest_binder: bool,
) -> Result<ArgSpec, ArgSpecError> {
    let (items, span) = parse_vector_items(src);
    parse_argspec_triples(
        &items,
        ":wat::test::fn",
        &span,
        ParseOptions { allow_rest_binder },
    )
}
```

All contract bodies that called `parse_triples(src, false, false)` become `parse_triples(src, false)`; contract 07 calls `parse_triples(src, false)` (allow_rest_binder=false to trigger rejection).

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md` — FM catalog
2. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.1.fix.md` — locked decisions D1-D12 + trap-door T1-T8 + STOP triggers
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/FORM-COLLAPSE-NOTES.md` — line 184 doctrine on argspec scope
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.1.md` § Vigilia Convergence — the findings driving Layer 1
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.1.md` — parent stone's design (now historically wrong-scope; preserved)
6. `/home/watmin/work/holon/wat-rs/src/argspec/parse.rs` — current uncommitted state (Layer 1 amends applied)
7. `/home/watmin/work/holon/wat-rs/src/argspec/error.rs` — current uncommitted state (Layer 1 amends applied)
8. `/home/watmin/work/holon/wat-rs/src/argspec/mod.rs` — module doc to update
9. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone1_argspec_canonical.rs` — current uncommitted state (Layer 1 amends applied; 13 contracts; needs Layer 2 cut to 9)
10. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/EXPECTATIONS-STONE-241.1.fix.md` — what completion looks like (Phase A scorecard)

## Implementation sketch (order of operations)

1. Read all four substrate files + the probe; confirm Layer 1 amends are in place
2. Baseline check: `cargo test --release --lib -p wat` (expect 834 PASS) + `cargo test --release --test probe_arc241_stone1_argspec_canonical` (expect 13 PASS — pre-scope-correction)
3. **error.rs**: delete `MissingRetArrow` + `RetTypeNotKeyword` variants; rename `IncompleteSignature` → `IncompleteTriple`; update `classify()` arms (delete two; rename one)
4. **parse.rs**: delete `ret_type` field from `ArgSpec`; delete `include_ret_type` field from `ParseOptions`; rename `IncompleteSignature` → `IncompleteTriple` at construction site; strip the loop's `is_bare_symbol("->")` break; strip the entire post-loop `if options.include_ret_type {...}` block; strip the final trailing-items check (loop consumes full slice in 241.1); update doc comments
5. **mod.rs**: update module doc to clarify argspec parses ONLY canonical triples; cite FORM-COLLAPSE-NOTES:184
6. **probe**: delete 5 contracts (current 03, 04, 08, 09, 12); add replacement contract_03 (multiple fixed params, no ret); rename `contract_13` → `contract_09_incomplete_triple` with updated match arm; renumber remaining contracts; update `parse_triples` helper (drop `include_ret_type` param); update all contract call sites from `parse_triples(src, X, Y)` to `parse_triples(src, Y)`
7. Verify: `cargo test --release --lib -p wat` (≥834 PASS) + `cargo test --release --test probe_arc241_stone1_argspec_canonical` (9 PASS) + `cargo build --release --tests --workspace` (clean) + `cargo clippy --release` (≤ 905)
8. Write `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.1.fix.md`
9. **DO NOT COMMIT.** Orchestrator commits after vigilia re-cast (Phase B).

## Discipline

- **`src/lib.rs` UNCHANGED.** Stone 241.1's `pub mod argspec;` stays.
- **No new files.** Strip existing ones.
- **No new public API.** `classify()` + `parse_keyword_type` stay PRIVATE.
- **`parse_argspec_triples` signature UNCHANGED.** Body shrinks; signature stays.
- **`ParseOptions` struct STAYS** (one field; `allow_rest_binder`).
- **`ArgSpec` struct STAYS** (two fields; `fixed_params` + `rest_param`).
- **Type-system migration is mechanical**: the compiler tells you every site that touched the removed `ret_type` field / `include_ret_type` field / removed variants. Follow the diagnostics. NO callers outside argspec touch these (Stone 241.1 didn't migrate any A1-A4 callers per D5 of original DESIGN); the compile errors will be confined to argspec home + the probe.
- **No `cargo run`; no wrapper scripts; just `cargo test/build/clippy`.**

## STOP triggers — each is REJECTION (ship NOTHING; surface as finding)

1. **STOP-1** — Unexpected compile errors NOT traced to the removed types/fields/variants
2. **STOP-2** — Lib baseline regression (<834)
3. **STOP-3** — 40 min elapsed
4. **STOP-4** — `holon-rs` touched
5. **STOP-5** — Rust files outside `src/argspec/error.rs`, `src/argspec/parse.rs`, `src/argspec/mod.rs`, `tests/probe_arc241_stone1_argspec_canonical.rs` touched. `src/lib.rs` MUST stay unchanged.
6. **STOP-6** — Scope creep: migrating A1/A2/A3/A4; minting `parse_ret_clause`; adding ANY new type/field/variant
7. **STOP-7** — Probe doesn't reach 9/9 PASS
8. **STOP-8** — Any prior arc 237 probe regresses
9. **STOP-9** — Clippy warnings > 905
10. **STOP-10** — Loop logic post-strip exhibits unexpected behavior (e.g., `->` at name slot doesn't surface as `MissingArrow`); surface as finding

## SCORE doc spec — write `SCORE-STONE-241.1.fix.md`

Mirror `SCORE-STONE-241.1.md`'s structural shape:

- **Header**: status (Mode A/B); runtime; summary one-liner covering BOTH layers (vigilia amends + scope correction)
- **Phase A scorecard** ~15 rows covering both layers
- **Final API signatures** verbatim post-scope shapes
- **Line counts per file** with deltas vs Stone 241.1 baseline (significant net negative expected)
- **Clippy delta** should be 0
- **Lib baseline** 834+ PASS / 0 FAIL
- **Probe**: 9/9 PASS
- **Workspace test-build**: clean
- **Honest deltas** anything you noticed
- **NO Vigilia Convergence section** — orchestrator inscribes after Phase B re-cast

## Post-strike

Return with a one-paragraph status summary. Orchestrator will:
1. Verify the scorecard rows independently
2. Re-cast vigilia Phase B (the solvere L2 should vanish structurally)
3. Commit when L1+L2=0

The gate doctrine `feedback_namespaced_home_vigilia_gate` survived its first real test (the vigilia caught Stone 241.1's scope confusion). Stone 241.1.fix CLOSES the gate by correcting the structural issue at its source.
