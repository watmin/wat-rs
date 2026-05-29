# BRIEF — Stone 241.9 — `:wat::core::defenum` HARD CUT; enum retires

You are sonnet. Phase 3 second stone — HARD CUT. Cascade migration; per `docs/SUBSTRATE-AS-TEACHER.md` the cascade IS the migration brief — iterate from the diagnostic stream until clean. Stone 241.8 (`SCORE-STONE-241.8.md`) is the migration shape you mirror.

## What to do

### S1 — Mint `parse_defenum` at `src/types.rs`

Place near the existing `parse_enum` (around types.rs:2250). Form shape per FORM-COLLAPSE-NOTES verdict D (positional + one-token look-ahead):

```scheme
(:wat::core::defenum :app::Status
  {:variant-metadata {:Error {:doc "..."}}}     ; OPTIONAL form-level metadata
  :Ok                                            ; positional unit variant
  :Pending                                       ; positional unit variant
  :Error [code    <- :wat::core::i64             ; positional tagged variant
          message <- :wat::core::String])        ; (Vector → argspec triples)
```

Algorithm:

1. **Args** (after head consumed by `parse_type_decl`):
   - `args[0]` — name keyword (e.g. `:app::Status`)
   - Optionally `args[1]` — metadata-map `{...}` (WatAST::List with head keyword `:wat::core::HashMap`)
   - Remaining args — positional variants

2. **Discriminate items[1]**: if `WatAST::List` with head `:wat::core::HashMap` → metadata-map; otherwise → first variant. The metadata is OPTIONAL and detected by structural match.

3. **Extract metadata when present**:
   - `:variant-metadata {:variant-kw {meta} ...}` — per-variant metadata map (inner keys MUST be keywords per Stone 241.8 T-fd trap-door)
   - Unknown keys: silently stored (D5)
   - **Empty `{}` REJECTED** per FORM-COLLAPSE-NOTES (Stone 241.6 inherited)

