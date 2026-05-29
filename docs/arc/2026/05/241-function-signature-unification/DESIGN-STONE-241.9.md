# DESIGN — Stone 241.9 — Phase 3 second: `:wat::core::defenum` HARD CUT (enum retires)

**Status:** READY (sub-DESIGN). Phase 3 second stone. **HARD CUT** — no shims; raw deletion of legacy `enum` (pair-form tagged variants + bare-keyword unit variants); mint `defenum` using FORM-COLLAPSE verdict D (positional variants with one-token look-ahead). Vigilia gate doctrine: `src/types.rs` is legacy flat substrate (D7 default) → no vigilia cast; commit on SCORE-green.

## Scope warning — substantial cascade expected

`grep -rl ":wat::core::enum\b" --include="*.rs" --include="*.wat" .` → **43 hits**. Some are WAT constructor calls or pattern matches that don't migrate (per Stone 241.8's WAT-source insight: declarations migrate; constructors don't). Net declaration sites estimated **~25-35** — similar order to 241.8's 33.

Per `docs/SUBSTRATE-AS-TEACHER.md`: the cascade IS the migration brief. Sonnet iterates from the diagnostic stream; fail-count is the progress meter. Predicted band: **60-120 min Mode A** (mirrors 241.8's band; per-site syntactic conversion is mechanical).

## Why this stone

Per FORM-COLLAPSE-NOTES verdict (D) (LOCKED via four-questions cast 2026-05-28):

```scheme
(:wat::core::defenum :app::Status
  {:variant-metadata {:Error {:doc "raised when the operation fails"}}}     ; OPTIONAL form-level
  :Ok                                                                       ; positional — unit variant
  :Pending                                                                  ; positional — unit variant
  :Error [code    <- :wat::core::i64                                        ; positional — tagged variant
          message <- :wat::core::String])                                   ; (followed by argspec Vector)
```

Replaces legacy:
- `(:wat::core::enum :Name :UnitV1 :UnitV2 (TaggedV (field :Type) ...))` — pair-form tagged variants; bare-keyword unit variants; no metadata

`defenum` uses arc 241's canonical `parse_argspec_triples` for tagged-variant argspec Vectors (proving the parser-unification work serves form-collapse — same pattern as 241.8's defstruct).

## What this stone delivers

### S1 — Mint `parse_defenum` at `src/types.rs`

New parser function. Algorithm:

1. **Args** (after head consumed by `parse_type_decl`):
   - `args[0]` — name keyword (e.g. `:app::Status`)
   - Optional `args[1]` — metadata-map `{...}` (WatAST::List with head `:wat::core::HashMap`); detected by first-position discriminator
   - Remaining args — positional variants

2. **Optional metadata-map** at `args[1]`:
   - Discriminator: `WatAST::List` with head `:wat::core::HashMap` → metadata; otherwise → variant
   - Recognize: `:variant-metadata {keyword → metadata-map}` (per-variant metadata)
   - Empty `{}` REJECTED per FORM-COLLAPSE-NOTES (parser layer; mirror 241.8 D4)

3. **Variant grammar — one-token look-ahead** (verdict D):
   - See `WatAST::Keyword` → variant name (strip `:`)
   - Peek next: `WatAST::Keyword` (or end of args) → current is UNIT variant
   - Peek next: `WatAST::Vector(items, _)` → current is TAGGED variant; consume Vector; parse via `crate::argspec::parse_argspec_triples(&items, HEAD, &vector_span, ParseOptions { allow_rest_binder: false })`
   - Convert `ArgSpec.fixed_params` (Vec<(String, TypeExpr)>) → `EnumVariant::Tagged { name, fields }`

4. **Per-variant metadata association**:
   - For each variant by keyword, check `:variant-metadata`'s inner map by keyword
   - Storage TBD — `EnumDef` does not currently carry variant-level metadata; sonnet either (a) extends `EnumDef` with optional `variant_restrictions: HashMap<String, ...>` or (b) silently stores under generic-metadata. **D5 below: silent generic storage** (no consumer-driven extension this stone).

5. **Empty variant list REJECTED** (preserve legacy `parse_enum` invariant — enum must have ≥ 1 variant).

6. **Build `TypeDef::Enum(EnumDef)`** with variants populated from positional walk.

### S2 — Delete `parse_enum`, `parse_enum_variant` (HARD CUT)

- `src/types.rs:2250` `parse_enum` — DELETE
- `src/types.rs:2575` `parse_enum_variant` — DELETE
- `src/types.rs:1900` dispatch arm `"enum" => parse_enum(...)` — DELETE
- Add new dispatch arm: `"defenum" => parse_defenum(iter.collect(), decl_span)`

### S3 — `parse_field` retirement audit

Per legacy `parse_enum_variant:2624`: `fields.push(parse_field(item)?)`. After Stone 241.9 HARD CUT, `parse_field`'s only caller path retires. Sonnet investigates whether `parse_field` becomes orphaned:
- If orphaned → DELETE (HARD CUT extends to dead-code removal per `feedback_substrate_naming_audit`)
- If still called by another path → KEEP and document

### S4 — Macro detection update

