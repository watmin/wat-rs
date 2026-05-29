# DESIGN — Stone 241.1.fix — vigilia-convergence amends on `src/argspec/*`

**Status:** READY (sub-DESIGN). Amend pass on Stone 241.1's home. Eliminates the 4 L1 findings + selected L2 findings from `SCORE-STONE-241.1.md` § Vigilia Convergence. Blocks Stone 241.2 per spawn-block winding.

## Why this stone

Stone 241.1 Phase A shipped behaviorally correct (probe 10/10; lib 834/0). Phase B (vigilia cast) declared the home **DIVERGED** — 4 L1 + ~12 L2 findings with cross-spell convergence on 3-4 sites. Per `feedback_namespaced_home_vigilia_gate`: commit-readiness requires L1+L2=0 on namespaced wat-rs homes. SCORE-green was the L0 floor; vigilia-convergence is the bar. The vigilia gate caught architectural issues sonnet's SCORE didn't surface — reason-string drift was actively biting; the probe leaked heap-pin strategy through an opaque trait return.

Per failure-engineering doctrine: **eliminate the class** at the *expressiveness* layer this time, not the behavior layer. Behavior is correct; the substrate's COMMUNICATION needs to converge on impeccable.

## What this stone delivers

Four substrate amends + six L2 cleanups, all mechanical given the locked decisions below. No new types, no new ParseOption fields, no new error variants. Pure expressiveness convergence on the existing surface.

### Substrate amends (L1)

| # | Site | Amend |
|---|---|---|
| A1 | `src/argspec/error.rs` | Extract `fn classify(self) -> (Span, String, String)` on `ArgSpecError`; three `From<>` impls collapse to mechanical 4-line wrappers |
| A2 | `src/argspec/parse.rs` | Extract `parse_keyword_type(ast, head, non_keyword_err)` helper; fixed-param + ret-type slots both route through it |
| A3 | `src/argspec/parse.rs:88-90` + struct field | Grimoire-prescribed `rune:purgare(future-fixture)` on the `unreachable!` arm AND on the `ArgSpec::rest_param` field |
| A4 | `tests/probe_arc241_stone1_argspec_canonical.rs:25-35` | Replace `impl std::ops::Deref<Target = wat::span::Span>` return with owned `(Vec<WatAST>, wat::span::Span)` — clone-cheap in test code, no trait leakage |

### Substrate cleanups (L2)

| # | Site | Cleanup |
|---|---|---|
| C1 | `parse.rs:158-163` | Remove tautological `is_bare_symbol(&args_vec[idx], "->")` guard — loop break invariant proves it can never fire; delete the guard + its error arm |
| C2 | `parse.rs:99` | Rewrite `idx + 2 >= args_vec.len()` as `args_vec.len().saturating_sub(idx) < 3` (idiomatic + semantic) |
| C3 | `parse.rs:98` | Delete the WHAT-comment ("Need 3 items for a complete triple; check before indexing.") — the `saturating_sub` form self-explains |
| C4 | `probe:25` | Rename helper `argspec_inputs` → `parse_vector_items` (reads as factory; is a parser) |
| C5 | `probe:38` | Rename helper `invoke` → `parse_triples` (intueri: surface-named for what it does) |
| C6 | `probe` | Add contracts 11–13 for the three currently-unprobed `ArgSpecError` variants: `MalformedTypeKeyword`, `RetTypeNotKeyword`, `IncompleteSignature` |

### NOT in scope (acceptable deferrals — vocare + complectens L2)

