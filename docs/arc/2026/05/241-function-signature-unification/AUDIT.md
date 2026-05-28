# Arc 241 Stone 241.0 — AUDIT (argspec-parser sites)

**Status:** READ-ONLY enumeration. No substrate change. Bounds the unknown for stones 241.1-241.5.

**Method:** grep `parse_fn_signature|parse_defclause_args|parse_argspec|<-\s*:` + manual read of each surfaced site. The cited 4 parsers in arc 241 DESIGN are verified authoritative; THIS AUDIT surfaces THREE MORE near-variants the DESIGN did not name.

---

## Catalog — every site that parses or walks the canonical `name <- :T` triple

### A. Authoritative validating parsers (CONFIRMED — the DESIGN-cited 4)

| # | Site | Function | Returns | Caller(s) |
|---|---|---|---|---|
| **A1** | `src/runtime.rs:6750` | `parse_fn_signature` | `Result<(Vec<String>, Vec<TypeExpr>, TypeExpr), RuntimeError>` | `eval_fn` (runtime.rs:6696), `try_parse_fn_shape_def` (3913, 4000) |
| **A2** | `src/check.rs:15205` | `parse_fn_signature_for_check` | `Result<(Vec<String>, Vec<TypeExpr>, TypeExpr), ()>` | check.rs:9592 (silent-path infer_fn) |
| **A3** | `src/check.rs:15258` | `parse_fn_signature_for_check_diag` | `Option<(Vec<String>, Vec<TypeExpr>, TypeExpr)>` (errors pushed by-ref) | check.rs:15154 (diagnostic-path infer_fn) |
| **A4** | `src/runtime.rs:6880` | `parse_defclause_args` | `Result<Vec<(String, TypeExpr)>, RuntimeError>` (no ret-type slot) | `parse_defclause_clause` (runtime.rs:7000) |

### B. Near-variants — same `<-` arrow shape, prefix-decorated chunks (NEW finding; DESIGN didn't enumerate)

| # | Site | Surrounding fn | Chunk shape | Returns |
|---|---|---|---|---|
| **B1** | `src/types.rs:2002-2112` | `parse_struct_restricted` (restricted section) | `[wlist] field <- :T` — flat chunks of **4** | side-effects `fields: Vec<(String, TypeExpr)>` + `field_restrictions: HashMap` |
| **B2** | `src/types.rs:2114-2160` | `parse_struct_restricted` (public section) | `field <- :T` — flat chunks of **3** | side-effects `fields: Vec<(String, TypeExpr)>` |

### C. Runtime mirrors of B (eval-time accessor synthesis; same chunk shape)

| # | Site | Surrounding context | Note |
|---|---|---|---|
| **C1** | `src/runtime.rs:3641-3676` | accessor-synthesis walker for struct-restricted restricted-section (chunks of 4) | only extracts field-name for accessor minting; tolerant |
| **C2** | `src/runtime.rs:3677-3712` | accessor-synthesis walker for struct-restricted public-section (chunks of 3) | same — tolerant |

### D. Tolerant arg-name walkers (consume same shape; non-validating; one-direction)

| # | Site | Surrounding fn | Direction | Note |
|---|---|---|---|---|
| **D1** | `src/closure_extract.rs:710-741` | `walk_fn_form` | READ → names | every-3rd-item starting at 0; tolerant skips |
| **D2** | `src/closure_extract.rs:2429-2476` | `function_to_fn_form` | EMIT triples | constructive; builds `[name <- :T ...]` from `Function` |
| **D3** | `src/runtime.rs:13817` | `eval_extract_arg_names` (`:wat::runtime::extract-arg-names`) | READ → names | operates on **HolonAST** (post-parse), not WatAST |
| **D4** | `src/runtime.rs:13899` | `eval_extract_arg_types` (`:wat::runtime::extract-arg-types`) | READ → types | sibling of D3; HolonAST layer |

### E. Diagnostic-only walker (not a real parser)

| # | Site | Surrounding fn | Note |
|---|---|---|---|
| **E1** | `src/check.rs:3199-3293` | `check_legacy_user_main_signature` | walks BOTH `(name :T)` pair-form (legacy `define`) AND `[name <- :T ...]` triple-form (`defn`); fires the arc-170-slice-1e ambient-main diagnostic only |

### F. Out of scope — different shape entirely

- `src/types.rs:1917` `parse_struct` — plain struct uses `(name :T)` **pair-Lists**, not flat `<-`-triples. Not in the unification target.
- Legacy `(:wat::core::define (name :T) (name :T) ...)` — same pair-List shape; checked only via E1's tolerant walker.

---

## Per-site invariants table (the load-bearing matrix for 241.1's `ParseOptions`)