4. **Variant walk — one-token look-ahead**:
   - See `WatAST::Keyword(k, _)` → variant name (strip leading `:`)
   - Peek next item: `WatAST::Keyword` or end-of-args → current is UNIT variant; push `EnumVariant::Unit(name)`
   - Peek next item: `WatAST::Vector(items, span)` → current is TAGGED variant; consume Vector via `crate::argspec::parse_argspec_triples(&items, ":wat::core::defenum", &span, ParseOptions { allow_rest_binder: false })` from Stone 241.1; convert `ArgSpec.fixed_params` to `Vec<(String, TypeExpr)>` → `EnumVariant::Tagged { name, fields }`
   - Bare symbol or other → `MalformedVariant` with hint (mirror legacy `parse_enum_variant`'s symbol-error UX)

5. **Empty variant list REJECTED** (preserve legacy invariant — enum must have ≥ 1 variant).

6. **Per-variant metadata storage** (D5 — silent generic for this stone):
   - For each variant keyword, check `:variant-metadata` inner map; if present, store generically (EnumDef doesn't currently carry per-variant metadata structure — extension is a future consumer-driven stone; do NOT extend EnumDef schema this stone)

7. **Build `TypeDef::Enum(EnumDef { name, type_params, variants })`**.

### S2 — DELETE `parse_enum` (types.rs:2250) and `parse_enum_variant` (types.rs:2575)

HARD CUT. Raw deletion. No shim. No alias.

### S3 — Update macro dispatch at `classify_type_decl` (types.rs:1860-1878)

```rust
// DELETE this arm:
//   ":wat::core::enum" => return Some("enum"),

// ADD this arm:
":wat::core::defenum" => return Some("defenum"),
```

Wire `"defenum"` to `parse_defenum` in the `parse_type_decl` dispatch (types.rs:1897-1908; mirror existing enum → parse_enum routing).

### S4 — Update `check.rs` HARD-CUT-rejection arm

Mirror Stone 241.8's check.rs:6936-6946 pattern for the retirement arm:

```rust
// HARD CUT: legacy enum forms REJECTED at check time.
":wat::core::enum" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!(
            "'{}' is retired (Stone 241.9); use ':wat::core::defenum' instead",
            k
        ),
        span: head_span.clone(),
    }]);
}
```

ALSO remove `:wat::core::enum` from the top-level whitelist at check.rs:6947-6958 (where it's currently grouped with `:wat::core::define`, `:wat::core::defstruct`, etc.); add `:wat::core::defenum` to that whitelist instead.

### S5 — `parse_field` retirement audit (D8)

`parse_field` (legacy struct/enum pair-form helper) is called by `parse_enum_variant`. After S2 deletes `parse_enum_variant`, run `grep -n "parse_field" src/`:
- If 0 remaining callers → DELETE `parse_field` per HARD CUT (`feedback_substrate_naming_audit`)
- If callers remain → KEEP and surface in SCORE honest delta

### S6 — Migration cascade (~25-35 declaration sites)

Per `grep -rl ":wat::core::enum\b"` → 43 hits; many are WAT constructor calls or pattern matches (do NOT migrate) — only DECLARATION sites migrate. Per Stone 241.8 precedent: WAT source files with constructors / matches stay unchanged.

**Conversion patterns:**

```scheme
;; PATTERN A: enum unit-only → defenum unit-only (positional preserved; head rename only)
(:wat::core::enum :Status :Ok :Pending :Error)
;; →
(:wat::core::defenum :Status :Ok :Pending :Error)

;; PATTERN B: enum pair-form tagged variant → defenum argspec-Vector
(:wat::core::enum :Result
  :Ok
  (Err (code :wat::core::i64) (message :wat::core::String)))
;; →
(:wat::core::defenum :Result
  :Ok
  :Err [code    <- :wat::core::i64
        message <- :wat::core::String])
```

Mechanical per-site conversion. Lib + test cascade drives discovery; each failure points at the next site.

### S7 — Probe verification

`tests/probe_arc241_stone9_defenum.rs` (already committed STRIKE-READY). 8 contracts; pre-stone 4/8 (C01-C04 weakly pass; C05-C08 disconfirm); post-stone 8/8.

## Discipline

- **`src/argspec/*` UNCHANGED.** The canonical parser stays as Stone 241.1.fix shipped it; you USE it; you don't modify it.
- **`src/lib.rs` UNCHANGED.**
- **Stone 241.1-241.8 probes UNCHANGED** at current PASS counts.
- **HARD CUT discipline**: NO compatibility shims; NO aliases keeping legacy callable; delete legacy parsers RAW.
- **Test cascade IS the migration brief** per `docs/SUBSTRATE-AS-TEACHER.md` — iterate `cargo test --release --lib -p wat` → read error → migrate that site → re-run. Fail-count is the progress meter.
- **No new ArgSpecError variants** — defenum uses the existing canonical errors via `From<ArgSpecError> for TypeError`.
- **`:variant-metadata` inner keys**: keyword syntax (`:Error` not `Error`) per Stone 241.8 T-fd trap-door. Parser routes `{bareSymbol {submap}}` to struct-destructure; keyword keys parse correctly.
- **NO EnumDef schema extension** — per D5, per-variant metadata stored generically; do not add `variant_metadata: HashMap<...>` field to EnumDef.

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/SUBSTRATE-AS-TEACHER.md` — the cascade discipline
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.9.md` — this
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.9.md` — D1-D8 + T1-T7 + STOP
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.8.md` — migration shape you mirror (per `feedback_stone_briefs_cite_prior_score`)
6. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/FORM-COLLAPSE-NOTES.md` § defenum (lines 118-151) + § Argspec stays RIGID (lines 174-184)
7. `/home/watmin/work/holon/wat-rs/src/types.rs` lines 1860-1908 (classify + dispatch) + 2250-2654 (parse_enum + parse_enum_variant)
8. `/home/watmin/work/holon/wat-rs/src/argspec/parse.rs` — the canonical parser you'll route through
9. `/home/watmin/work/holon/wat-rs/src/check.rs` lines 6925-6970 — Stone 241.8's HARD-CUT-rejection arm shape to mirror
10. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone9_defenum.rs` — 8-contract probe (4/8 at HEAD)
11. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/EXPECTATIONS-STONE-241.9.md`

## Implementation sketch

1. Read substrate + probe + FORM-COLLAPSE-NOTES + SCORE-STONE-241.8.md
2. Baseline: lib 834; Stone 241.9 probe 4/8 (expected disconfirmation)
3. **S1+S2+S3+S4**: substrate mint + delete legacy + classify update + check.rs HARD-CUT arm
4. Run lib tests; the cascade BEGINS
5. **S5**: `parse_field` audit + delete-or-keep verdict
6. **S6**: iterate the cascade — read failure → migrate site → re-run; track progress by fail-count
7. **S7**: verify Stone 241.9 probe 8/8
8. Verify all Stone 241.x probes preserved
9. Final: `cargo test --release --lib -p wat` ≥ 834 · workspace clean · clippy ≤ 902
10. Write `SCORE-STONE-241.9.md`
11. **DO NOT COMMIT.**

## STOP triggers — REJECTION

1. Compile errors not traced to defenum mint or cascade migration sites
2. Lib < 834 (post-cascade-migration final state)
3. **150 min elapsed** (HARD CUT cascade upper bound; +30 vs 241.8 to absorb the `parse_field` audit)
4. holon-rs touched
5. Files outside `src/types.rs`, `src/runtime.rs` (if register/check), `src/check.rs`, `src/special_forms.rs`, `src/freeze.rs`, `src/closure_extract.rs`, the migration target files (~25-35 in src/+tests/+wat-tests/), `tests/probe_arc241_stone9_*`, SCORE doc
6. Scope creep: `define ⇒ defn` (241.10); INSCRIPTION (241.11); new type-system features; new ArgSpecError variants; EnumDef schema extension; `:variant-metadata` consumer-driven semantics
7. Stone 241.9 probe < 8/8
8. Stone 241.x or arc 237/238 probes regress (after migration)
9. Clippy > 902
10. Attempting to keep legacy enum callable in any form (HARD CUT violation)
11. `feedback_no_semantic_abuse_of_option` violation — if reaching for `Option<HashMap>` to flavor-encode variant-metadata storage, STOP

## SCORE doc spec

Mirror `SCORE-STONE-241.8.md`. Include:
- Header (Mode A/B; runtime; one-line summary; cascade size)
- Phase A scorecard (11 rows)
- Structural verification (6 rows)
- Migration cascade audit (per-file changes; total count touched)
- Final `parse_defenum` body (verbatim)
- Honest deltas (cascade discoveries; trap-door builds; `parse_field` audit verdict)
- PHASE 3 advances inscription
- NO Vigilia section

## Post-strike

Return one-paragraph status: defenum minted; legacy deleted; cascade depth (file count); Stone 241.9 probe 8/8; `parse_field` audit verdict; any surfaced gaps.

Phase 3 advances. One more stone (241.10 `define ⇒ defn`) before INSCRIPTION at 241.11. Strike clean.