Whatever path lit up `"enum"` for parser dispatch (likely `src/types.rs:1864-1870`-style match arm or `src/special_forms.rs` registry):
- Remove `:wat::core::enum` recognition
- Add `:wat::core::defenum` recognition

Mirror 241.8's check.rs HARD-CUT-rejection arm: callers using `:wat::core::enum` get an honest "form retired; use :wat::core::defenum" error (or `MalformedForm` per substrate convention).

### S5 — Migration cascade (~25-35 declaration sites)

Per substrate-as-teacher: the diagnostic stream drives migration. Per-site conversions:

**Bare-keyword unit variant — DROP-IN MIGRATION** (already positional):

```scheme
;; LEGACY (still positional unit; trivial change)
(:wat::core::enum :Status :Ok :Pending :Error)

;; NEW (defenum; same positional unit-variant grammar)
(:wat::core::defenum :Status :Ok :Pending :Error)
```

**Pair-form tagged variant — argspec triple migration:**

```scheme
;; LEGACY
(:wat::core::enum :Status
  :Ok
  (Error (code :wat::core::i64) (message :wat::core::String)))

;; NEW (defenum; tagged variant uses argspec Vector triples)
(:wat::core::defenum :Status
  :Ok
  :Error [code    <- :wat::core::i64
          message <- :wat::core::String])
```

Mechanical per-site (head rename + tagged-variant List `(Name (f :T) ...)` → keyword + Vector triples).

## Locked decisions

### D1 — HARD CUT: no shims, no compatibility aliases

`:wat::core::enum` ceases to exist post-stone. Any caller using it gets an honest form-retired error. Migration is forward-only. Mirror 241.8 D1.

### D2 — `parse_argspec_triples` reuse for tagged variants

The argspec Vector inside a tagged variant parses via the canonical parser from Stone 241.1. `ParseOptions { allow_rest_binder: false }` (tagged-variant fields are NOT variadic). This validates the canonical parser's design across form-collapse — same parser; third binding site (after fn-form 241.2, defclause 241.3, defstruct 241.8). Mirror 241.8 D2.

### D3 — Variant grammar: positional + one-token look-ahead (verdict D)

Per FORM-COLLAPSE-NOTES four-questions cast 2026-05-28:
- Variants are POSITIONAL keywords; no outer Vector wrap
- Look-ahead one token to discriminate UNIT vs TAGGED:
  - Next is keyword (or end-of-args) → current is UNIT
  - Next is Vector `[...]` → current is TAGGED; consume Vector as argspec

Rejected candidates A, B, C documented in FORM-COLLAPSE-NOTES § "Variant grammar."

### D4 — Empty `{}` metadata REJECTED

Mirror 241.8 D4. Empty metadata-map is illegal per Stone 241.6 doctrine. defenum inherits.

### D5 — Per-variant metadata storage: silent generic

For now: per-variant metadata under `:variant-metadata` is silently accepted into `EnumDef` (storage TBD — either extend the schema with optional `variant_metadata: HashMap<String, HolonAST>` or store under a generic-metadata bucket). No consumer-driven semantic this stone; the storage is the surface contract.

If `:variant-metadata` access becomes load-bearing later, an extension stone mints the consumer.

### D6 — Vigilia gate doctrine

`src/types.rs` is legacy flat substrate (NOT a `src/<noun>/` namespaced home). Per `feedback_namespaced_home_vigilia_gate` D7 default: no vigilia cast. Commit on SCORE-green. Mirror 241.8 D6.

### D7 — Probe + lib + arc 241.1-241.8 probes preserved

After Stone 241.9:
- `cargo test --release --lib -p wat` ≥ 834 PASS / 0 FAIL (post-migration; tests using legacy enum syntax updated)
- Stone 241.1/2/3/4/5/6/7/8 probes preserved at PASS counts
- Stone 241.9 probe ≥ 7/7 PASS (6 success-path + 2 HARD-CUT rejection contracts; see § FM 2-bis evidence)
- Arc 237/238 probes preserved

### D8 — `parse_field` retirement audit

Per S3. Sonnet investigates; either DELETE if orphaned or KEEP with documentation. No predetermined verdict.

## Trap-door audit

### T1 — Cascade size uncertain

43 grep hits include WAT constructor calls (`(:Status::Ok)`-style) + pattern matches that don't migrate. Net declaration sites estimated ~25-35. Sonnet's diagnostic stream confirms.

### T2 — Test files testing legacy enum semantics

Files like `tests/wat_user_enums.rs` and `tests/probe_let_splice_enum.rs` literally test enum behavior. Tests migrate to defenum semantics; test intent (verifying enum behavior) preserved. Mirror 241.8 T2.

### T3 — `parse_field` shared with prior parsers

`parse_field` may still be referenced from removed-by-241.8 code OR by other parsers. Sonnet greps `parse_field` callers; if zero post-241.9, DELETE.

### T4 — `EnumDef` schema extension for `:variant-metadata`

If sonnet chooses (a) explicit `variant_metadata` field on `EnumDef`, that's a substrate schema change with downstream consumers (encoder, signature reflection, etc.). Recommend (b) silent generic storage this stone; defer schema extension to a consumer-driven future stone (named follow-up). Mirror 241.8's D5 simple-storage path.

