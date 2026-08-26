# CONFORMARE — error-shape discipline

Substrate doctrine. Every error type in wat-rs MUST conform to the structural pattern this document specifies. The wrong shape is uncompilable.

Sibling to `ZERO-MUTEX.md`. Same shape of doctrine: a class of failure made structurally unavailable by construction.

## The class being eliminated

An error type whose variants are permitted by the type system to silently lack location data. The Rust language does not enforce "every error variant carries a location"; without a structural discipline, error types accumulate variants that drop diagnostic data, the substrate's user-facing error messages degrade silently, and the practitioner cannot trust that any given error path will surface a usable location.

Worked example (retired by Stone 243.4.1): `ParseStep::ArityMismatch { actual: usize }` — no span field; the type accepted both this variant and `ArrowMissing { span: Span }` without complaint; consumer code that wanted the span across variants had to exhaustively match, and the spanless variant produced `Span::unknown()` at the consumer boundary. The substrate's diagnostic surface degraded to "something failed somewhere." Retired by making `parse_fn_signature_prefix` take `&[WatAST; 3]` — arity is now type-impossible; the wrong-length case cannot reach the prefix at all.

## The principle

> An error type must guarantee diagnostic completeness STRUCTURALLY. The constructor must demand the location. The type cannot lie about having one.

## The pattern — Pattern A: outer struct + kind enum

Every diagnostic-bearing error type takes this shape:

```rust
/// Top-level error: location at the outer level; variant data in `kind`.
pub struct MyError {
    pub span: Span,
    pub kind: MyErrorKind,
}

pub enum MyErrorKind {
    /// Variants carry their unique data only — NO location field per variant.
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

The constructor demands the location. There is no way to construct `MyError` without supplying one. Silent omission is uncompilable.

**Location access:**
```rust
let span: &Span = &err.span;  // ONE path, every variant
```

Consumers do not exhaustive-match to extract the location. The field is at the top.

## Zero exceptions — the location is mandatory, typed to its domain

**User direction (2026-05-30): anything wat can toss from Rust must be location-aware.** There is no "this error has no location" escape. What varies is the location's TYPE, not its presence:

- **Source-position domain (the common case):** the outer field is `Span` (file:line:col). Most error types.
- **Non-source domains:** the outer field is the domain-appropriate location type — a file `Path` (load/resolve domain), a byte `Position` (pre-span lexing), etc. The SHAPE is identical (outer struct + kind enum + mandatory location); only the location's type changes.

The retired idea was a *tier* of "infrastructure" errors permitted to be location-less. That is gone: an error that operates on file paths carries the path; an error that operates on byte offsets carries the offset. A registration-time error with "no AST node in scope" **threads the caller's span** (Stone 243.5 proved this for `register_subtype` — the caller always has a location; the registration boundary discarded it) rather than excusing itself with `Span::unknown()`.

`Span::unknown()` remains a real value — the explicit sentinel for the genuinely-irrecoverable site — but it is NOT a doctrinal escape. A type that *routinely* constructs with `Span::unknown()` is a span-threading bug to fix, not a domain exemption to bless.

## Construction guarantees

1. The outer struct's location field is the domain's location type (bare; not `Option<_>`). `Span` for source positions; `PathBuf`/`Position`/… for non-source domains. The author MUST supply it; the struct-literal rule makes omission uncompilable.
2. The `kind` enum's variants carry ONLY data unique to that variant. No location field per variant.
3. New variant authorship: add to `MyErrorKind`. The location discipline is inherited for free — no per-variant field to remember.

## Namespaced-home requirement

Every error type lives in a vigilia-protected namespaced home: `src/<noun>/error.rs`. The home holds the `MyError` struct + `MyErrorKind` enum + their `Display`/`diagnostic()` impls + the `From` conversions into it. The error file is the home's first honest neighbor; the home earns a `vigilatum` stamp (see `VIGILATUM.md`) by a live vigilia cast at L1+L2=0 plus clippy-clean. Flat `src/*.rs` error types are pre-grimoire debt; the conformance retrofit and the home-carve land together (the rolling audit, below).

Carved homes to date: `src/types/error.rs` (TypeError, Stone 243.5), `src/argspec/error.rs` (ArgSpecError), `src/check/error.rs` (CheckError, Stone 243.6a). Remaining: RuntimeError (Stone 243.7a) and the rolling-audit set below.

## The module-doc contract (honest elision)

Every error home's module-doc states the contract the code actually enforces — no oversell. The standard wording:

> The location field is mandatory at construction — Rust's struct-literal rule makes a location-less error uncompilable. `Span::unknown()` is the explicit sentinel for the rare site with no recoverable location; `Display` and `diagnostic()` **elide** unknown locations (no `<runtime>:0:0` noise).

"Elide" is a load-bearing claim: it must be TRUE on every rendering surface. `Display` (primary AND any mid-prose secondary spans) and `diagnostic()` (every `location` field) must each suppress the unknown sentinel — route every location emission through one elide-aware mechanism so an unguarded emission is structurally impossible (Stone 243.6a: the `loc_field` helper + the `shown = span.filter(|s| !s.is_unknown())` gate; the claim is backed by a probe, not convention). Do NOT write "elides unknown spans" over code that emits the sentinel on some path — that is the exact claim-vs-code oversell `circumspicere` exists to catch.

## Conversion impls (`From<E1> for E2`)

Every conversion preserves the location:

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

The destination's location is populated from the source's. Conversions that produce `span: Span::unknown()` while the source carries a real location are doctrinally invalid.

## Multi-span variants

Some variants carry secondary locations (e.g., `SandboxScopeLeak` has the call site and the outer define site; `DefRedefForbidden` has the prior and current locations; `ProcessJoinBeforeOutputDrain` has the join site and the output-accessor site).

The PRIMARY location (the most actionable site for the user) lives in the outer struct's location field. Secondary locations live in the `kind` variant's fields, with domain-descriptive names (`outer_define_span`, `original_def_span`, `output_accessor_span`, …). Secondary locations follow the SAME elision contract — render them through the elide-aware mechanism (Stone 243.6a gated all four secondary-span Display interpolations); a secondary span is not a license to emit `<runtime>:0:0`.

```rust
pub struct RuntimeError {
    pub span: Span,  // PRIMARY — the action site
    pub kind: RuntimeErrorKind,
}

pub enum RuntimeErrorKind {
    SandboxScopeLeak { offending_name: String, outer_define_span: Span },
    DefRedefForbidden { name: String, original_def_span: Span },
}
```

Consumers wanting the secondary location match the kind; consumers wanting the primary use `err.span` — universal.

## The rune (exemptions)

Per `feedback_runes_illegal_when_solvable`: runes are EXCEPTION mechanisms. Legal ONLY when the fix is genuinely unsolvable in current scope, OR fixing would impair performance (hot-path construction; must be benchmarked).

For conformare, exactly one rune kind remains:

**`rune:conformare(attested-arc)`** — the fix is tracked in a named follow-up arc that is open and in flight. The reason MUST cite the arc number + DESIGN.md path. The rune retires when the arc lands.

```rust
// rune:conformare(attested-arc) — RuntimeError Pattern-A retrofit tracked in
// Stone 243.7a (DESIGN at docs/arc/2026/05/243-conformare-error-shape/DESIGN-STONE-243.7a.md)
```

A rune with empty/vague reason fails the spell. A rune citing a nonexistent arc fails.

**Retired at Stone 243.4: `rune:conformare(spanless-by-domain)`.** It excused a missing location by domain ("no AST node in scope," "operates on bytes," etc.). Zero-exceptions retires it: a non-source domain carries its appropriate location TYPE (path/position), and a registration boundary threads the caller's span (243.5). There is no honest "this error has no location."

## Rolling audit — every error type, no tiers

The retired "Tier 1/2/3" taxonomy permitted infrastructure types to be location-less. Under zero-exceptions there are no tiers: **every** error type is Pattern A with a mandatory, domain-typed location, lives in a namespaced home, and is retrofitted by the rolling audit. Per-type retrofit lands as its own stone (substrate-as-teacher cascade per type; the FM 2-bis probe per retrofit).

Retrofit status:
- **Done:** `TypeError` (243.3 / home 243.5), `ArgSpecError`, `CheckError` (243.6a).
- **In flight:** `RuntimeError` → Stone 243.7a (the 605-site by-value-return retrofit; carries the `attested-arc` runes until it lands).
- **Pending (rolling audit):** `ParseError`/`ParseStep`, `LexError` (byte-position location), `LoadError`/`LoadFetchError` (path location), `ResolveError` (path), `HashError`, `StdlibError`, `MacroError`, `LowerError`, `EdnReadError`, `ClauseGrammarError`, `ConfigError`, and the wrapper/aggregate types (`StartupError`, `GuestError`, crate-root `Error`) — each gets a Pattern-A + home stone as the audit reaches it. Wrappers whose variants only wrap a conformant inner error inherit the inner location; wrapper variants carrying NEW data conform directly.

## Audit: the conformare spell

`datamancy.dev/conformare/SKILL.md` — the audit spell. Casts on error-type definitions (shape match?), constructor accessibility (buildable without a location?), `From` impls (location preserved?), and parser/check API signatures (caller location threaded into errors?). Joins vigilia's default defensive set for error-touching code files; after the retrofit completes it continues as backsliding-protection — any future error type bypassing Pattern A is flagged at cast time.

## Anti-patterns

These shapes are doctrinally invalid; the spell catches them:

**Per-variant location field (legacy form):**
```rust
// WRONG — Rust accepts variants with and without location fields silently
pub enum MyError {
    HasSpan { span: Span, ... },
    NoSpan { ... },  // lies about diagnostic completeness
}
```

**`Option<Span>` at the outer struct:**
```rust
// WEAKER — author can pass None silently
pub struct MyError { pub span: Option<Span>, pub kind: MyErrorKind }
```

**Domain-spanless exemption (retired):**
```rust
// WRONG (post-243.4) — there is no "this domain has no location" escape;
// carry the domain's location type, or thread the caller's span.
```

**`From` impl drops the source location:**
```rust
// WRONG — location discarded at the conversion boundary
RuntimeError { span: Span::unknown(), kind: ... }  // e.span was available
```

**Parser API takes a bare slice without a head-span parameter:**
```rust
// WRONG — caller has the location; the API discards it, forcing Span::unknown()
pub fn parse_something(args: &[WatAST]) -> Result<T, MyError> { ... }
// RIGHT — head_span threaded through
pub fn parse_something(args: &[WatAST], head_span: &Span) -> Result<T, MyError> { ... }
```

**"Elides unknown" claimed over a path that emits the sentinel:**
```rust
// WRONG — the module-doc promises elision the code doesn't enforce on every
// rendering surface (circumspicere's claim-vs-code catch). Route all location
// emission through one elide-aware mechanism; back the claim with a probe.
```

## Cross-references

- `scratch/FAILURE-ENGINEERING.md` — the meta-principle (eliminate the CLASS structurally)
- `docs/ZERO-MUTEX.md` — sibling doctrine; same shape
- `docs/VIGILATUM.md` — the ward-provenance marker each error home earns
- `datamancy.dev/conformare/SKILL.md` — the audit spell
- `docs/arc/2026/05/243-conformare-error-shape/` — the arc that brings the substrate into conformance (DESIGN + per-stone SCOREs)
- `docs/arc/2026/05/243-conformare-error-shape/CONFORMARE-FIRST-CAST.md` — the spell's first-cast verdict that produced Pattern A from first principles

## The principle behind the doctrine

The substrate ships diagnostic-complete error types. Diagnostic completeness is not a convention to remember — it is a structural guarantee enforced by Rust's type system at the moment of construction. The practitioner who adds a new variant cannot silently break the contract because the contract isn't theirs to break. The compiler is the discipline; the doctrine names the shape; the spell audits the boundary cases the compiler cannot reach (conversion impls, API signatures, the elision claim). There are no tiers and no domain exemptions: every error carries its location, typed to its domain, in a warded home.

The datamancer conformat. Errors shape together to one standard; the wrong shape is uncompilable.
