# Stone 243.3 — TypeError Pattern A retrofit (first conformare cascade)

**Status:** STRIKE-READY pending FM 2-bis probe commit + BRIEF/EXPECTATIONS commit.

**Trigger:** conformare spell's first-cast verdict (CONFORMARE-FIRST-CAST.md) identified TypeError as the recommended starter for the per-error-type retrofit — smallest blast radius among diagnostic-bearing types + cleans the base of the type hierarchy (ArgSpecError converts INTO TypeError; closing TypeError first stabilizes the foundation).

**Discipline:** Pattern A per `docs/CONFORMARE.md`. Type shape:
```rust
pub struct TypeError {
    pub span: Span,
    pub kind: TypeErrorKind,
}
pub enum TypeErrorKind { /* variants without per-variant span */ }
```

## Four-questions verdict (recorded; ran inline before stone planning)

| Axis | Verdict |
|---|---|
| Obvious | YES — std::io::Error precedent; new variant authorship trivial ("add to kind; struct handles span") |
| Simple | YES — removes 15 per-variant span field declarations; collapses 16-arm match at parse.rs to single `err.span` access |
| Honest | YES — Rust compiler enforces span at construction; CyclicSubtype documented via rune; future variants cannot silently break discipline |
| Good UX | YES — span access universal (`err.span`); cascade is mechanical one-time investment |

All four atomic YES. Alternatives stress-tested:
- Alt 1 (just add span to CyclicSubtype): fails Honest (class survives; convention-based)
- Alt 2 (defer; only fix ParseStep::ArityMismatch): fails Obvious + Honest (instance not class)
- Alt 3 (Pattern A retrofit, this stone): wins 4/4

## Scope

### Substrate refactor (src/types.rs)

1. **Rename** `pub enum TypeError { ... }` to `pub enum TypeErrorKind { ... }`
2. **Strip** `span: Span` field from every variant in `TypeErrorKind` (15 variants: DuplicateType, ReservedPrefix, MalformedDecl, MalformedName, MalformedField, MalformedVariant, MalformedTypeExpr, AnyBanned, CyclicAlias, AliasArityMismatch, InnerColonInCompoundArg, CyclicUnion, EmptyUnion, SingleMemberUnion, InvalidUnionMember)
3. **Keep** `CyclicSubtype { child: String, parent: String }` as-is in TypeErrorKind (no span field; legitimately spanless per domain — see rune)
4. **Mint** new `pub struct TypeError { pub span: Span, pub kind: TypeErrorKind }`
5. **Add** `// rune:conformare(spanless-by-domain) — register_subtype operates on FQDN string arguments; no AST node in scope at registration time; the struct's span field is Span::unknown() because the registration site has no source location to point at` immediately preceding the `CyclicSubtype` variant in TypeErrorKind
6. **Update** `impl Display for TypeError` (currently matches all 16 variants) to match on `&self.kind` instead; use `self.span` directly for any span-prefix rendering
7. **Update** internal helper functions in types.rs that match TypeError variants

### Cascade — emitter sites (estimated 114 across substrate)

Every `TypeError::Variant { span, ... }` rewrites to `TypeError { span, kind: TypeErrorKind::Variant { ... } }`. The 15 spanned variants pass `span` to the outer struct + drop it from the variant fields. CyclicSubtype's single emitter at `src/types.rs:435` passes `span: Span::unknown()` explicitly.

Sites identified by initial survey (full list discovered by sonnet via grep during sweep):
- `src/types.rs` — 30+ internal sites
- `src/check.rs` — TypeError consumers + conversions
- `src/runtime.rs` — TypeError consumers
- `src/argspec/error.rs` — From<TypeError> impl
- `src/function/parse.rs` — BadRetType arm + 16-arm match
- `src/freeze.rs:583` — From<TypeError> for StartupError
- `src/macros.rs`, `src/lower.rs`, `src/check.rs` — additional consumers per spell's audit

### Cascade — consumer sites

Every `match err { TypeError::Variant { span, .. } => ... }` rewrites to `match err.kind { TypeErrorKind::Variant { .. } => ... }` + uses `err.span` separately where span access is needed.

The 16-arm span-extraction match at `src/function/parse.rs:154-172` collapses to:
```rust
ParseStep::BadRetType(e) => {
    RuntimeError::MalformedForm {
        head: ":wat::core::fn".into(),
        reason: e.kind.to_string(),  // or appropriate kind render
        span: e.span,  // single field access
    }
},
```
(17 lines → ~6 lines.)

