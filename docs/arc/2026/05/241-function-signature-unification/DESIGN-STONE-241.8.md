# DESIGN — Stone 241.8 — Phase 3 opens: `:wat::core::defstruct` HARD CUT (struct + struct-restricted retire)

**Status:** READY (sub-DESIGN). Phase 3 first stone. **HARD CUT** — no shims; raw deletion of legacy `struct` + `struct-restricted`; mint `defstruct` using the metadata-map mechanism (Stone 241.6/7). Vigilia gate doctrine TBD (types.rs is namespaced-adjacent but not src/<noun>/; default no gate).

## Scope warning — substantial cascade expected

**35 files** in `src/`, `tests/`, and `wat/` reference `:wat::core::struct` or `:wat::core::struct-restricted` per `grep -rl ":wat::core::struct\b\|:wat::core::struct-restricted"`. HARD CUT means ALL of them migrate to `:wat::core::defstruct`.

Per `docs/SUBSTRATE-AS-TEACHER.md`: the cascade IS the migration brief. Sonnet iterates from the diagnostic stream; fail-count is the progress meter. Predicted band: **60-120 min Mode A** (substantially larger than the recent ~10-30 min stones).

This stone may warrant a multi-spawn cycle (substrate mint first; then cascade migration as a second sonnet flight). Surfaced here for user review.

## Why this stone

Per FORM-COLLAPSE-NOTES (LOCKED via intueri cast 2026-05-28; δ verdict on per-field metadata):

```scheme
(:wat::core::defstruct :my::ns::MyType
  {:restricted-to  [:my::ns::]                          ; ctor restriction (form-level)
   :field-metadata {field1 {:restricted-to [:my::ns::]} ; per-field metadata
                    field2 {:restricted-to [:my::ns::]}}}
  [field1 <- :T1                                        ; argspec RIGID 3-slot triples
   field2 <- :T2
   field3 <- :T3])                                      ; no entry in :field-metadata = public access
```

Replaces TWO legacy forms with ONE:
- `(:wat::core::struct :name (field :Type) ...)` — pair-form fields; no restrictions
- `(:wat::core::struct-restricted :name [ctor-wlist] (restricted-section) (public-section))` — 4-arg shape; arc 203 restrictions

Both retire. `defstruct` uses arc 241's canonical `parse_argspec_triples` for the field-vector (proving the parser-unification work serves form-collapse).

## What this stone delivers

### S1 — Mint `parse_defstruct` at `src/types.rs`

New parser function. Algorithm:

1. **Arity discriminator**: items length tells the form variant
   - 3 items after head: `(defstruct :name {} [fields])` — invalid (empty `{}` rejected)
   - 3 items after head: `(defstruct :name {metadata} [fields])` — discriminate items[1] head keyword `:wat::core::HashMap`
   - 2 items after head: `(defstruct :name [fields])` — no metadata; valid

2. **Optional metadata-map** at items[1]:
   - If `WatAST::List` with head `:wat::core::HashMap` → extract entries
   - Recognize: `:restricted-to [keyword-list]` (form-level ctor restriction); `:field-metadata {symbol → metadata-map}` (per-field metadata)
   - Empty `{}` REJECTED per FORM-COLLAPSE-NOTES (parser layer)

3. **Field-vector** at items[N]:
   - `WatAST::Vector(items, span)` → pass items to `crate::argspec::parse_argspec_triples(items, head, form_span, ParseOptions { allow_rest_binder: false })`
   - Convert returned `ArgSpec.fixed_params` to `StructDef.fields`

4. **Per-field metadata association**:
   - For each field name in argspec, check `:field-metadata`'s inner map by symbol
   - Associate per-field restrictions/metadata via `StructDef.restrictions` (existing arc 203 structure; sonnet investigates extension)

5. **Build `TypeDef::Struct(StructDef)`** with restrictions populated from form-level + field-level metadata

### S2 — Delete `parse_struct` and `parse_struct_restricted` (HARD CUT)

`types.rs:1917` and `types.rs:1956` — delete entirely. Their macro dispatch entries at `types.rs:1864-1870` (`":wat::core::struct"` and `":wat::core::struct-restricted"` arms) also delete.

Add NEW dispatch entry: `":wat::core::defstruct" => "defstruct"` (or similar — sonnet matches the pattern).

### S3 — Migration of 35 files

