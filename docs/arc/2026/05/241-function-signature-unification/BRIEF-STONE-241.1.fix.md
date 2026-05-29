# BRIEF — Stone 241.1.fix — vigilia-convergence amends on `src/argspec/*`

You are sonnet (the Shadowdancer). You strike on a small-scope amend pass against the home Stone 241.1 just laid. The behavior is correct; the substrate's COMMUNICATION needs to converge on impeccable. The vigilia gate caught architectural issues sonnet's SCORE didn't surface; this stone closes them.

## What to do

Make EXACTLY these four substrate amends + six L2 cleanups. No others. No new files. No new types. No new ParseOption fields. No new error variants. No public API changes.

### Amend A1 — `src/argspec/error.rs` — extract `classify()` (eliminates reason-string drift at source)

The three `From<ArgSpecError>` impls currently produce different reason strings for the same variant across sites (RuntimeError keeps "at slot 1"; CheckError drops it; TypeError says "field/arg" instead of "arg-vector"). The drift is two axes — within-domain (RuntimeError vs CheckError) and across-domains (arg-form vs field-form). The clean resolution: **make reasons domain-NEUTRAL**. The `head` field already carries the form name (`:wat::core::defn` vs `:wat::core::defstruct`); the reader gets domain context from `head`, not from the reason wording.

Strip "arg-vector" and "field/arg" prefixes. Say "name slot," "triple," "type slot," "return-type slot."

Add this method to `ArgSpecError`:

```rust
impl ArgSpecError {
    fn classify(self) -> (Span, String, String) {
        match self {
            ArgSpecError::NameNotSymbol { span, head } => (
                span,
                head,
                "name slot must be a plain symbol (not a keyword, literal, or nested form)".into(),
            ),
            ArgSpecError::MissingArrow { span, head } => (
                span,
                head,
                "triple must be `name <- :T`; `<-` arrow not found at slot 1".into(),
            ),
            ArgSpecError::TypeNotKeyword { span, head } => (
                span,
                head,
                "type slot must be a keyword (e.g. `:wat::core::i64`); got a non-keyword".into(),
            ),
            ArgSpecError::MalformedTypeKeyword { span, head, inner } => (
                span,
                head,
                format!("type keyword is malformed: {inner}"),
            ),
            ArgSpecError::MissingRetArrow { span, head } => (
                span,
                head,
                "expected `->` return-type arrow after argspec triples; not found".into(),
            ),
            ArgSpecError::RetTypeNotKeyword { span, head } => (
                span,
                head,
                "return-type slot after `->` must be a keyword; got a non-keyword".into(),
            ),
            ArgSpecError::TrailingItems { span, head, count } => (
                span,
                head,
                format!("{count} trailing item(s) beyond the expected signature shape"),
            ),
            ArgSpecError::IncompleteSignature { span, head } => (
                span,
                head,
                "triple is incomplete; expected `name <- :T` but ran out of items".into(),
            ),
            ArgSpecError::RestBinderNotSupported { span, head } => (
                span,
                head,
                "`&` rest-binder is not supported at this binding site".into(),
            ),
        }
    }
}
```

Each From impl collapses to a 4-line wrapper:

```rust
impl From<ArgSpecError> for crate::runtime::RuntimeError {
    fn from(err: ArgSpecError) -> Self {
        let (span, head, reason) = err.classify();
        Self::MalformedForm { head, reason, span }
    }
}

impl From<ArgSpecError> for crate::check::CheckError {
    fn from(err: ArgSpecError) -> Self {
        let (span, head, reason) = err.classify();
        Self::MalformedForm { head, reason, span }
    }
}

impl From<ArgSpecError> for TypeError {
    fn from(err: ArgSpecError) -> Self {
        let (span, head, reason) = err.classify();
        Self::MalformedDecl { head, reason, span }
    }
}
```

`classify(self)` consumes the error; the `from(err)` impls already take `err` owned; no `.clone()` needed.

### Amend A2 — `src/argspec/parse.rs` — extract `parse_keyword_type` helper

Fixed-param type slot (current `parse.rs:126-142`) and ret-type slot (current `parse.rs:173-189`) have identical keyword-parse logic. Extract:

```rust
fn parse_keyword_type<F>(
    ast: &WatAST,
    head: &str,
    non_keyword_err: F,
) -> Result<TypeExpr, ArgSpecError>
where
    F: FnOnce(Span, String) -> ArgSpecError,
{
    match ast {
        WatAST::Keyword(kw, kw_span) => parse_type_expr_with_span(kw, kw_span).map_err(|inner| {
            ArgSpecError::MalformedTypeKeyword {
                span: kw_span.clone(),
                head: head.to_string(),
                inner: Box::new(inner),
            }
        }),
        other => Err(non_keyword_err(other.span().clone(), head.to_string())),
    }
}
```

