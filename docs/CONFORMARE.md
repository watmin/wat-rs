# CONFORMARE — error-shape discipline

Substrate convention. Every error type in wat-rs MUST conform to the structural pattern this document specifies. The wrong shape is uncompilable.

Sibling to `ZERO-MUTEX.md`. Same shape of doctrine: a class of failure made structurally unavailable by construction.

## The class being eliminated

An error type whose variants are permitted by the type system to silently lack source location data. The Rust language does not enforce "every error variant has a span field"; without a structural discipline, error types accumulate variants that drop diagnostic data, the substrate's user-facing error messages degrade silently, and the practitioner cannot trust that any given error path will surface a usable source location.

Worked example (pre-discipline, now retired): `ParseStep::ArityMismatch { actual: usize }` — no span field; the type accepts both this variant and `ArrowMissing { span: Span }` without complaint; consumer code that wants the span across variants must exhaustively match, and the spanless variant produces `Span::unknown()` at the consumer boundary. The substrate's diagnostic surface degrades to "something failed somewhere."

## The principle

> An error type must guarantee diagnostic completeness STRUCTURALLY. The constructor must demand the span. The type cannot lie about having one.

## The pattern — Pattern A: outer struct + kind enum

Every diagnostic-bearing error type takes this shape:

```rust
/// Top-level error: span at the outer level; variant data in `kind`.
pub struct MyError {
    pub span: Span,
    pub kind: MyErrorKind,
}

pub enum MyErrorKind {
    /// Variants carry their unique data only — NO `span` field per variant.
    SomeFailure { detail: String },
    AnotherFailure { actual: usize, expected: usize },
    // ...
}
```

**Construction:**
```rust
MyError {
    span: caller_span.clone(),
    kind: MyErrorKind::SomeFailure { detail: "...".into() },
}
```

The constructor demands the span. There is no way to construct `MyError` without supplying one. Silent omission is uncompilable.

**Span access:**
```rust
let span: &Span = &err.span;  // ONE path, every variant
```

Consumers do not exhaustive-match to extract span. The field is at the top.

## Construction guarantees

1. The outer struct's `span` field is `Span` (bare; not `Option<Span>`). Author MUST decide. If no source location exists by domain (registry-time errors, etc.), the author writes `Span::unknown()` explicitly + documents with a rune (see § Rune).
2. The `kind` enum's variants carry ONLY data unique to that variant. No span field per variant.
3. New variant authorship: add to `MyErrorKind`. Span discipline is inherited for free — no per-variant span field to remember to add.

## Conversion impls (`From<E1> for E2`)

Every conversion preserves span:

```rust
impl From<TypeError> for RuntimeError {
    fn from(e: TypeError) -> Self {
        RuntimeError {
            span: e.span,  // PRESERVED — never replaced with Span::unknown
            kind: RuntimeErrorKind::TypeFailure(e.kind),
        }
    }
}
```

The destination's `span` field is populated from the source's `span` field. Conversion impls that produce `span: Span::unknown()` while the source carries a real span are doctrinally invalid.

## Multi-span variants

Some variants carry secondary spans (e.g., `SandboxScopeLeak` has both the call site and the outer define site; `DefRedefForbidden` has both prior_loc and current_loc; `PostconditionFailed` has both body_span and ensure_span).

The PRIMARY span (the most actionable location for the user) lives in the outer struct's `span` field. Secondary spans live in the `kind` variant's fields. Field names for secondary spans MAY be domain-descriptive (`call_span`, `prior_loc`, etc.) — secondary spans are not under the primary-span discipline.

```rust
pub struct RuntimeError {
    pub span: Span,  // PRIMARY — the action site
    pub kind: RuntimeErrorKind,
}

pub enum RuntimeErrorKind {
    SandboxScopeLeak { offending_name: String, outer_define_span: Span },
    DefRedefForbidden { name: String, prior_loc: Span },
    // primary span lives in outer struct; secondary span is variant-local
}
```

Consumers wanting the secondary span match on the kind for the case that needs it. Consumers wanting the primary span use `err.span` — universal.

## The rune (exemptions)