### T5 — `:variant-metadata` inner-map key syntax (parser routing)

**Mirror 241.8's trap-door T-fd verbatim:** `{bareSymbol {submap}}` routes to struct-destructure (Stone 241.8 SCORE § "Honest Deltas"). `:variant-metadata` inner keys MUST use keyword syntax (`:Error` not `Error`). Probe contracts 03/04 reflect this; FORM-COLLAPSE-NOTES already uses keyword syntax in examples.

### T6 — Multi-spawn potential

Stone 241.9 may warrant TWO sonnet flights: (a) substrate mint + macro routing; (b) cascade migration. Orchestrator decides post-first-spawn if needed. Mirror 241.8 T3.

### T7 — WAT source files (constructor + match sites)

Per 241.8 SCORE: WAT files with `(:Variant ...)` constructors or `(:match ...)` patterns do NOT migrate. Only DECLARATION sites `(:wat::core::enum :Name ...)` migrate. Sonnet distinguishes per file.

## STOP triggers

1. Compile errors not traced to migration sites
2. Lib < 834 (after cascade migration; failures during cascade are expected per substrate-as-teacher; END STATE must be 834)
3. **150 min elapsed** (upper bound; ~30% over 241.8's 120-min ceiling to account for slightly larger cascade + storage decision)
4. holon-rs touched
5. Files outside `src/types.rs`, `src/runtime.rs` (if needed for register/check), `src/check.rs`, `src/special_forms.rs`, `src/closure_extract.rs`, `src/freeze.rs` (mirroring 241.8's substrate surface), `tests/probe_arc241_stone9_*`, the cascade target files, SCORE doc
6. Scope creep: `define ⇒ defn` (241.10); INSCRIPTION (241.11); new type-system features beyond defenum semantics; `:variant-metadata` consumer-driven semantics
7. Stone 241.9 probe < 7/7 PASS
8. Stone 241.1/2/3/4/5/6/7/8 probes regress; arc 237/238 probes regress
9. Clippy > 902 (matches 241.8 ceiling)
10. `feedback_no_semantic_abuse_of_option` violation: if sonnet reaches for `Option<HashMap>` flavor-encoding for variant-metadata storage, STOP — silent generic storage per D5

## FM 2-bis evidence

`tests/probe_arc241_stone9_defenum.rs` (NEW). 7 contracts. At HEAD: contracts 01-05 FAIL (defenum verb doesn't exist); contracts 06-07 FAIL (legacy enum still works). Post-stone: all 7 PASS.

**Contracts:**

1. **C01** — `(defenum :T :A :B :C)` plain unit-only enum
2. **C02** — `(defenum :T :A :Tagged [f <- :i64])` mixed unit + tagged
3. **C03** — `(defenum :T :A :T1 [x <- :i64 y <- :i64] :B :T2 [s <- :wat::core::String])` interleaved
4. **C04** — `(defenum :T {:variant-metadata {:A {:doc "..."}}} :A :B)` with form-level metadata
5. **C05** — Empty `{}` REJECTED (D4)
6. **C06** — Legacy `(:wat::core::enum :T :A :B)` HARD CUT rejected
7. **C07** — Legacy `(:wat::core::enum :T (Tagged (f :Type)))` HARD CUT rejected

Probe disconfirms at HEAD; passes post-stone. Per FM 2-bis: probe committed BEFORE BRIEF; BRIEF cites it verbatim.

## Calibration

**Target band: 60-120 min Mode A.**
**Upper bound: 150 min (STOP-3).**

| File | Pre | Post | Delta |
|---|---|---|---|
| `src/types.rs` (mint + delete legacy) | (~+150 −105 net) | similar | ~+45 net |
| `tests/probe_arc241_stone9_defenum.rs` (NEW) | 0 | ~180 | **+180** |
| ~25-35 migration target files | various | various | **per-site syntactic conversion** |
| Possible `parse_field` deletion | (kept) | (deleted if orphaned) | **~-40** if deleted |
| **Net delta** | — | — | **substantial mixed** |

**Per `feedback_stone_briefs_cite_prior_score`:** Stone 241.9 BRIEF cites `SCORE-STONE-241.8.md` as the migration shape sonnet mirrors (same HARD CUT cadence; same canonical-parser reuse; same metadata-map mechanism; per-variant trap-door echoes per-field trap-door).

## What this unblocks

**Stone 241.10** — `define ⇒ defn` HARD CUT. With defstruct + defenum HARD-CUT clean, the def*-prefix family converges; `define` retirement is the closure-prep stone.

**Stone 241.11** — INSCRIPTION closes arc 241. Per `feedback_no_regression_until_arc_done`: arc 237.8b reopens AFTER 241.11.

---

**Recommendation**: this DESIGN is committed for review. Stone 241.9's full open (probe + BRIEF + EXPECTATIONS + spawn) commits to ~60-120 min of sonnet work + verification cycles. Cascade scope similar to 241.8 (which shipped at ~41 min — UNDER band). Predicted runtime favorable.