- TypeExpr-content checks on contracts 03/04 (verifying ret_type's `TypeExpr` payload, not just `.is_some()`) — extend if/when ret-type-content correctness becomes load-bearing
- Per-helper `#[test]` for `parse_vector_items` + `parse_triples` (the probe's helpers) — extend if surface grows
- Re-exporting `Span` from `wat::argspec` — probe explicitly imports `wat::span::Span`; the reach is documented

These are noted in SCORE-green Phase B re-cast; vigilia accepts them as L2-acceptable mumbles.

## Locked decisions

### D1 — `classify()` returns ONE domain-neutral reason per variant (drift eliminated at source)

The vigilia finding named "reason-string drift across 3 From impls (NameNotSymbol: 2× 'arg-vector...' + 1× 'field/arg...')." The drift was actually TWO axes:

- **Within domain** — RuntimeError says "at slot 1" for MissingArrow; CheckError drops the "at slot 1." Same domain (fn args). Pure drift; must be eliminated.
- **Across domains** — RuntimeError + CheckError say "arg-vector"; TypeError says "field/arg" because struct fields aren't args. Domain-tailored language; legitimately different.

The clean resolution: **make the reason DOMAIN-NEUTRAL**. Strip "arg-vector" and "field/arg" prefixes; just say "name slot," "triple," "type slot," "return-type slot." The `head` field already carries the form name (`:wat::core::defn` vs `:wat::core::defstruct`); the reader gets domain context from `head`, not from the reason wording.

Canonical reasons (locked):

| Variant | Reason |
|---|---|
| `NameNotSymbol` | `"name slot must be a plain symbol (not a keyword, literal, or nested form)"` |
| `MissingArrow` | `"triple must be \`name <- :T\`; \`<-\` arrow not found at slot 1"` |
| `TypeNotKeyword` | `"type slot must be a keyword (e.g. \`:wat::core::i64\`); got a non-keyword"` |
| `MalformedTypeKeyword { inner, .. }` | `format!("type keyword is malformed: {inner}")` |
| `MissingRetArrow` | `"expected \`->\` return-type arrow after argspec triples; not found"` |
| `RetTypeNotKeyword` | `"return-type slot after \`->\` must be a keyword; got a non-keyword"` |
| `TrailingItems { count, .. }` | `format!("{count} trailing item(s) beyond the expected signature shape")` |
| `IncompleteSignature` | `"triple is incomplete; expected \`name <- :T\` but ran out of items"` |
| `RestBinderNotSupported` | `"\`&\` rest-binder is not supported at this binding site"` |

Each From impl becomes a 4-line wrapper:

```rust
impl From<ArgSpecError> for crate::runtime::RuntimeError {
    fn from(err: ArgSpecError) -> Self {
        let (span, head, reason) = err.classify();
        Self::MalformedForm { head, reason, span }
    }
}
```

`CheckError::MalformedForm` mirrors. `TypeError::MalformedDecl` mirrors with the field-renaming `{ head, reason, span }`.

### D2 — `parse_keyword_type` helper unifies fixed-param + ret-type slots

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
        WatAST::Keyword(kw, kw_span) => {
            parse_type_expr_with_span(kw, kw_span).map_err(|inner| {
                ArgSpecError::MalformedTypeKeyword {
                    span: kw_span.clone(),
                    head: head.to_string(),
                    inner: Box::new(inner),
                }
            })
        }
        other => Err(non_keyword_err(other.span().clone(), head.to_string())),
    }
}
```

Call sites:

```rust
let ty = parse_keyword_type(&args_vec[idx + 2], head, |span, head| {
    ArgSpecError::TypeNotKeyword { span, head }
})?;

// ... ret-type slot ...

let ret = parse_keyword_type(&args_vec[idx], head, |span, head| {
    ArgSpecError::RetTypeNotKeyword { span, head }
})?;
```

The closure picks the non-Keyword error variant per site; the malformed-keyword path is uniform.

### D3 — Runes ONLY on genuinely future-fixture sites

Per purgare's grimoire (`~/work/holon/datamancy/purgare/SKILL.md`): format is `// rune:purgare(<category>) — <reason>`. Category `future-fixture` denotes substrate that is currently quiescent but exists for a planned later stone.

**Rune-accept (2 sites):**

1. `parse.rs:88-90` — the `unreachable!("allow_rest_binder is always false in Stone 241.1")` arm. Rune:
   ```rust
   // rune:purgare(future-fixture) — Stone 241.4 ships allow_rest_binder=true logic;
   // 241.1 path unreachable by design; field exists so API surface is stable from 241.1.
   unreachable!("allow_rest_binder is always false in Stone 241.1");
   ```

2. `parse.rs:14-17` — the `ArgSpec::rest_param` field. Rune:
   ```rust
   /// Rest parameter `(name, type)`, populated by Stone 241.4.
   /// Always `None` in Stone 241.1.
   // rune:purgare(future-fixture) — Stone 241.4 populates rest_param via allow_rest_binder
   //                                   path; field exists in 241.1 for API stability.
   pub rest_param: Option<(String, TypeExpr)>,
   ```

**NOT rune candidates (alive in 241.1):**