| Property | A1 (fn rt) | A2 (fn chk) | A3 (fn diag) | A4 (defclause) | B1 (rt-attrs) | B2 (pub-attrs) |
|---|---|---|---|---|---|---|
| **Outer arity check** | `args.len()==4` | `args.len()==4` | `args.len()==4` | none (caller arity) | `len % 4 == 0` | `len % 3 == 0` |
| **Includes ret-type slot?** | yes (`-> :Ret`) | yes | yes | NO | NO | NO |
| **Has leading prefix-slot?** | NO | NO | NO | NO | YES (`[wlist]` Vector) | NO |
| **Name slot kind** | Symbol → name | Symbol → name | Symbol → name | Symbol → name (**explicit contract; arc 159/169/234**) | bare Symbol → name (no leading colon) | bare Symbol → name |
| **Non-Symbol at name slot** | MalformedForm | silent `Err(())` | push CheckError; return None | MalformedForm **citing arc lineage** | MalformedDecl | MalformedDecl |
| **Type slot kind** | Keyword → parse_type_keyword | Keyword → parse_type_expr | Keyword → parse_type_expr | Keyword → parse_type_keyword | Keyword → parse_type_expr_with_span | Keyword → parse_type_expr_with_span |
| **Arrow token** | bare Symbol `<-` | bare Symbol `<-` | bare Symbol `<-` | bare Symbol `<-` | bare Symbol `<-` | bare Symbol `<-` |
| **Error enum class** | `RuntimeError` | `()` (silenced) | `CheckError` | `RuntimeError` | `TypeError` | `TypeError` |
| **Error message tag** | `":wat::core::fn"` | (none) | `":wat::core::fn"` | head (`":wat::core::defclause"` etc.) | `"struct-restricted"` | `"struct-restricted"` |
| **Rest-binder (`&`) supported?** | **NO** | NO | NO | **NO** ← THE GAP arc 241 closes | NO | NO |

### Cross-cutting invariants (universal — all six)

- The arrow token is **bare Symbol `<-`** at every site. No site uses a Keyword variant.
- Name slot is **always validated as Symbol** when validation happens at all (A2 is silent; D-family is tolerant; everyone else errors).
- Type slot is **always a Keyword** parsed by some `parse_type_*` helper. Three different helper names (`parse_type_keyword`, `parse_type_expr`, `parse_type_expr_with_span`) — the variation is real (parse_type_keyword vs parse_type_expr; with-vs-without span); not just naming drift.

---

## The error-class divergence finding (NEW; not in DESIGN)

The same structural failure ("name slot is not a Symbol") produces **three different error enum variants** across the six parsers:

- **A1 + A4** → `RuntimeError::MalformedForm`
- **A2** → `()` (silenced; caller falls through to None)
- **A3** → `CheckError::MalformedForm` (pushed into `&mut Vec<CheckError>`)
- **B1 + B2** → `TypeError::MalformedDecl`

This is the deeper duplication. The DESIGN named "parser divergence across binding sites"; the audit confirms divergence runs all the way down to the **error-enum class** — not just message wording. A consolidated `parse_argspec_triples` MUST emit ONE error type at its boundary; callers convert at their site boundary (RuntimeError → CheckError for the check path; RuntimeError → TypeError for the type-decl path). This is a structural improvement orthogonal to the rest-binder feature.

**Recommendation for 241.1:** mint canonical `ArgSpecError` (a small sum) at the parser boundary; provide `From<ArgSpecError>` impls for `RuntimeError`, `CheckError`, and `TypeError` so callers convert at their site. The conversions are mechanical; the canonical error type stays the single source of truth for malformedness shape.

---

## In-scope vs adjacent — proposed 241 scope

### In scope (241.1-241.5) — the four authoritative parsers

A1 + A2 + A3 + A4. These four all consume the same canonical `name <- :T` triple shape inside a Vector. Per-site invariants reduce to **two ParseOptions axes**:

- `include_ret_type: bool` (A1/A2/A3 = true; A4 = false)
- `error_kind: ParserErrorMode { Strict, SilentResult, DiagnosticPush(&mut Vec<CheckError>) }`

The `name_symbol_only` field in the DESIGN's `ParseOptions` is **always true** at every authoritative site → it should NOT be an option; it should be the unconditional canonical-parser contract. The DESIGN's framing of it as configurable is over-specified; can simplify.

`allow_rest_binder: bool` is the new axis 241.5 adds. A1/A2/A3/A4 all set it false today; defclause (A4) opts in via 241.5.

### Adjacent (NOT in 241 MVP; surface for follow-up arc)

**B1 + B2 + C1 + C2 — struct-restricted parsers** consume a **near-variant**: chunks-of-4 (with leading `[wlist]` Vector prefix) or chunks-of-3, instead of the canonical loose triples-in-a-Vector. The inner `name <- :T` triple is identical; the chunking + prefix-slot are different.