Per `feedback_runes_illegal_when_solvable`: runes are EXCEPTION mechanisms. Legal ONLY when:
- The fix is genuinely unsolvable in current scope (substrate-level constraint that can't be lifted), OR
- Fixing would impair performance (hot-path construction; must be benchmarked)

For conformare specifically:

**`rune:conformare(spanless-by-domain)`** — the variant has no source location by domain semantics. Examples:
- Registry-cycle detection at registration time (no AST node in scope)
- Startup-time invariant violations with no input source
- Digest/hash failures (operate on bytes, not source positions)

Format:
```rust
/// rune:conformare(spanless-by-domain) — register_subtype operates on
/// FQDN strings; no AST node in scope at call depth; the span field is
/// `Span::unknown()` because no source location exists for this failure.
pub enum TypeErrorKind {
    // ...
    CyclicSubtype { child: String, parent: String },
}
```

Construction site MUST still pass `Span::unknown()` explicitly — the rune documents WHY the unknown is honest; it does NOT exempt the constructor from supplying a span argument.

**`rune:conformare(attested-arc)`** — fix tracked in named follow-up arc that is open and the work is in flight. Reason MUST cite arc number + DESIGN.md path. The rune retires when the arc lands.

A rune with empty or vague reason fails the spell. A rune citing an arc that doesn't exist fails.

## Tier framework

Error types are tiered by their relationship to user-facing diagnostics:

**Tier 1 — Primary diagnostic-bearing types.** MUST conform to Pattern A. These are the types that surface to users at compile time or run time:
- `TypeError`, `RuntimeError`, `CheckError`, `ParseError`, `ParseStep`, `ArgSpecError`, `MacroError`, `LowerError`, `EdnReadError`, `ClauseGrammarError`, `ConfigError`

**Tier 2 — Wrapper / aggregate types.** Wrap tier-1 types; span lives in the wrapped error. The wrapper's own variants that carry NEW data must conform to Pattern A. Variants that ONLY wrap a tier-1 type inherit conformance:
- `StartupError`, `HarnessError`, `Error` (crate root)

**Tier 3 — Infrastructure types.** Operate at layers where source-position spans are not meaningful (file paths, byte positions, digests). MAY be domain-spanless without rune:
- `LexError` (operates pre-span-labeling on byte positions)
- `LoadError`, `LoadFetchError` (file-path domain)
- `HashError` (digest bytes)
- `ResolveError` (paths not positions)
- `StdlibError` (baked-in source)
- `TryRecvError` (channel state enum, not user diagnostic)

If a tier-3 type SOMETIMES needs to surface to users with span context, it is promoted to tier 2 (wrapped by a tier-1 or tier-2 type that adds the span) — not retrofitted with span fields.

## Audit: the conformare spell

`datamancy.dev/conformare/SKILL.md` — the audit spell. Casts on:
- Error-type enum + struct definitions (does the shape match Pattern A?)
- Constructor accessibility (can the type be built without supplying span?)
- `From<E1> for E2` impls (does conversion preserve span?)
- Parser / check API signatures (do they thread caller-supplied span into errors?)

Joins vigilia's default defensive set for code files. Cast at every commit boundary on error-touching changes. After the substrate completes the conformare retrofit, the spell continues as backsliding-protection — any future error type that bypasses Pattern A is flagged at code-review time.

## Anti-patterns

These shapes are doctrinally invalid; the spell catches them:

**Anti-pattern: per-variant span field (current legacy form)**
```rust
// WRONG — Rust accepts variants with and without span fields silently
pub enum MyError {
    HasSpan { span: Span, ... },
    NoSpan { ... },  // accepted by compiler; lies about diagnostic completeness
}
```

**Anti-pattern: Option<Span> at outer struct**
```rust
// WEAKER — author can pass None silently
pub struct MyError {
    pub span: Option<Span>,  // permissive; silent omission still possible
    pub kind: MyErrorKind,
}
```

**Anti-pattern: `From` impl drops source span**
```rust
// WRONG — span data discarded at conversion boundary
impl From<TypeError> for RuntimeError {
    fn from(e: TypeError) -> Self {
        RuntimeError {
            span: Span::unknown(),  // dropped despite `e.span` being available
            kind: RuntimeErrorKind::TypeFailure(format!("{:?}", e)),
        }
    }
}
```

**Anti-pattern: parser API takes bare slice without head_span parameter**
```rust
// WRONG — caller has span context but the API discards it at the boundary
pub fn parse_something(args: &[WatAST]) -> Result<T, MyError> {
    // If this fails, where does the span come from? The caller has it;
    // the parser doesn't. Forces Span::unknown() at the error site.
}

// RIGHT — head_span threaded through the API
pub fn parse_something(args: &[WatAST], head_span: &Span) -> Result<T, MyError> {
    // Errors carry head_span; user gets a source location.
}
```

## Cross-references

- `scratch/FAILURE-ENGINEERING.md` — the meta-principle this doctrine embodies (eliminate the CLASS structurally)
- `docs/ZERO-MUTEX.md` — sibling doctrine; same shape (a class of failure made structurally unavailable by construction)
- `datamancy.dev/conformare/SKILL.md` — the audit spell
- `docs/arc/2026/05/243-conformare-error-shape/` — the arc that brings the substrate into conformance
- `docs/arc/2026/05/243-conformare-error-shape/CONFORMARE-FIRST-CAST.md` — the spell's first-cast verdict that produced Pattern A from first principles
- `src/argspec/error.rs` (line 6 doctrine comment + `classify()` method) — substrate precedent that informed Pattern A; argspec home enforced span discipline by convention; Pattern A elevates the convention to structure

## The principle behind the doctrine

The substrate ships diagnostic-complete error types. Diagnostic completeness is not a convention to remember — it is a structural guarantee enforced by Rust's type system at the moment of construction. The practitioner who adds a new variant cannot silently break the contract because the contract isn't theirs to break. The compiler is the discipline; the doctrine names the shape; the spell audits the boundary cases the compiler cannot reach (conversion impls, API signatures).

The datamancer conformat. Errors shape together to one standard; the wrong shape is uncompilable.