- The three `From<>` impls — ALL are reached at compile-time wiring of `?` operators in 241.2/241.3 callers; 241.1 ships the impls as forward-compatible substrate per AUDIT.md "Recommendation." Vigilia's struere flagged them as L2 wrong-level; the `classify()` extraction (A1) resolves the wrong-level concern. They are NOT dead.
- `ArgSpecError::RestBinderNotSupported` — reachable in 241.1 via probe contract 10 (the `&` rejection). Alive.

### D4 — Probe shape: owned span, not opaque trait

The probe's `argspec_inputs` returns `(Vec<WatAST>, impl std::ops::Deref<Target = wat::span::Span>)`. The `Box::new(span)` heap-pin is a workaround for the desire to avoid naming `wat::span::Span` as a type annotation. Vigilia's struere/sequi/complectens/vocare CONVERGED on this (4 spells) — strong AMEND signal.

The replacement:

```rust
fn parse_vector_items(src: &str) -> (Vec<WatAST>, wat::span::Span) {
    let ast = wat::parse_one!(src).expect("parse_one! should succeed for argspec source");
    match ast {
        WatAST::Vector(items, span) => (items, span),
        other => panic!("expected Vector form, got {:?}", other),
    }
}

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

Owned `Span` is clone-cheap; test code can name `wat::span::Span` directly (it's just `wat::span::Span`); no trait leakage.

### D5 — Three new contracts (11/12/13) for full ArgSpecError variant coverage

| Contract | Variant | Source form |
|---|---|---|
| 11 | `MalformedTypeKeyword` | A type-keyword shape that `parse_type_expr_with_span` rejects — sonnet finds via grep of `src/types.rs` rejection paths (candidates: `[x <- :wat::core::]` trailing-colon; `[x <- :NonExistentType]` unresolvable; whichever shape produces `TypeError` at parse time) |
| 12 | `RetTypeNotKeyword` | `[x <- :wat::core::i64 -> "string-not-keyword"]` with `include_ret_type=true` |
| 13 | `IncompleteSignature` | `[x <-]` (idx + 2 ≥ len before triple completes) |

Each follows the pattern of contracts 5–10: invoke + expect_err + matches! on the specific variant.

**Sonnet discretion on contract 11**: if no shape readily triggers `MalformedTypeKeyword` (the `parse_type_expr_with_span` rejection paths are narrow), surface as a finding — do NOT mint a fixture or skip the contract. The substrate-as-teacher cascade reveals what shape is needed.

### D6 — No new files; no new types; no API surface changes

Stone 241.1.fix amends EXISTING files. Specifically:

- `src/argspec/error.rs` — same enum, same variants, same `From<>` impls (now thin wrappers around `classify()`)
- `src/argspec/parse.rs` — same `parse_argspec_triples` signature, same `ArgSpec` / `ParseOptions` shapes (runes added to field + arm)
- `src/argspec/mod.rs` — UNCHANGED
- `tests/probe_arc241_stone1_argspec_canonical.rs` — same probe, helpers renamed, return type concrete, +3 contracts

Public API of `wat::argspec` UNCHANGED. Downstream consumers (Stone 241.2's A1/A2/A3 migration callers, future Stones) see the same surface. The amends are purely internal.

### D7 — Lib baseline preserved; probe expands 10→13

After Stone 241.1.fix:

- `cargo test --release --lib -p wat` = 834 PASS / 0 FAIL (or higher; never lower)
- `cargo test --release --test probe_arc241_stone1_argspec_canonical` = 13 PASS / 0 FAIL (was 10/0)
- `cargo build --release --tests --workspace` clean
- `cargo clippy --release` warning count unchanged

### D8 — Vigilia re-cast must converge L1+L2=0

Phase B re-cast on the amended files. Acceptable outcomes:

- All 8 spells return CONVERGED individually, OR
- L2 mumbles ACCEPTED via `rune:<spell>(<category>) — <reason>` only where the rune's REASON is load-bearing (per intueri's rune discipline)

The acceptable-deferral list (TypeExpr content; per-helper tests; Span re-export) stays L2-acceptable per the original Phase B inscription. New L2 findings on the amend pass must be addressed or rune'd, not deferred.

---

## Trap-door audit

### T1 — `classify()` consumes self; From impls work because `from(err)` already takes owned

The three `From<ArgSpecError>` impls take `err: ArgSpecError` by value. `classify(self)` consumes the error to extract owned `(Span, String, String)`. No clone needed at the call site; the move IS the conversion. Discipline: sonnet must NOT add `.clone()` calls — the value-move shape is intentional.

### T2 — `parse_keyword_type` returns `Result<TypeExpr, ArgSpecError>` (uniform error class)

The helper returns the canonical `ArgSpecError`, not a per-site error class. Conversion to runtime/check/type happens at the binding-site boundary (the `?` in 241.2/241.3 callers triggers `From<>`). The helper stays parser-internal.

### T3 — Closure shape for `non_keyword_err`

`FnOnce(Span, String) -> ArgSpecError` — the err-ctor takes ownership of the span + head and returns the constructed error. Simple closure; no lifetimes; no nesting. Both call sites use the same shape with different variants.

### T4 — Removing the tautology preserves error semantics

The current `parse.rs:158-163` block has TWO guards:
```rust
if idx >= args_vec.len() { return Err(MissingRetArrow ...); }
if !is_bare_symbol(&args_vec[idx], "->") { return Err(MissingRetArrow ...); }
```

The loop exits when EITHER `idx >= args_vec.len()` OR `is_bare_symbol(&args_vec[idx], "->")`. After the loop:
- If `idx >= args_vec.len()`: first guard fires.
- If `is_bare_symbol(args_vec[idx], "->") == true`: second guard's `!` makes it false; guard does NOT fire.

So the second guard is unreachable. Removing it preserves semantics. The first guard alone handles "consumed all items without `->`."

### T5 — `saturating_sub` semantic equivalence

`idx + 2 >= args_vec.len()` ⇔ `args_vec.len() - idx <= 2` when `args_vec.len() >= idx` (always true since idx is a slice index ≤ len). `saturating_sub` form: `args_vec.len().saturating_sub(idx) < 3` reads as "fewer than 3 items remaining." Equivalent for all valid idx values; saturates safely at idx > len (impossible here, but defensive).

### T6 — Rune on `rest_param` field doesn't break Stone 241.4's API extension

The rune is a comment + format: `// rune:purgare(future-fixture) — ...`. Field SHAPE (`pub rest_param: Option<(String, TypeExpr)>`) stays unchanged. Stone 241.4 will populate the field (replacing `None` with `Some((name, ty))` in the rest-binder path); the field's TYPE and visibility stay. Rune doesn't constrain that.

