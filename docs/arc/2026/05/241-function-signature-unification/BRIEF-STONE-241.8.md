# BRIEF — Stone 241.8 — `:wat::core::defstruct` HARD CUT; struct + struct-restricted retire

You are sonnet. Phase 3 first stone — HARD CUT. This is a LARGE stone with cascade migration; per `docs/SUBSTRATE-AS-TEACHER.md` the cascade IS the migration brief — iterate from the diagnostic stream until clean.

## What to do

### S1 — Mint `parse_defstruct` at `src/types.rs`

Place near the existing `parse_struct` (around types.rs:1917). Form shape per FORM-COLLAPSE-NOTES:

```scheme
(:wat::core::defstruct :name
  {:restricted-to [...]                       ; OPTIONAL form-level
   :field-metadata {field-symbol {...} ...}}  ; OPTIONAL per-field
  [field1 <- :T1 field2 <- :T2 ...])          ; REQUIRED argspec
```

Algorithm:

1. **Arity**: `args.len()` after head:
   - 2: `(defstruct :name [fields])` — no metadata
   - 3: `(defstruct :name {metadata} [fields])` — with metadata
   - else: malformed
2. **Discriminate items[1]**: if `WatAST::List` with head keyword `:wat::core::HashMap` → metadata; else → must be the field-vector (the 2-item case)
3. **Extract metadata** when present:
   - `:restricted-to` → vec of keyword prefixes (form-level ctor restriction)
   - `:field-metadata` → inner map `{field-symbol → metadata-map}` for per-field
   - Unknown keys: silently stored (D5)
4. **Empty `{}` REJECTED** per FORM-COLLAPSE-NOTES (Stone 241.6 inherited)
5. **Field-vector** at items[last]:
   - `WatAST::Vector(items, span)` → call `crate::argspec::parse_argspec_triples(&items, ":wat::core::defstruct", &span, ParseOptions { allow_rest_binder: false })` from Stone 241.1
   - Convert `ArgSpec.fixed_params: Vec<(String, TypeExpr)>` to `StructDef.fields`
6. **Per-field metadata association**: for each `(field_name, _)` in fixed_params, check `:field-metadata` map by field_name; populate `StructDef.restrictions` accordingly (existing arc 203 structure)
7. **Build `TypeDef::Struct(StructDef { name, type_params, fields, restrictions })`**

### S2 — DELETE `parse_struct` (types.rs:1917) and `parse_struct_restricted` (types.rs:1956)

HARD CUT. Raw deletion. No shim. No alias.

### S3 — Update macro dispatch at types.rs:1864-1870

```rust
// DELETE these arms:
//   ":wat::core::struct" => return Some("struct"),
//   ":wat::core::struct-restricted" => return Some("struct-restricted"),

// ADD this arm:
":wat::core::defstruct" => return Some("defstruct"),
```

Wire `"defstruct"` to `parse_defstruct` in the dispatch chain (mirror existing struct → parse_struct routing).

### S4 — Update `parse_type_decl` (or whatever calls parse_struct / parse_struct_restricted) to call parse_defstruct

Sonnet finds the dispatch site by grep.

### S5 — Migration cascade

35 files reference legacy `:wat::core::struct` or `:wat::core::struct-restricted` (per `grep -rl ":wat::core::struct\b\|:wat::core::struct-restricted"`). After S1-S4 ships and lib tests are RED, iterate per the diagnostic stream:

**Conversion patterns:**

```scheme
;; PATTERN A: struct pair-form → defstruct triples
(:wat::core::struct :name (f1 :T1) (f2 :T2))
;; →
(:wat::core::defstruct :name [f1 <- :T1 f2 <- :T2])

;; PATTERN B: struct-restricted 4-section → defstruct with metadata
(:wat::core::struct-restricted :name
  [:my::ns]                              ; ctor whitelist
  ([wlist] f1 <- :T1)                    ; restricted section
  (f2 <- :T2))                           ; public section
;; →
(:wat::core::defstruct :name
  {:restricted-to  [:my::ns]
   :field-metadata {f1 {:restricted-to [:wlist]}}}
  [f1 <- :T1 f2 <- :T2])
```