Replace the fixed-param block with:
```rust
let ty = parse_keyword_type(&args_vec[idx + 2], head, |span, head| {
    ArgSpecError::TypeNotKeyword { span, head }
})?;
```

Replace the ret-type block with:
```rust
let ret = parse_keyword_type(&args_vec[idx], head, |span, head| {
    ArgSpecError::RetTypeNotKeyword { span, head }
})?;
```

### Amend A3 — Rune `unreachable!` arm + `rest_param` field (grimoire-prescribed)

Format per `~/work/holon/datamancy/purgare/SKILL.md`: `// rune:purgare(<category>) — <reason>`.

At the `unreachable!` arm (currently `parse.rs:87-90`):

```rust
// Stone 241.4 implements rest-binder parsing here.
// rune:purgare(future-fixture) — Stone 241.4 ships allow_rest_binder=true logic;
// 241.1 path unreachable by design; field exists so API surface is stable from 241.1.
unreachable!("allow_rest_binder is always false in Stone 241.1");
```

At the `ArgSpec::rest_param` field (currently `parse.rs:14-17`):

```rust
/// Rest parameter `(name, type)`, populated by Stone 241.4.
/// Always `None` in Stone 241.1.
// rune:purgare(future-fixture) — Stone 241.4 populates rest_param via allow_rest_binder
//                                 path; field exists in 241.1 for API stability.
pub rest_param: Option<(String, TypeExpr)>,
```

NO other runes. The three `From<>` impls are NOT dead (used at compile-time by `?` operators in 241.2/241.3 callers); `RestBinderNotSupported` is reachable in 241.1 via probe contract 10.

### Amend A4 — Probe: replace opaque trait return with owned shape

Currently:
```rust
fn argspec_inputs(src: &str) -> (Vec<WatAST>, impl std::ops::Deref<Target = wat::span::Span>) {
    let ast = wat::parse_one!(src).expect("parse_one! should succeed for argspec source");
    match ast {
        WatAST::Vector(items, span) => {
            // Heap-pin the span so the &Span reference passed to parse_argspec_triples
            // can outlive the match arm without naming wat::span::Span as a type annotation.
            (items, Box::new(span))
        }
        other => panic!("expected Vector form, got {:?}", other),
    }
}
```

Replace with (combined with C4 rename):
```rust
fn parse_vector_items(src: &str) -> (Vec<WatAST>, wat::span::Span) {
    let ast = wat::parse_one!(src).expect("parse_one! should succeed for argspec source");
    match ast {
        WatAST::Vector(items, span) => (items, span),
        other => panic!("expected Vector form, got {:?}", other),
    }
}
```

The opaque `impl Deref<Target=Span>` was a workaround to avoid naming `wat::span::Span` as a type annotation. The annotation IS the honest shape; the trait return leaks heap-pin strategy. Vigilia's struere+sequi+complectens+vocare CONVERGED here (4-spell signal — strong AMEND).

### Cleanup C1 — Remove tautological guard at `parse.rs:158-163`

Currently:
```rust
if idx >= args_vec.len() {
    return Err(ArgSpecError::MissingRetArrow { span: form_span.clone(), head: head.to_string() });
}
if !is_bare_symbol(&args_vec[idx], "->") {
    return Err(ArgSpecError::MissingRetArrow { span: args_vec[idx].span().clone(), head: head.to_string() });
}
```

The loop exits when EITHER `idx >= args_vec.len()` OR `is_bare_symbol(&args_vec[idx], "->")` is true. After the loop:
- `idx >= args_vec.len()` → first guard fires.
- `is_bare_symbol(args_vec[idx], "->") == true` → second guard's `!` makes it false; guard does NOT fire.

The second guard is unreachable. Delete it (the `if !is_bare_symbol(...)` block + its error arm). Keep the first guard.

### Cleanup C2 — Rewrite `parse.rs:99` as `saturating_sub`

Currently:
```rust
if idx + 2 >= args_vec.len() {
    return Err(ArgSpecError::IncompleteSignature { ... });
}
```

Replace with:
```rust
if args_vec.len().saturating_sub(idx) < 3 {
    return Err(ArgSpecError::IncompleteSignature { ... });
}
```

Reads as "fewer than 3 items remaining to form a triple."

### Cleanup C3 — Delete the WHAT-comment at `parse.rs:98`

Currently: `// Need 3 items for a complete triple; check before indexing.`

DELETE. The `saturating_sub` form self-explains.

### Cleanup C4 — Rename probe helper `argspec_inputs` → `parse_vector_items`

Already covered in Amend A4 (combined). The new name reads as a parser, not a factory.