### T7 — Span re-export decision deferred

Vigilia's L2 finding suggested re-exporting `Span` from `wat::argspec`. Decision: DEFER. The probe imports `wat::span::Span` explicitly; the reach is honest (not laundered through `argspec`). Re-exporting WOULD add API surface for nobody; defer until/unless a 241.2/3 caller needs the convenience.

### T8 — The `head: &str` → `String` clone in `parse_keyword_type`

The helper signature takes `head: &str` and clones via `head.to_string()` when constructing errors. This mirrors the existing convention (`head.to_string()` appears at every error-construction site in `parse.rs`). No regression; just relocated into the helper.

---

## STOP triggers (REJECTION — not permission to defer)

1. **STOP-1** — Unexpected compile errors not traced to the amend-named sites
2. **STOP-2** — Lib baseline regression (current: 834 PASS / 0 FAIL; must hold ≥834)
3. **STOP-3** — 40 min elapsed (smaller scope than 241.1; this is the upper bound)
4. **STOP-4** — `holon-rs` touched (substrate is frozen)
5. **STOP-5** — Rust files outside `src/argspec/error.rs`, `src/argspec/parse.rs`, `tests/probe_arc241_stone1_argspec_canonical.rs` touched. `src/argspec/mod.rs` and `src/lib.rs` MUST stay unchanged.
6. **STOP-6** — Scope creep:
   - Migrating ANY of A1/A2/A3/A4 — that is 241.2/3
   - Implementing `&` rest-binder logic — that is 241.4
   - Adding NEW `ParseOptions` fields, `ArgSpecError` variants, or `ArgSpec` fields
   - Re-exporting `Span` from `wat::argspec` (T7 deferral)
   - Adding new files anywhere
7. **STOP-7** — Probe doesn't reach 13/13 PASS
8. **STOP-8** — Any prior arc 237 probe regresses (237.5/.5fix/.6/.8a tests stay green)
9. **STOP-9** — Clippy warnings increase above baseline (~54 per CLIFFNOTES)
10. **STOP-10** — Contract 11 can't find a shape that triggers `MalformedTypeKeyword` — surface as finding (do NOT skip the contract or rune-defer; orchestrator will guide on substrate)