Per substrate-as-teacher: the diagnostic stream drives migration. Sonnet runs `cargo test`; each failure points at a site using `:wat::core::struct` or `:wat::core::struct-restricted`; each migrates to `:wat::core::defstruct` with appropriate syntax conversion:

**`:wat::core::struct` pair-form → `:wat::core::defstruct` triple-form:**

```scheme
;; LEGACY
(:wat::core::struct :my::T (field1 :T1) (field2 :T2))

;; NEW (defstruct; argspec triples)
(:wat::core::defstruct :my::T [field1 <- :T1 field2 <- :T2])
```

**`:wat::core::struct-restricted` 4-section form → `:wat::core::defstruct` with metadata:**

```scheme
;; LEGACY
(:wat::core::struct-restricted :my::T
  [:my::ns]                                          ; ctor whitelist
  ([wlist] field1 <- :T1)                            ; restricted section
  (field2 <- :T2))                                   ; public section

;; NEW (defstruct; metadata-map + argspec)
(:wat::core::defstruct :my::T
  {:restricted-to  [:my::ns]
   :field-metadata {field1 {:restricted-to [:my::ns]}}}
  [field1 <- :T1
   field2 <- :T2])
```

Mechanical syntactic conversion per site.

### S4 — Macro detection update (`types.rs:1864-1870`)

```rust
match keyword {
    ":wat::core::defstruct" => return Some("defstruct"),
    // ":wat::core::struct" and ":wat::core::struct-restricted" arms DELETED
    ":wat::core::enum" => return Some("enum"),
    // (defenum HARD CUT is Stone 241.9; struct continues to NO LONGER WORK after this stone)
    ...
}
```

## Locked decisions

### D1 — HARD CUT: no shims, no compatibility aliases

`:wat::core::struct` and `:wat::core::struct-restricted` cease to exist post-stone. Any caller using them gets `Unknown form` or similar error. Migration is forward-only.

### D2 — `parse_argspec_triples` reuse

The field-vector parses via the canonical parser from Stone 241.1. `ParseOptions { allow_rest_binder: false }` (struct fields are NOT variadic). This validates the canonical parser's design across form-collapse stones — same parser; different binding sites.

### D3 — Per-field metadata via `:field-metadata` (δ verdict from FORM-COLLAPSE-NOTES)

Form-level `:field-metadata` maps `field-symbol → metadata-map`. The argspec stays RIGID 3-slot triples; per-field metadata lives OUTSIDE the argspec at form level. Per `feedback_simple_is_uniform_composition`.

### D4 — Empty `{}` REJECTED per FORM-COLLAPSE-NOTES

Empty metadata-map is illegal per Stone 241.6 doctrine. defstruct inherits this.

### D5 — Unknown metadata keys

For now: silently ACCEPTED into StructDef metadata (stored generically). Specific consumers (`:restricted-to`, `:field-metadata`, `:doc`, etc.) project their needs. Unknown keys are user-extensibility; no validation at substrate.

### D6 — Vigilia gate doctrine

`src/types.rs` is legacy flat substrate (NOT a `src/<noun>/` namespaced home). Per `feedback_namespaced_home_vigilia_gate` D9 default: no vigilia cast. Commit on SCORE-green.

If sonnet introduces a NEW namespaced home (e.g., `src/defstruct/` or extends `src/argspec/`), vigilia applies on that home — but the recommended scope is "extend types.rs" not "mint new home" given the HARD CUT cleanup nature.

### D7 — Lib + Stone 241.x probes preserved (after migration)

After Stone 241.8:
- `cargo test --release --lib -p wat` ≥ 834 PASS / 0 FAIL (post-migration; tests using legacy syntax updated to new)
- Stone 241.1/2/3/5/6/7 probes preserved at PASS counts
- Stone 241.8 probe ≥ N/N PASS

### D8 — Probe at `tests/probe_arc241_stone8_defstruct.rs`

New FM 2-bis probe. ~6-8 contracts covering:
1. `(defstruct :T [f <- :i64])` — plain struct
2. `(defstruct :T {:restricted-to [...]} [f <- :i64])` — form-level metadata
3. `(defstruct :T {:field-metadata {f {:restricted-to [...]}}} [f <- :i64])` — per-field metadata
4. `(defstruct :T {:restricted-to [...] :field-metadata {...}} [...])` — both
5. Multiple fields with mixed metadata
6. Empty `{}` REJECTED
7. Legacy `(:wat::core::struct ...)` REJECTED (HARD CUT)
8. Legacy `(:wat::core::struct-restricted ...)` REJECTED (HARD CUT)

