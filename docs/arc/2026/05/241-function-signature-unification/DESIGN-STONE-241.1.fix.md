# DESIGN — Stone 241.1.fix — vigilia-convergence + scope correction on `src/argspec/*`

**Status:** READY (sub-DESIGN). Amend pass on Stone 241.1's home. Eliminates the 4 L1 + L2 vigilia findings AND corrects the argspec scope confusion surfaced during Phase B re-cast. Blocks Stone 241.2 per spawn-block winding.

## Evolution note (2026-05-28 mid-day → late-mid-day)

This DESIGN evolved during execution. The original Stone 241.1.fix scope was vigilia-driven cleanups only (classify extraction, parse_keyword_type, runes, probe refactor). The original Stone 241.1 Phase A shipped per AUDIT.md (which carried `include_ret_type: bool` and `ret_type: Option<TypeExpr>` in argspec). Vigilia Phase B re-cast on the amended substrate caught a solvere L2 (`RetTypeNotKeyword` conflates slot-absent + slot-wrong). Surfacing the L2 to the user produced the verdict: **"Y — args have nothing to do with ret type."**

The deeper issue: **Stone 241.1 was scope-confused.** The user's canonical form (2026-05-28 early): *"the canonical form is `[arg1 <- :Arg1Type arg2 <- :Arg2Type argN <- :ArgNType]`"* — JUST args, no ret. `FORM-COLLAPSE-NOTES.md:184` confirms: *"Arc 241's `parse_argspec_triples` parses the canonical 3-slot triple uniformly across all binding sites."* The `AUDIT.md` snapshot (locked 2026-05-27 pre-form-collapse) had folded the ret-arrow + ret-keyword into argspec; the orchestrator shipped Stone 241.1 per AUDIT without re-surfacing the tension when form-collapse landed.

Per `feedback_trap_door_build_the_dependency`: don't declare the user's question incoherent; build the missing dependency — strip ret concerns out of argspec entirely; let fn-form parsers (241.2 callers) handle ret-clause parsing separately.

This Stone 241.1.fix now bundles BOTH:
1. **The vigilia amends** (Phase A green; substrate currently has them on disk uncommitted)
2. **The scope correction** (strip ret-clause from argspec)

The solvere L2 vanishes because the variant vanishes from this module.

## Why this stone