### From impls preserve span

- `impl From<ArgSpecError> for TypeError` (if exists; check) — destination's `span: Span` populated from source's `classify()` span
- `impl From<TypeError> for StartupError` at `src/freeze.rs:583` — preserves `e.span`
- `impl From<TypeError> for RuntimeError` (if exists) — preserves `e.span`

### Test cascade

- `tests/probe_arc237_stone1_typeunion_substrate.rs` — update any pattern matches over TypeError to new shape
- All workspace tests must continue to pass

### CONFORMARE.md update

Cite Stone 243.3 in the doctrine doc as the FIRST APPLIED EXAMPLE of Pattern A — concrete demonstration replacing the abstract Pattern A description for TypeError specifically.

## Trap-doors

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | Some TypeError variants have multi-line definitions (MalformedVariant, AliasArityMismatch, InnerColonInCompoundArg, InvalidUnionMember) with span across multiple lines — care needed during the strip | sonnet's per-variant audit during refactor | strip span from each, regardless of line shape |
| **T2** | `Display` impl matches all 16 variants for formatting; spans were in the destructure pattern — needs update to access `self.span` separately | Compile errors at the Display impl after enum rename | substrate-as-teacher catches; mechanical update |
| **T3** | Span-extraction match at parse.rs:154-172 is the load-bearing simplification — must collapse to `err.span` access | Manual verification + clippy may complain about dead match arms | Replace the whole match block with single `e.span` field access |
| **T4** | Tests that construct TypeError variants directly need shape updates | `cargo test` failures after refactor | mechanical per-site update |
| **T5** | Some TypeError emitters may use complex span derivation (e.g., `node.span().clone()`) — span flow must be preserved at the outer struct field | Per-site review during emitter cascade | every site that currently constructs `TypeError::Variant { span, ... }` already has span in scope; just move to outer position |
| **T6** | `From<ArgSpecError> for TypeError` (if exists) needs span preservation via `classify()` extraction | Conformare audit verification post-stone | grep for From<.*> for TypeError; ensure each preserves span |
| **T7** | TypeError exported as `pub` — any external crate consumer (currently none beyond wat-rs but check wat-* crates in workspace) needs shape update | `cargo build --workspace` failures | likely scope-only; survey on strike |

## STOP triggers

1. Compile errors not traced to TypeError refactor (something else broken)
2. Lib test count drops below baseline (currently ~890; verify pre-spawn)
3. Tests/function regression (must stay 8/8)
4. `cargo build --tests --workspace` fails after refactor (workspace-level cascade)
5. holon-rs touched (STOP-5)
6. New error types defined as bypassing Pattern A (out of scope; this stone retrofits TypeError ONLY)
7. Conformare spell re-cast on TypeError finds residual L1 (refactor incomplete)

## What this stone DOES NOT do

- Does not retrofit RuntimeError, CheckError, ParseStep, or other error types (each gets its own stone per spell's recommended ordering)
- Does not address the `span_prefix` helper duplication (out of scope; can land as a follow-up cleanup when other retrofits land)
- Does not add a `trait Conformare` (per spell verdict + four-questions; Pattern A is structural per-type)
- Does not change TypeError's diagnostic prose (Display impl preserves current message text for each variant)

## Strike outcome — what completion looks like

- `src/types.rs` exhibits Pattern A for TypeError (struct + kind enum + rune on CyclicSubtype)
- 114+ emitter sites updated to new construction shape
- All consumer match arms updated
- `parse.rs:154-172` 16-arm match collapsed to `err.span` field access (~6 lines)
- From impls preserve span via outer struct's field
- Test gate: lib ≥ 890; tests/function 8/0; workspace test-build clean
- Conformare spell re-cast on `src/types.rs` returns CONFORMARE for TypeError (Pattern A verified structurally)
- ParseStep::ArityMismatch remains untouched (next stone's scope)

## Calibration

Cascade size: comparable to arc 241's defenum (Stone 241.9: 33 files / 50min UNDER 60-120 band) and defstruct (Stone 241.8: 27 files / 41min UNDER band). TypeError has more emitter sites (114) but the per-site rewrite is more mechanical (no shape-redesign per variant; just relocate span to outer struct).

Predicted band: 60-120 min Mode A. Substrate-as-teacher discipline handles the cascade via Rust's compile errors.