Two options for follow-up:
- **(α)** extend canonical parser with `prefix_kind: Option<PrefixKind>` and `chunk_arity: ChunkArity` → routes B1/B2 through it; their B-specific chunking is encoded in the ParseOptions
- **(β)** keep B's chunk-walker outside; only its inner `name <- :T` triple-extraction calls a canonical `parse_one_triple(items: &[WatAST; 3])` helper

(β) is honest about the structural difference (struct-restricted's outer shape really IS different); (α) bends the canonical parser into a universal walker. **Recommend (β)** for a follow-up arc per "removal of options"; the universal walker (α) becomes a god-function. NOT in 241 scope. Surface as `[follow-up arc] struct-restricted argspec sub-helper extraction`.

**D1/D2 — closure_extract walkers** are tolerant + one-direction (D1 reads, D2 emits). The READ walker (D1) could share canonical's name-extraction logic but its tolerance is intentional (silent skip vs error). The EMIT walker (D2) is print-side — symmetric concern. Both could route through a `format_argspec_triples(spec: &ArgSpec) -> WatAST::Vector` printer once `ArgSpec` exists. **Surface as follow-up; not in 241 MVP.**

**D3/D4 — reflection-layer extract-arg-names/types** operate on **HolonAST** (Bundle children), not WatAST. Different transport; orthogonal concern. Out of 241 scope entirely.

**E1 — legacy-main diagnostic walker** is a one-time arc-170-slice-1e ambient-main diagnostic; tolerant + dual-shape (handles legacy define AND defn). Should NOT route through canonical (it intentionally accepts both shapes). Out of scope.

---

## Open question for the orchestrator (for 241.1 BRIEF)

The DESIGN's `ParseOptions.name_symbol_only` is unused (every authoritative site already requires Symbol). Either:
- **drop it** — the canonical contract is "name MUST be Symbol; non-Symbol is always MalformedForm"
- **keep it as forward-compat** — future binding sites that allow destructuring patterns at the name slot could set `false`

The "removal of options" philosophy favors dropping. Forward-compat for destructuring is hypothetical (no current consumer needs it; the destructure-at-let-binding is a different form entirely — arc 159 / 169). **Recommend drop.** Re-introduce if/when a real consumer surfaces (no need to pre-build the dependency for nobody).

---

## Confirmed for the consolidation plan

- **Canonical parser signature** (lock):
  ```rust
  pub fn parse_argspec_triples(
      args_vec: &[WatAST],
      head: &str,
      form_span: &Span,
      options: ParseOptions,
  ) -> Result<ArgSpec, ArgSpecError>;
  ```
- **ArgSpec**:
  ```rust
  pub struct ArgSpec {
      pub fixed_params: Vec<(String, TypeExpr)>,
      pub rest_param: Option<(String, TypeExpr)>,  // None pre-241.5
      pub ret_type: Option<TypeExpr>,              // None for defclause sites
  }
  ```
- **ParseOptions** (lean):
  ```rust
  pub struct ParseOptions {
      pub include_ret_type: bool,    // fn = true; defclause = false
      pub allow_rest_binder: bool,   // 241.5 only; default false
  }
  ```
- **ArgSpecError**: small sum (incomplete trailing, non-Symbol at name, missing `<-` arrow, non-Keyword at type, missing `->` between args + ret, non-Keyword at ret-type, malformed-type-keyword wrapped); each variant carries `span: Span`.
- **Callers convert** at their boundary: A1/A4 → `RuntimeError::MalformedForm`; A2 → silenced (drop); A3 → push CheckError.

---

## Scope contract (241 won't drift)

- 241.1 mints `argspec.rs` + tests; old parsers untouched
- 241.2/3/4 migrate A1/A2/A3 + A4 one-by-one; tests stay green
- 241.5 adds `& rest <- :Vector<T>` to canonical parser; A4's caller (`parse_defclause_clause`) opts in via `ParseOptions::allow_rest_binder = true`; probe 237.8b Gate 1 flips green
- 241.6 INSCRIPTION + memory mint

**B/C/D/E sites are NOT migrated in 241.** They are documented here for a follow-up arc; pulling them into 241 would balloon scope past the recipe-lock unblock.

---

## Cross-references

- `DESIGN.md` (this arc) — class-elimination shape; this AUDIT confirms its 4-parser core AND surfaces 2 near-variants + 4 tolerant walkers + 1 diagnostic the DESIGN didn't enumerate
- `docs/arc/2026/05/237-polymorphism-consolidation/PAUSE-CONTEXT.md` — the 237.8b blocker that drove this arc
- `tests/probe_arc237_8b_defclause_arithmetic.rs` Gate 1 — empirical RED at HEAD; flips green at 241.5
- `/home/watmin/work/holon/scratch/FAILURE-ENGINEERING.md` — the discipline (eliminate the CLASS; here the class is parser divergence across binding sites)
- `feedback_wat_llm_first_design` — one-canonical-path principle; materialized by this arc for argspec parsing