### Cleanup C5 — Rename probe helper `invoke` → `parse_triples`

Currently:
```rust
fn invoke(
    src: &str,
    include_ret_type: bool,
    allow_rest_binder: bool,
) -> Result<ArgSpec, ArgSpecError> {
    let (items, span) = argspec_inputs(src);
    parse_argspec_triples(
        &items,
        ":wat::test::fn",
        &span,
        ParseOptions { include_ret_type, allow_rest_binder },
    )
}
```

Rename to:
```rust
fn parse_triples(
    src: &str,
    include_ret_type: bool,
    allow_rest_binder: bool,
) -> Result<ArgSpec, ArgSpecError> {
    let (items, span) = parse_vector_items(src);
    parse_argspec_triples(
        &items,
        ":wat::test::fn",
        &span,
        ParseOptions { include_ret_type, allow_rest_binder },
    )
}
```

Update all call sites in contracts 1–10 from `invoke(...)` to `parse_triples(...)`.

### Cleanup C6 — Add three new probe contracts (11/12/13)

After contract_10, append:

```rust
#[test]
fn contract_11_malformed_type_keyword() {
    // A type-keyword shape that parse_type_expr_with_span rejects → MalformedTypeKeyword.
    // Find a shape via grep of src/types.rs rejection paths.
    let result = parse_triples(/* <malformed keyword source> */, false, false);
    let err = result.expect_err("malformed type keyword must error");
    assert!(
        matches!(err, ArgSpecError::MalformedTypeKeyword { .. }),
        "expected MalformedTypeKeyword, got {:?}",
        err
    );
}

#[test]
fn contract_12_ret_type_not_keyword() {
    // [x <- :wat::core::i64 -> "string-not-keyword"] with include_ret_type=true.
    let result = parse_triples(
        r#"[x <- :wat::core::i64 -> "string-not-keyword"]"#,
        true,
        false,
    );
    let err = result.expect_err("non-keyword ret type must error");
    assert!(
        matches!(err, ArgSpecError::RetTypeNotKeyword { .. }),
        "expected RetTypeNotKeyword, got {:?}",
        err
    );
}

#[test]
fn contract_13_incomplete_signature() {
    // [x <-] — fewer than 3 items, runs out before triple completes.
    let result = parse_triples("[x <-]", false, false);
    let err = result.expect_err("incomplete triple must error");
    assert!(
        matches!(err, ArgSpecError::IncompleteSignature { .. }),
        "expected IncompleteSignature, got {:?}",
        err
    );
}
```

**Contract 11 — sonnet's discretion**: find a type-keyword shape that triggers `parse_type_expr_with_span` to return `Err`. Candidates to try: `[x <- :wat::core::]` (trailing colon), `[x <- :NonexistentNamespace::Type]`, `[x <- :1invalid]`. Grep `src/types.rs` for the rejection paths in `parse_type_expr_with_span`; use whichever shape clearly hits a parse-time rejection. If NO shape works (the rejection paths require runtime context not available at parse time): STOP-10. Surface to orchestrator. Do NOT skip the contract or rune-defer.

## Read in order

1. `docs/COMPACTION-AMNESIA-RECOVERY.md` — the FM catalog; FM 2-bis evidence discipline; FM 16 (no tool preamble)
2. `docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.1.fix.md` — the locked decisions for this stone
3. `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.1.md` § Vigilia Convergence — the findings driving each amend
4. `docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.1.md` — the parent stone's design (all decisions preserved)
5. `src/argspec/error.rs` — the file you'll amend (extract `classify()`)
6. `src/argspec/parse.rs` — the file you'll amend (extract `parse_keyword_type`; runes; tautology cleanup)
7. `tests/probe_arc241_stone1_argspec_canonical.rs` — the probe you'll amend (rename helpers; owned span; +3 contracts)
8. `~/work/holon/datamancy/purgare/SKILL.md` — rune format reference
9. `docs/arc/2026/05/241-function-signature-unification/EXPECTATIONS-STONE-241.1.fix.md` — what completion looks like (14-row scorecard + Phase B vigilia)

## Implementation sketch (order of operations)