Mechanical per-site conversion. Lib + test cascade drives discovery; each failure points at the next site.

### S6 — Probe verification

`tests/probe_arc241_stone8_defstruct.rs` (already committed). 8 contracts; pre-stone 3/8; post-stone 8/8.

## Discipline

- **`src/argspec/*` UNCHANGED.** The canonical parser stays as Stone 241.1.fix shipped it; you USE it; you don't modify it.
- **`src/lib.rs` UNCHANGED.**
- **Stone 241.1-241.7 probes UNCHANGED** at current PASS counts.
- **HARD CUT discipline**: NO compatibility shims; NO aliases keeping legacy callable; delete legacy parsers RAW.
- **Test cascade IS the migration brief** per docs/SUBSTRATE-AS-TEACHER.md — iterate `cargo test` → read error → migrate that site → re-run. Fail-count is the progress meter.
- **No new ArgSpecError variants** — defstruct uses the existing canonical errors via `From<ArgSpecError> for TypeError`.

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/SUBSTRATE-AS-TEACHER.md` — the cascade discipline
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.8.md` — this
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.8.md` — D1-D8 + T1-T6 + STOP
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/FORM-COLLAPSE-NOTES.md` § defstruct (lines 94-116) + § Argspec stays RIGID (lines 174-184)
6. `/home/watmin/work/holon/wat-rs/src/types.rs` lines 1864-2160 (parse_struct + parse_struct_restricted + macro dispatch)
7. `/home/watmin/work/holon/wat-rs/src/argspec/parse.rs` — the canonical parser you'll route through
8. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone8_defstruct.rs` — 8-contract probe (3/8 at HEAD)
9. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/EXPECTATIONS-STONE-241.8.md`

## Implementation sketch

1. Read substrate + probe + FORM-COLLAPSE-NOTES
2. Baseline: lib 834; Stone 241.8 probe 3/8 (expected disconfirmation)
3. **S1+S2+S3+S4**: substrate mint + delete legacy + macro routing
4. Run lib tests; the cascade BEGINS
5. **S5**: iterate the cascade — read failure → migrate site → re-run; track progress by fail-count
6. **S6**: verify Stone 241.8 probe 8/8
7. Verify all Stone 241.x probes preserved
8. Final: `cargo test --release --lib -p wat` ≥834 · workspace clean · clippy ≤902
9. Write `SCORE-STONE-241.8.md`
10. **DO NOT COMMIT.**

## STOP triggers — REJECTION

1. Compile errors not traced to defstruct mint or cascade migration sites
2. Lib < 834 (post-cascade-migration final state)
3. **120 min elapsed** (larger upper bound for HARD CUT cascade)
4. holon-rs touched
5. Files outside src/types.rs, src/runtime.rs (if register/check), src/check.rs (if needed), the migration target files (~35 in src/+tests/+wat/), tests/probe_arc241_stone8_*, SCORE doc
6. Scope creep: defenum (241.9); define ⇒ defn (241.10); INSCRIPTION (241.11); new type-system features; new ArgSpecError variants
7. Stone 241.8 probe < 8/8
8. Stone 241.x or arc 237/238 probes regress (after migration)
9. Clippy > 902
10. Attempting to keep legacy struct/struct-restricted callable in any form (HARD CUT violation)

## SCORE doc spec

Mirror SCORE-STONE-241.7.md but expanded for cascade scope. Include:
- Header (Mode A/B; runtime; one-line summary; cascade size)
- 10-row scorecard
- Migration cascade audit (per-file changes; total count touched)
- Final parse_defstruct body (verbatim)
- Honest deltas (cascade discoveries; trap-door builds)
- PHASE 3 OPENS inscription
- NO Vigilia section

## Post-strike

Return one-paragraph status: defstruct minted; legacy deleted; cascade depth (file count); Stone 241.8 probe 8/8; any surfaced gaps.

Phase 3 opens with this stone. Two more stones (241.9 defenum; 241.10 define ⇒ defn) before INSCRIPTION at 241.11. Strike clean.