Stone 241.1 Phase A shipped behaviorally correct but scope-confused. The vigilia gate caught BOTH the expressiveness L1+L2 (drift, duplication, opaque trait return) AND the structural scope tension (argspec parsing ret-clause it doesn't own). Per `feedback_namespaced_home_vigilia_gate`: commit-readiness requires L1+L2=0 on namespaced wat-rs homes. The bar is exceptional; the path to it is BOTH amend layers.

## What this stone delivers

### Layer 1 — Vigilia amends (already on disk uncommitted; carry forward)

| # | Site | Amend |
|---|---|---|
| A1 | `src/argspec/error.rs` | `fn classify(self) -> (Span, String, String)` extracted on `ArgSpecError`; three `From<>` impls collapse to mechanical 4-line wrappers |
| A2 | `src/argspec/parse.rs` | `fn parse_keyword_type<F>(ast, head, non_keyword_err)` extracted; ONE call site after Layer 2 (just the fixed-param slot — ret-type slot vanishes) |
| A3 | `src/argspec/parse.rs:88-90` + struct field | `rune:purgare(future-fixture)` on `unreachable!` arm AND on `ArgSpec::rest_param` field |
| A4 | `tests/probe_arc241_stone1_argspec_canonical.rs:25-35` | Owned `(Vec<WatAST>, wat::span::Span)` return replaces `impl Deref<Target=Span>` |

### Layer 2 — Scope correction (NEW; strip ret-clause concerns)

| # | Site | Strip |
|---|---|---|
| S1 | `src/argspec/parse.rs` ArgSpec struct | Remove `pub ret_type: Option<TypeExpr>` field |
| S2 | `src/argspec/parse.rs` ParseOptions struct | Remove `pub include_ret_type: bool` field |
| S3 | `src/argspec/error.rs` ArgSpecError enum | Remove `MissingRetArrow` and `RetTypeNotKeyword` variants |
| S4 | `src/argspec/error.rs` classify() | Remove match arms for the two removed variants |
| S5 | `src/argspec/parse.rs` parse_argspec_triples body | Remove the entire `if options.include_ret_type {...}` block (post-loop section ~lines 137-178); loop becomes the whole walker |
| S6 | `src/argspec/parse.rs` parse_argspec_triples loop | Remove the `if is_bare_symbol(args_vec[idx], "->")` break (no longer a terminator within argspec; if `->` shows up, it's a `TrailingItems` / `NameNotSymbol` depending on position) |
| S7 | `src/argspec/mod.rs` module doc | Update to clarify: argspec parses ONLY the canonical triple; ret-clause is fn-form-parser concern (241.2 callers compose) |
| S8 | `tests/probe_arc241_stone1_argspec_canonical.rs` | Remove contracts 03, 04, 08, 09, 12 (the 5 ret-related); ret-related fixtures vanish; `parse_triples` helper signature loses `include_ret_type` param; remaining contracts renumber 01–08 |

### NOT in scope

- **`parse_ret_clause` is NOT minted in this stone.** Stone 241.2 (A1/A2/A3 migration) handles ret-clause parsing — either inlines it per-caller OR mints a sibling helper at that point. Stone 241.1.fix's mandate is "make argspec exceptional"; ret-clause is fn-form-parser concern.
- **A1/A2/A3/A4 callers UNTOUCHED.** Stone 241.2/3 migration territory.
- **`&` rest-binder UNTOUCHED.** Stone 241.4.
- Acceptable deferrals from prior Phase B remain: per-helper `#[test]` for probe helpers (thin-wrapper L3 per complectens SKILL); Span re-export from `wat::argspec` (architectural exemption per vocare).

## Locked decisions

### D1 — `classify()` returns domain-neutral reason per variant (drift eliminated at source)

(Carried from prior DESIGN unchanged in shape; arm list shrinks per S4.)

After scope correction, the `classify()` covers 7 variants (was 9): `NameNotSymbol`, `MissingArrow`, `TypeNotKeyword`, `MalformedTypeKeyword`, `TrailingItems`, `IncompleteSignature`, `RestBinderNotSupported`. The two ret-variants vanish per S3.

Canonical reasons (locked, domain-neutral — no "arg-vector" / "field/arg" prefix; head field carries form context):

| Variant | Reason |
|---|---|
| `NameNotSymbol` | `"name slot must be a plain symbol (not a keyword, literal, or nested form)"` |
| `MissingArrow` | `"triple must be \`name <- :T\`; \`<-\` arrow not found at slot 1"` |
| `TypeNotKeyword` | `"type slot must be a keyword (e.g. \`:wat::core::i64\`); got a non-keyword"` |
| `MalformedTypeKeyword { inner, .. }` | `format!("type keyword is malformed: {inner}")` |
| `TrailingItems { count, .. }` | `format!("{count} trailing item(s) beyond the expected argspec shape")` |
| `IncompleteSignature` | `"triple is incomplete; expected \`name <- :T\` but ran out of items"` |
| `RestBinderNotSupported` | `"\`&\` rest-binder is not supported at this binding site"` |

Each From impl: 4-line wrapper consuming `err.classify()`.

### D2 — `parse_keyword_type` helper (now used by ONE site only)

After scope correction, only the fixed-param type slot routes through it. The ret-type call site vanishes per S5. The helper stays as-is (still load-bearing for the fixed-param slot; also forward-compatible for Stone 241.4's rest-binder type slot).

### D3 — Runes (unchanged from existing amends; both still load-bearing)

`rune:purgare(future-fixture)` on:
1. The `unreachable!` arm at parse.rs (still present; Stone 241.4 territory)
2. The `ArgSpec::rest_param` field (still present)

### D4 — Probe shape: owned span, not opaque trait (unchanged from existing amends)

`parse_vector_items` returns `(Vec<WatAST>, wat::span::Span)`. `parse_triples` helper signature loses `include_ret_type` param (it parsed only argspec now); signature becomes:

```rust
fn parse_triples(
    src: &str,
    allow_rest_binder: bool,
) -> Result<ArgSpec, ArgSpecError>;
```

### D5 — Probe contracts: 8 total (was 13)

| Contract | Variant tested | Source |
|---|---|---|
| 01 | empty argspec parses cleanly | `[]` |
| 02 | single fixed param parses | `[x <- :wat::core::i64]` |
| 03 | multiple fixed params parse | `[x <- :wat::core::i64 y <- :wat::core::i64]` |
| 04 | non-Symbol at name slot → `NameNotSymbol` | `[:keyword-not-symbol <- :wat::core::i64]` |
| 05 | missing `<-` arrow → `MissingArrow` | `[x = :wat::core::i64]` |
| 06 | non-Keyword at type slot → `TypeNotKeyword` | `[x <- "string-not-keyword"]` |
| 07 | `&` rest-marker rejected → `RestBinderNotSupported` | `[x <- :wat::core::i64 & rest <- :wat::core::Vector<:wat::core::i64>]` |
| 08 | malformed type keyword → `MalformedTypeKeyword` | `[x <- :Any]` (reject_any() fires at parse time) |

**REMOVED**: ret-related contracts (was 03 multi-with-ret, 04 ret-only, 08 missing-ret-arrow, 09 trailing-items-after-ret, 12 ret-not-keyword). All five exercised semantics that no longer belong to argspec.

**NEW probe size**: ~150 lines (down from 235); contracts each 8 lines body + attribute.

### D6 — `ArgSpec` final shape (post-scope correction)

```rust
pub struct ArgSpec {
    /// Ordered list of `(name, type)` pairs for the fixed positional parameters.
    pub fixed_params: Vec<(String, TypeExpr)>,
    /// Rest parameter `(name, type)`, populated by Stone 241.4.
    /// Always `None` in Stone 241.1.
    // rune:purgare(future-fixture) — Stone 241.4 populates rest_param via allow_rest_binder
    //                                path; field exists in 241.1 for API stability.
    pub rest_param: Option<(String, TypeExpr)>,
}
```

NO `ret_type` field. NO `include_ret_type` ParseOption.

### D7 — `ParseOptions` final shape (post-scope correction)

```rust
pub struct ParseOptions {
    /// Whether a `& name <- :T` rest-binder is permitted in the arg-vector.
    /// Always `false` in Stone 241.1. Stone 241.4 adds rest-binder logic;
    /// `defclause` callers set this `true` via 241.5.
    pub allow_rest_binder: bool,
}
```

ONE field. The struct stays (allow_rest_binder is per-site invariant).

### D8 — `ArgSpecError` final shape (post-scope correction)

```rust
pub enum ArgSpecError {
    NameNotSymbol { span: Span, head: String },
    MissingArrow { span: Span, head: String },
    TypeNotKeyword { span: Span, head: String },
    MalformedTypeKeyword { span: Span, head: String, inner: Box<TypeError> },
    TrailingItems { span: Span, head: String, count: usize },
    IncompleteSignature { span: Span, head: String },
    RestBinderNotSupported { span: Span, head: String },
}
```

7 variants (was 9). NO `MissingRetArrow`, NO `RetTypeNotKeyword`.

### D9 — Module doc inscribes the corrected scope

`src/argspec/mod.rs` doc comment updated to:
- Strip "include_ret_type" / "ret-type slot" framing
- Add: "Argspec parses ONLY the canonical `[name <- :T name <- :T ... [& rest <- :T]]` triple form. Ret-clause (`-> :Ret`) parsing belongs to fn-form parsers (defn, fn, fn type-signature) which compose argspec + ret-clause at the form-level."
- Reference FORM-COLLAPSE-NOTES.md:184 doctrine

### D10 — `parse_argspec_triples` final signature (unchanged from current)

```rust
pub fn parse_argspec_triples(
    args_vec: &[WatAST],
    head: &str,
    form_span: &Span,
    options: ParseOptions,
) -> Result<ArgSpec, ArgSpecError>;
```

Signature stays; body shrinks (the `if options.include_ret_type {...}` block goes away).

### D11 — Lib baseline preserved; probe shrinks 13→8

After Stone 241.1.fix:
- `cargo test --release --lib -p wat` = 834+ PASS / 0 FAIL
- `cargo test --release --test probe_arc241_stone1_argspec_canonical` = **8 PASS / 0 FAIL** (was 10 pre-amend, 13 post-amend, 8 post-scope-correction)
- `cargo build --release --tests --workspace` clean
- `cargo clippy --release` ≤ 905 warnings (baseline)

### D12 — Vigilia re-cast must converge L1+L2=0

Solvere's L2 (RetTypeNotKeyword conflation) vanishes structurally — the variant is gone. The 8-spell re-cast should produce CONVERGED on all spells.

---

## Trap-door audit

### T1 — Loop logic post-scope-correction

The current loop has a break on `is_bare_symbol(args_vec[idx], "->")` that terminates fixed-param parsing when `->` is encountered. After scope correction, `->` is no longer a terminator (argspec doesn't know about ret-clauses). Options:

- **(α)** Keep the break: a stray `->` inside argspec would silently terminate parsing — bad. REJECTED.
- **(β)** Remove the break: a stray `->` falls into the triple-walker; since `->` is a `WatAST::Symbol("->")`, slot 0 of the next triple sees it; the `WatAST::Symbol` matches `NameNotSymbol`'s NEGATIVE check (Symbol IS what we want for slot 0); so `name = "->"`; slot 1 expects `<-`; bare `->` won't match → fires `MissingArrow`. Reasonable behavior; the error names what went wrong.
- **(γ)** Reject `->` explicitly: add an explicit check at slot 0 — "the name slot must not be the `->` ret-arrow symbol"; mint a `RetArrowInArgspec` variant. ADDS variant complexity for a niche case.

**Verdict (β)**: Remove the break; rely on the generic `MissingArrow` error to surface the malformed shape. The caller (fn-form parser) is responsible for splitting the slice at `->` BEFORE calling argspec; if a stray `->` reaches argspec, the caller already malformed the input — argspec surfacing it as `MissingArrow` is honest.

### T2 — `TrailingItems` semantics post-scope-correction

Before: `TrailingItems` fired when items remained after the expected ret-type slot. After scope correction: `TrailingItems` is unreachable in 241.1 because the loop consumes all items (it only stops on rest-marker `&` or end-of-slice). Until Stone 241.4 ships rest-binder support, the loop ALWAYS consumes the full slice.

Options:

- **(α)** Remove `TrailingItems` variant + its handling: the variant becomes dead in 241.1.
- **(β)** Keep `TrailingItems` variant; rune-accept with `rune:purgare(future-fixture)`: Stone 241.4's rest-binder logic re-introduces the case where items might remain after the rest-binder triple — at that point `TrailingItems` becomes reachable.

**Verdict (β)**: Keep + rune. Stone 241.4 needs this variant; removing now and re-adding later is churn. Rune format: `// rune:purgare(future-fixture) — Stone 241.4 makes TrailingItems reachable after rest-binder logic ships; 241.1 loop consumes full slice.`

### T3 — `IncompleteSignature` rename consideration

The variant is named `IncompleteSignature` — "signature" implies fn-form, which we just stripped from argspec scope. The honest rename: `IncompleteTriple`. The semantic is identical (fewer than 3 items remain to form a triple).

**Verdict**: RENAME to `IncompleteTriple`. The variant name should be honest about what it parses (triples, not signatures). One additional touch.

### T4 — Layer-1 amends are atomic with Layer-2 amends in this stone

The vigilia amends (classify, parse_keyword_type, runes, probe refactor) and the scope correction land as ONE atomic commit. The intermediate state (vigilia amends with wrong-scope substrate) is NOT committed. Per `feedback_namespaced_home_vigilia_gate`: commit only when L1+L2=0; intermediate state at L1+L2=1 (solvere's L2) is uncommitted.

### T5 — Stone 241.2 prep — what the migration callers need

Stone 241.2 (A1/A2/A3 migration) now needs:
1. Split args_vec at `->` (find the arrow position)
2. Call `parse_argspec_triples` on prefix
3. Parse ret-clause on suffix: `[->, :Ret]` shape → consume `->` + parse `:Ret` keyword

The ret-clause parsing is either inlined per-caller OR factored into a small helper. Stone 241.2 decides; not 241.1.fix's concern.

### T6 — `head: &str` parameter stays despite ret-clause removal

`head` is the surface form name for error context (`":wat::core::defn"` etc). Stays uniform. Not affected by scope correction.

### T7 — Probe contract 11 (MalformedTypeKeyword via `:Any`) survives

The amend kept contract 11 (now renumbered to 08 in the post-correction probe). `[x <- :Any]` still triggers `reject_any()` at parse time; `MalformedTypeKeyword` is still a valid argspec variant (it covers the fixed-param type slot's parse-time rejection). Contract structure unchanged; just renumbered.

### T8 — The amends and scope correction compose cleanly

`classify()` arm list shrinks from 9 → 7 (drop MissingRetArrow + RetTypeNotKeyword arms). `parse_keyword_type` call-site count shrinks from 2 → 1 (only fixed-param slot). `parse_argspec_triples` body shrinks (the entire post-loop ret-clause block goes away). Probe shrinks 13 → 8 contracts. All layers integrate without conflict.

---

## STOP triggers (REJECTION — not permission to defer)

1. **STOP-1** — Unexpected compile errors not traced to amend-named sites
2. **STOP-2** — Lib baseline regression (current: 834 PASS / 0 FAIL; must hold ≥834)
3. **STOP-3** — 40 min elapsed (smaller scope; mechanical amends)
4. **STOP-4** — `holon-rs` touched (substrate is frozen)
5. **STOP-5** — Rust files outside `src/argspec/*` (3 files) + `tests/probe_arc241_stone1_argspec_canonical.rs` touched. `src/lib.rs` MUST stay unchanged.
6. **STOP-6** — Scope creep:
   - Migrating ANY of A1/A2/A3/A4 — that is 241.2/3
   - Implementing `&` rest-binder — that is 241.4
   - Minting `parse_ret_clause` — that is 241.2 caller territory
   - Adding NEW types or fields beyond the locked shape
7. **STOP-7** — Probe doesn't reach 8/8 PASS
8. **STOP-8** — Any prior arc 237 probe regresses
9. **STOP-9** — Clippy warnings increase above 905 baseline
10. **STOP-10** — Vigilia re-cast finds new L1/L2; surface as findings (re-amend before commit)

Each STOP is REJECTION criteria.

---

## FM 2-bis evidence

The existing probe at `tests/probe_arc241_stone1_argspec_canonical.rs` is mid-amend (Stone 241.1.fix Layer 1 applied; Layer 2 scope correction pending). The probe's CORRECTED shape (8 contracts) IS the substrate sonnet mirrors. Pre-amend (Stone 241.1 at HEAD `6621f2a2`): 10/10 PASS. Mid-amend (current uncommitted): 13/13 PASS. Post-scope-correction (target): 8/8 PASS — the 5 dropped ret-contracts become uncompilable (they reference `include_ret_type`, a field that's being removed), surfacing the scope correction at compile time.

---

## SCORE doc spec

`docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.1.fix.md` (NEW; the prior draft was discarded per scope evolution). Mirror SCORE-STONE-241.1.md structural shape:

- **Header**: status (Mode A/B); runtime; one-line summary covering BOTH layers
- **Phase A scorecard**: ~15 rows covering Layer 1 amends + Layer 2 scope correction
- **Final API signatures** — verbatim post-scope ArgSpec / ParseOptions / ArgSpecError shapes
- **Line counts per file** — actual deltas
- **Clippy delta** — 0
- **Lib baseline** — 834+ PASS / 0 FAIL
- **Probe**: 8/8 PASS
- **Workspace test-build**: clean
- **Honest deltas**
- **NO Vigilia Convergence section** — orchestrator inscribes after Phase B re-cast

---

## Calibration

**Target band:** 20–35 min Mode A (slightly higher than original 20-30 estimate due to Layer 2 substrate removal + probe restructure).
**Upper bound:** 40 min (STOP-3).

**Surface estimate (net delta significant code SAVED):**

| File | Pre-scope-correction | Post-scope-correction | Delta |
|---|---|---|---|
| `src/argspec/error.rs` | 133 (post-amend) | ~110 | **-23** (2 fewer variants + 2 fewer classify arms) |
| `src/argspec/parse.rs` | 212 (post-amend) | ~145 | **-67** (strip post-loop block; loop break removal) |
| `src/argspec/mod.rs` | 47 | ~50 | **+3** (doc update) |
| `tests/probe_arc241_stone1_argspec_canonical.rs` | 225 (post-amend) | ~155 | **-70** (drop 5 contracts; helper signature shrink) |
| **Net delta from Stone 241.1 baseline** | — | — | **~-240 lines** |

**Confidence: HIGH.** Mechanical strip; locked decisions; no new types; user verdict decisive.

---

## What this unblocks

Stone 241.2 — A1/A2/A3 migration. The fn-form parsers now compose:
1. Split args_vec at `->` arrow
2. `parse_argspec_triples(prefix_slice)` — the canonical surface
3. Parse ret-clause on suffix (inlined OR via a small helper minted in 241.2)

Argspec is exceptional. Ret-clause is a fn-form concern.

---

## Cross-references

- `SCORE-STONE-241.1.md` § Vigilia Convergence — the 4 L1 + ~12 L2 findings that drove Layer 1
- `FORM-COLLAPSE-NOTES.md:184` — the doctrinal source for argspec scope (parses ONLY canonical triples)
- User direction 2026-05-28 mid-day: *"Y — args have nothing to do with ret type"* — the scope correction verdict
- `feedback_trap_door_build_the_dependency` — drove the choice to strip rather than rename
- `feedback_namespaced_home_vigilia_gate` — the gate doctrine; L1+L2=0 is the bar
- `feedback_inscription_immutable` — DESIGN docs are LIVING (evolved here); SCOREs/INSCRIPTIONs are immutable
- COMPACTION-AMNESIA-RECOVERY § FM 13 — memory contradicts DESIGN → memory (and user verdict) wins
- `~/work/holon/datamancy/purgare/SKILL.md` — rune format reference