1. Read the 4 files in `src/argspec/*` and `tests/probe_arc241_stone1_argspec_canonical.rs` to know current state
2. Verify baseline: `cargo test --release --lib -p wat` (expect 834 PASS) and `cargo test --release --test probe_arc241_stone1_argspec_canonical` (expect 10 PASS)
3. **error.rs**: extract `classify()`; collapse three From impls to 4-line wrappers
4. **parse.rs**: extract `parse_keyword_type` helper; replace fixed-param + ret-type blocks; rune the `unreachable!` arm; rune the `rest_param` field; remove tautology at the ret-arrow check; rewrite saturating_sub; delete WHAT-comment
5. **probe**: rename `argspec_inputs` → `parse_vector_items` with owned `Span` return; rename `invoke` → `parse_triples`; update contracts 1–10 call sites; append contracts 11/12/13
6. Verify: `cargo test --release --lib -p wat` (≥834 PASS) + `cargo test --release --test probe_arc241_stone1_argspec_canonical` (13 PASS) + `cargo build --release --tests --workspace` (clean) + `cargo clippy --release` (≤ baseline)
7. Write `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.1.fix.md` per the spec in EXPECTATIONS
8. **DO NOT COMMIT.** Orchestrator commits after vigilia re-cast (Phase B).

## Discipline

- **`src/argspec/mod.rs` UNCHANGED.** Module doc + re-exports stay verbatim.
- **`src/lib.rs` UNCHANGED.** The one-line `pub mod argspec;` from Stone 241.1 stays.
- **No new files.** This is an amend, not an extension.
- **No new public API.** `parse_keyword_type` and `classify()` are PRIVATE (no `pub`).
- **Type signatures preserved.** `parse_argspec_triples` signature unchanged. `ArgSpec` / `ParseOptions` / `ArgSpecError` shapes unchanged.
- **No `.clone()` on `err` in From impls.** `classify(self)` consumes; the move IS the conversion.
- **Errors stay parser-internal in shape.** The helper returns `Result<TypeExpr, ArgSpecError>`; conversion to `RuntimeError`/`CheckError`/`TypeError` happens at the binding-site boundary (Stone 241.2/3 callers' `?` triggers `From<>`).
- **No `cargo run`, no wrapper scripts, no shell pipelines beyond `cargo test/build/clippy`.**

## STOP triggers — each is REJECTION criteria (ship NOTHING; surface as finding)

1. **STOP-1** — Unexpected compile errors not traced to the amend-named sites
2. **STOP-2** — Lib baseline regression (current: 834 PASS / 0 FAIL; must hold ≥834)
3. **STOP-3** — 40 min elapsed (this is the upper bound)
4. **STOP-4** — `holon-rs` touched (substrate is frozen)
5. **STOP-5** — Rust files outside `src/argspec/error.rs`, `src/argspec/parse.rs`, `tests/probe_arc241_stone1_argspec_canonical.rs` touched. `src/argspec/mod.rs` and `src/lib.rs` MUST stay unchanged.
6. **STOP-6** — Scope creep: migrating A1/A2/A3/A4 (241.2/3); implementing `&` rest-binder logic (241.4); adding NEW ParseOptions fields / ArgSpecError variants / ArgSpec fields; re-exporting `Span`; adding new files
7. **STOP-7** — Probe doesn't reach 13/13 PASS
8. **STOP-8** — Any prior arc 237 probe regresses (237.5/.5fix/.6/.8a tests stay green)
9. **STOP-9** — Clippy warnings increase above baseline
10. **STOP-10** — Contract 11 can't find a shape that triggers `MalformedTypeKeyword` — surface as finding; do NOT skip the contract or rune-defer

## FM 2-bis evidence

The existing probe IS the substrate sonnet mirrors. Pre-stone (HEAD `6621f2a2`): 10/10 PASS. Post-stone: 13/13 PASS (contracts 11/12/13 added). No new probe file; the amend extends the existing surface.

## SCORE doc spec — write `SCORE-STONE-241.1.fix.md`

Mirror `SCORE-STONE-241.1.md`'s structural shape:

- **Header**: status (Mode A / B), runtime, summary one-liner
- **Phase A scorecard (14 rows)** — see EXPECTATIONS-STONE-241.1.fix.md
- **Final API signatures** — verbatim `classify()` + `parse_keyword_type` signatures
- **Line counts per file** — actual deltas
- **Clippy delta** — should be 0
- **Lib baseline** — 834+ PASS / 0 FAIL
- **Probe**: 13/13 PASS
- **Workspace test-build**: clean
- **Honest deltas** — anything you noticed mid-strike that the BRIEF didn't anticipate
- **Cascade depth** — 0 expected (pure substrate-internal amend)
- **NO Vigilia Convergence section** — orchestrator inscribes that after Phase B re-cast

## Post-strike

When SCORE-STONE-241.1.fix.md is written, return. Orchestrator will:
1. Independently verify the 14 scorecard rows
2. Cast vigilia (Phase B) on the amended files
3. If L1+L2=0 → atomic commit covering substrate + probe + SCORE doc
4. If vigilia DIVERGES → re-brief Stone 241.1.fix.fix (rare; this amend is mechanical)

The doctrine `feedback_namespaced_home_vigilia_gate` survived its first real test on Stone 241.1; Stone 241.1.fix is the second test. Strike clean.