Each STOP is REJECTION criteria: ship NOTHING when hit; surface as finding in SCORE.

---

## FM 2-bis evidence

The existing probe at `tests/probe_arc241_stone1_argspec_canonical.rs` IS the FM 2-bis substrate. Stone 241.1.fix:
- Renames helpers (Phase A: 10/10 still PASS after rename)
- Replaces opaque return with owned shape (Phase A: 10/10 still PASS)
- Adds contracts 11/12/13 (Phase A: 13/13 PASS post-stone)

Pre-stone (HEAD `6621f2a2`): 10/10 PASS at current shape. No new diagnostic surface is needed for the amend pass — the EXISTING probe + 3 new contracts cover the surface.

---

## SCORE doc spec

`docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.1.fix.md` (NEW). Mirror `SCORE-STONE-241.1.md`'s structural shape:

- **Phase A scorecard (14 rows)** — probe 13/13 + lib baseline + workspace test-build + clippy delta + file discipline + arc 237 regression
- **Final API signatures** — `classify()` signature, `parse_keyword_type` signature, `ArgSpec` shape (unchanged), `ParseOptions` shape (unchanged)
- **Line counts per file** — net delta (expect ~120 lines saved in error.rs; ~15 net in parse.rs; ~25 added in probe)
- **Clippy delta** — should be 0
- **Lib baseline confirmation** — 834+ PASS / 0 FAIL
- **Honest deltas** — anything sonnet noticed mid-strike
- **Vigilia Convergence section (Phase B)** — orchestrator-inscribed after re-cast; lists each spell's verdict + any runes accepted

---

## Calibration

**Target band:** 20–30 min Mode A.
**Upper bound:** 40 min (STOP-3).

**Surface estimate (net delta ~-80 to -100 lines; significant code SAVED):**

| File | Pre | Post | Delta |
|---|---|---|---|
| `src/argspec/error.rs` | 253 | ~130 | **-123** (classify() ~50 + 3 From impls × 4 lines + enum ~70) |
| `src/argspec/parse.rs` | 219 | ~210 | **-9** (helper +20; tautology -8; loop body -15; rune +4) |
| `tests/probe_arc241_stone1_argspec_canonical.rs` | 185 | ~235 | **+50** (renames 0; owned span -3; +3 contracts ~55) |

**Confidence: HIGH.** Mechanical amend; locked decisions; no new types; no new design forks. The vigilia findings are concrete and addressable; the resolution shape is named in the decisions above.

**Per `feedback_stone_briefs_cite_prior_score`:** mirror `SCORE-STONE-241.1.md`'s Phase A → Phase B → Phase C structural shape; the amend pass is smaller scope but same discipline.

---

## What this unblocks

Stone 241.2 — migrate the three fn-parser variants (A1 + A2 + A3) to route through `parse_argspec_triples`. The canonical parser is now BOTH behaviorally correct AND expressively impeccable; the substrate-as-teacher cascade for migration begins on a foundation that won't generate maintenance debt.

Beyond Phase 1: the `classify()` discipline pattern (one canonical reason per variant; head carries form context) generalizes to future error consolidations across the substrate.

---

## Cross-references

- `SCORE-STONE-241.1.md` § Vigilia Convergence — the 4 L1 + ~12 L2 findings driving this amend pass
- `DESIGN-STONE-241.1.md` — the parent stone's locked decisions; this amend stone preserves all of them
- `DESIGN.md` § Scope expansion 2026-05-28 — arc-level framing; Stone 241.1.fix is part of Phase 1 foundation work
- `feedback_namespaced_home_vigilia_gate` — the gate doctrine this amend honors
- `feedback_sonnet_writes_substrate` — orchestrator briefs/scores; sonnet writes the Rust
- `~/work/holon/datamancy/purgare/SKILL.md` — rune format reference (`rune:purgare(future-fixture) — <reason>`)
- `~/work/holon/datamancy/vigilia/SKILL.md` — aggregator spell for Phase B re-cast
- `COMPACTION-AMNESIA-RECOVERY.md` § FM 2-bis — the empirical-probe discipline (existing probe covers Stone 241.1.fix)