## Trap-door audit (compact)

### T1 — Cascade size unknown

35 files reference legacy syntax. Some are PROBES that test deletion of LEGACY behavior — those may need their assertions updated rather than just rewritten. Sonnet investigates per file.

### T2 — Test files testing legacy behavior

Files like `tests/wat_arc203_struct_restricted.rs` literally test arc 203's `struct-restricted`. The TEST itself migrates to defstruct semantics; the test's intent (verifying restriction behavior) is preserved.

### T3 — Multi-spawn potential

Stone 241.8 may warrant TWO sonnet flights: (a) substrate mint + macro routing; (b) cascade migration. Orchestrator decides post-first-spawn if needed.

### T4 — `parse_field` legacy parser

The current `parse_struct` calls `parse_field(item)` (types.rs:1927). That helper handles the pair-form `(field :Type)`. After HARD CUT, `parse_field` may also need DELETION or repurposing. Sonnet investigates.

### T5 — `StructDef` schema

`StructDef.restrictions` carries arc 203 metadata. Stone 241.8 likely keeps the schema (existing storage) but populates from defstruct's `:restricted-to` + `:field-metadata` instead of struct-restricted's 4-section shape.

### T6 — Wat `core.wat` macros

The `defstruct` macro may need to be authored in `wat/core.wat` (similar to how `defn` is authored there). OR `defstruct` can be a substrate primitive (parsed directly by types.rs). Per FORM-COLLAPSE-NOTES design — likely substrate primitive given the parsing complexity.

## STOP triggers

1. Compile errors not traced to migration sites
2. Lib < 834 (after cascade migration; "test cascade IS the brief" — failures during cascade are expected; the END STATE must be 834)
3. **120 min elapsed** (upper bound for this larger stone; HARD CUT cascades CAN take this long; per SUBSTRATE-AS-TEACHER iterate)
4. holon-rs touched
5. Files outside `src/types.rs`, `src/runtime.rs` (if needed for register/check), `src/check.rs` (if needed), `tests/probe_arc241_stone8_*`, the 35-ish migration target files, SCORE doc
6. Scope creep: defenum (241.9); define ⇒ defn (241.10); INSCRIPTION (241.11); new type-system features beyond defstruct semantics
7. Stone 241.8 probe < N/N PASS
8. Stone 241.1/2/3/5/6/7 probes regress; arc 237/238 probes regress
9. Clippy > 902

## FM 2-bis evidence

`tests/probe_arc241_stone8_defstruct.rs` (NEW). 6-8 contracts disconfirm at HEAD (defstruct verb doesn't exist; legacy struct + struct-restricted still work). Post-stone: probe N/N PASS + legacy verbs HARD-CUT-rejected.

## Calibration

**Target band: 60-120 min Mode A.**
**Upper bound: 120 min (STOP-3).**

Substantially larger than recent stones. Cascade migration dominates runtime. Sonnet may surface mid-strike scope expansions per `feedback_trap_door_build_the_dependency` (Stone 241.7 surfaced 241.6 storage gap; 241.8 may surface similar in defstruct semantics).

| File | Pre | Post | Delta |
|---|---|---|---|
| `src/types.rs` (mint + delete legacy) | (current) | (~+120 −180 net) | **~-60** |
| `tests/probe_arc241_stone8_defstruct.rs` (NEW) | 0 | ~180 | **+180** |
| ~30 migration target files | various | various | **migration syntax conversion; size depends** |
| **Net delta** | — | — | **substantial mixed** |

## What this unblocks

**Stone 241.9** — defenum HARD CUT. Per FORM-COLLAPSE-NOTES verdict (D) positional variants with look-ahead. The argspec inside tagged variants uses `parse_argspec_triples` (canonical again).

**Stone 241.10** — `define ⇒ defn` HARD CUT.

**Stone 241.11** — INSCRIPTION closes the arc. Arc 237.8b reopens.

---

**Recommendation**: this DESIGN is committed for review. Stone 241.8's full open (probe + BRIEF + EXPECTATIONS + spawn) commits to ~60-120 min of sonnet work + verification cycles. Orchestrator surfaces this scope to user before authorizing the full open.
