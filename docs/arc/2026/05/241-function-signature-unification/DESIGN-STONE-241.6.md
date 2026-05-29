# DESIGN — Stone 241.6 — Phase 2 opens: optional `{...}` metadata-map storage on `def`; defn inherits

**Status:** READY (sub-DESIGN). Phase 2 first stone. Substrate STORAGE only; reflection verb (Stone 241.7) reads what this stone stores. Vigilia gate doctrine TBD per scope (likely no new namespaced home; legacy flat substrate edits).

## Why this stone

Per `FORM-COLLAPSE-NOTES.md` (settled doctrine; intueri-cast verdicts locked 2026-05-28): the substrate needs a UNIFORM metadata-map mechanism that:
- Sits between binding-name and value-expr on `def`
- Inherits to `defn` (which macro-expands to `def + fn`)
- Composes uniformly with `defstruct` (form-level `:restricted-to` + `:field-metadata`) and `defenum` (`:variant-metadata`)
- Subsumes arc 203's `def-restricted` / `defn-restricted` / `struct-restricted` legacy
- Generic dictionary: `HashMap<Keyword, HolonAST>` — substrate doesn't enforce specific keys; downstream consumers (Stone 241.7 reflection; future 241.10 HARD CUT of def-restricted) consume specific keys

Per `feedback_no_regression_until_arc_done` (just inscribed): we continue forward on arc 241. Phase 2 first stone is the storage; Stone 241.7 ships the reflection verb; Stones 241.8-241.10 HARD CUT legacy surface using this storage.

## What this stone delivers

The substrate capability `def` accepts an optional `{...}` clause between the name and value-expr; the metadata persists; reflection verb (241.7) can later READ it. NO new keys enforced; no legacy surface removed in this stone (HARD CUTs are 241.8-241.10).

### S1 — Extend `try_parse_fn_shape_def` discrimination

Current shape (runtime.rs:3868): `(def :name <fn-form>)` — exactly 3 items.

Extended shape per FORM-COLLAPSE-NOTES:
- `(def :name <fn-form>)` — 3 items; no metadata; UNCHANGED behavior
- `(def :name {...metadata...} <fn-form>)` — 4 items; metadata-map between name and value-expr

Detection via one-token look-ahead on items[2]:
- items[2] is a `WatAST::List` whose head is `:wat::core::HashMap` keyword → metadata-map; items[3] is value-expr
- items[2] is anything else → no metadata; items[2] is value-expr (existing path)

Note: `{...}` in wat source parses to `WatAST::List([Keyword(":wat::core::HashMap"), Keyword(":K"), Keyword(":V"), key1, val1, key2, val2, ...], span)` per the parser's brace-form dispatch (parser.rs:222-287). The discriminator is the head being `:wat::core::HashMap`.

### S2 — Store metadata at substrate (SymbolTable extension)

Add `pub binding_metadata: HashMap<String, HashMap<String, WatAST>>` to `SymbolTable` (the OUTER key is the binding name `:my::ns::my-fn`; the INNER map is `:key → value-WatAST`). Or use a more typed structure if the substrate has one — sonnet investigates.

When `def` parsing succeeds with metadata present, INSERT into `binding_metadata`. Stone 241.7 reads from here.

Empty `{}` is ILLEGAL per FORM-COLLAPSE-NOTES (divide-by-zero); reject as parse error. Sonnet checks if the parser already enforces this OR if `try_parse_fn_shape_def` needs to.

### S3 — `defn` macro expansion threads metadata

The `defn` defmacro currently expands `(defn :name [args] -> :ret body)` to `(def :name (fn [args] -> :ret body))`. Stone 241.6 extends:

- `(defn :name {...} [args] -> :ret body)` → `(def :name {...} (fn [args] -> :ret body))`

Sonnet finds the defn macro definition + extends it. The substrate's macro-expansion path handles the rest.

### S4 — `try_parse_fn_shape_def_restricted` UNCHANGED

`def-restricted` (arc 203) keeps its existing parser at runtime.rs:3948. The HARD CUT (replace def-restricted with `def + {:restricted-to ...}`) is a LATER stone — 241.8 or 241.10 territory. Stone 241.6 just adds the new mechanism alongside.

### S5 — Non-fn-shape `def` parsing (def with plain value, not fn-form)

Looking at current code: `try_parse_fn_shape_def` returns `None` if items[2] is NOT a fn-form List. For NON-fn def (e.g., `(def :x 42)` or `(def :x {:key :val} 42)`), the substrate has a separate path — sonnet investigates the non-fn def handling and extends it similarly.

The metadata-map discrimination is UNIVERSAL across def shapes: between name and value-expr, regardless of whether value-expr is a fn-form or plain.

## Locked decisions

### D1 — Metadata-map discrimination = head keyword `:wat::core::HashMap`

Per parser.rs brace-form dispatch: `{...}` parses to `(:wat::core::HashMap :K :V k v ...)` List. The def parser checks items[2]'s head keyword.

### D2 — Empty `{}` is ILLEGAL per FORM-COLLAPSE-NOTES

Reject at parse time. If the parser doesn't already enforce, surface the rejection in def-level discrimination.

### D3 — Metadata persists at SymbolTable level

`binding_metadata` (new field on SymbolTable, or use existing per-binding registry). Keyed by binding name (the full `:my::ns::name`). Inner value is the metadata HashMap as `WatAST` (raw AST; downstream consumers project to their needs).

### D4 — `def-restricted` UNCHANGED in this stone

Arc 203's `def-restricted` continues to work via its existing path. The HARD CUT happens in a later stone. Stone 241.6 just adds the new mechanism alongside.

### D5 — No new keys enforced; storage is generic

`HashMap<Keyword, HolonAST>` — the substrate's metadata-map is a generic dictionary. Specific keys like `:restricted-to`, `:doc`, `:deprecated`, etc. are downstream-consumer concerns. Storage doesn't validate keys.

### D6 — `defn` macro updated for metadata-map inheritance

Per FORM-COLLAPSE-NOTES: `(defn :name {meta} [args] -> :ret body)` → `(def :name {meta} (fn [args] -> :ret body))`. Macro expansion threads metadata into def.

### D7 — Vigilia gate doctrine TBD

If Stone 241.6 confines edits to legacy flat substrate (runtime.rs + check.rs), gate does NOT apply. If sonnet introduces a new namespaced home (e.g., `src/metadata/` for the storage layer), gate APPLIES on that home.

**Default expectation**: legacy flat substrate only; no gate.

### D8 — Lib baseline + Stone 241.x probes preserved

After Stone 241.6:
- `cargo test --release --lib -p wat` ≥ 834 PASS / 0 FAIL
- `cargo test --release --test probe_arc241_stone1_argspec_canonical` = 15/15 PASS preserved
- `cargo test --release --test probe_arc241_stone2_fn_parser_migration` = 10/10 PASS preserved
- `cargo test --release --test probe_arc241_stone3_defclause_parser_migration` = 6/6 PASS preserved
- `cargo test --release --test probe_arc241_stone5_defclause_rest_dispatch` = 8/8 PASS preserved
- `cargo test --release --test probe_arc237_8b_defclause_arithmetic gate_1` = 1 PASS preserved
- `cargo build --release --tests --workspace` clean
- `cargo clippy --release` ≤ 904

### D9 — New probe at `tests/probe_arc241_stone6_def_metadata_map.rs`

FM 2-bis probe with ~6 contracts covering:
1. `(def :x {:doc "..."} 42)` parses + persists metadata
2. `(def :y 42)` parses with NO metadata (unchanged behavior)
3. `(def :z {:k1 :v1 :k2 :v2} expr)` multi-entry metadata persists
4. `(def :w {} 42)` empty metadata REJECTED
5. `(defn :f {:doc "..."} [x <- :i64] -> :i64 x)` metadata inherits via macro
6. `(def-restricted :name [...] (fn ...))` UNCHANGED behavior (regression)

The reflection verb (Stone 241.7) is needed to QUERY the stored metadata; for Stone 241.6, the probe verifies STORAGE — likely via internal substrate inspection (sonnet finds the test-accessible path) OR via Stone 241.7 prep (mint a minimal getter privately for testing; Stone 241.7 ships the public verb).

Sonnet decides the verification strategy; STOP-6 if requires substantial test infrastructure beyond ~20 lines.

### D10 — Arc 203 `:restricted-to` semantics preserved end-to-end

The `def-restricted` path (existing) and the NEW metadata-map path coexist. Stone 241.6 doesn't unify them; that's the HARD CUT's job (Stone 241.10 or similar). Tests asserting on `:restricted-to` behavior continue to pass via the `def-restricted` path.

---

## Trap-door audit

### T1 — Non-fn def parsing path

The substrate likely has separate def handling for fn-shape vs plain-value def. Stone 241.6 extends BOTH paths. Sonnet investigates the plain-value def path (probably in eval_define or similar).

### T2 — Where to insert metadata-map into SymbolTable

`SymbolTable` may have multiple binding registries (functions, defined_values, etc.). Sonnet finds the right structure — could be:
- New field on SymbolTable
- Decoration on existing FunctionDef / DefinedValue struct
- Separate `BindingMetadata` map keyed by name

Pick the simplest extension that survives across def/defn/defstruct/defenum binding contexts.

### T3 — Empty `{}` rejection mechanism

If parser already rejects empty brace literals as `MalformedBraceLiteral`, Stone 241.6 doesn't need to add the check. If not, def-level validation rejects.

### T4 — Defn macro location

The defn macro is likely defined in `wat/runtime.wat` or similar. Sonnet finds it and extends the expansion pattern.

### T5 — Backward compatibility on existing def calls

`(def :x 42)` and `(def :name (fn [args] -> :ret body))` must continue to work without changes. The metadata-map is OPTIONAL; absence preserves current behavior.

### T6 — Type checking of metadata values

The metadata value-WatAST is generic — could be a Keyword, String, integer, Vector, etc. The substrate doesn't type-check metadata content; Stone 241.7's reflection verb returns the raw HashMap; downstream consumers project to their typed needs.

### T7 — `defn` metadata + fn-form interaction

If user writes `(defn :name {:doc "..."} [x <- :i64] -> :i64 x)`, the macro expands. The metadata flows to `def`'s binding_metadata, NOT to the inner `fn`'s closure or signature. The binding metadata is at the BINDING level, not the value level.

### T8 — Storage atomicity

If def-with-metadata fails mid-process (e.g., metadata stored but def evaluation fails), the substrate must not leak partial state. Sonnet ensures atomic registration.

### T9 — Macro expansion order

defn's macro expansion happens before def's parse. Metadata flows through the expansion. Sonnet verifies the order is correct.

### T10 — Test cascade

Existing def tests (if they assert exact form shape) may need updates. Per Stone 241.x calibration: substrate tests assert structurally; cascade likely zero. Surface as honest delta if non-zero.

---

## STOP triggers (REJECTION)

1. Compile errors not traced to migration sites
2. Lib < 834 (after assertion updates)
3. 50 min elapsed
4. holon-rs touched
5. Files outside `src/runtime.rs`, `src/check.rs` (if needed), `wat/runtime.wat` (or similar — defn macro location), `tests/probe_arc241_stone6_*`, SCORE doc, test files with assertion updates. `src/argspec/*` MUST stay unchanged; `src/lib.rs` MUST stay unchanged; Stone 241.x probes MUST stay at their current PASS counts.
6. Scope creep:
   - Minting `:wat::runtime::metadata-of` reflection verb (that's Stone 241.7)
   - HARD CUT of `def-restricted` / `defn-restricted` (that's Stone 241.10)
   - HARD CUT of `struct` / `struct-restricted` (that's Stone 241.8)
   - New ClauseFailureReason / ArgSpecError variants
   - New namespaced home (likely overkill for storage)
7. Stone 241.6 probe doesn't reach N/N PASS
8. Stone 241.x probes regress (1: 15/15; 2: 10/10; 3: 6/6; 5: 8/8); arc 237/238 probes regress
9. Clippy > 904

---

## FM 2-bis evidence

`tests/probe_arc241_stone6_def_metadata_map.rs` (NEW). 6 contracts disconfirm the missing metadata-map storage at HEAD (current def parser rejects 4-item def OR misinterprets the `{...}` form).

**Pre-stone**: contracts that exercise metadata-map storage will FAIL at HEAD (the current substrate doesn't recognize the optional clause).

**Post-stone**: probe passes N/N; Stone 241.7 will then add the reflection surface that lets external code query the storage.

---

## SCORE doc spec

`docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.6.md`. Mirror Stone 241.5's SCORE shape (no vigilia section if no namespaced home):

- Header (Mode A/B; runtime; one-line summary)
- Phase A scorecard ~10 rows
- Structural verification ~5 rows
- Migration audit (per-file deltas)
- Final post-stone code shapes (verbatim discrimination logic + storage)
- Honest deltas
- Cascade depth note
- **PHASE 2 OPENS inscription**: metadata-map storage shipped; Stone 241.7 reflection verb queued
- NO Vigilia Convergence section (per D7 default; unless sonnet mints a namespaced home)

---

## Calibration

**Target band:** 25–45 min Mode A.
**Upper bound:** 50 min (STOP-3).

**Surface estimate:**

| File | Pre | Post | Delta |
|---|---|---|---|
| `src/runtime.rs` (def parser + SymbolTable extension) | (current) | (+~50 lines) | **+50** |
| `wat/runtime.wat` (or similar — defn macro) | (current) | (+~5 lines for macro update) | **+5** |
| `src/check.rs` (if needed — metadata binding-table integration) | (current) | (+~10 lines) | **+10** |
| `tests/probe_arc241_stone6_def_metadata_map.rs` (NEW) | 0 | ~150 | **+150** |
| **Net delta** | — | — | **~+215 lines** |

**Confidence: MODERATE-HIGH.** Cleaner than Stone 241.5 (the dispatch wiring) but with several SymbolTable + macro integration touch points. The defn macro location + SymbolTable structure are the main investigation risks.

**Per `feedback_stone_briefs_cite_prior_score`**: BRIEF cites Stone 241.5 SCORE for the named-follow-up doctrine; Stone 241.4 SCORE § Vigilia Convergence for the canonical home's foundation.

---

## What this unblocks

**Stone 241.7** — mint `:wat::runtime::metadata-of` reflection verb. Per FORM-COLLAPSE-NOTES naming verdicts (LOCKED): the verb name + `Option<HashMap<Keyword, HolonAST>>` return shape (encoded as `#wat.core/Some {...}` / `#wat.core/None nil` per arc 216.7 + 218.2). Stone 241.7 reads from binding_metadata that THIS stone stores.

**Stones 241.8-241.10** — HARD CUTs of legacy `struct`/`struct-restricted`/`enum`/`define`. These use the metadata-map mechanism to express what the legacy forms expressed differently.

**Phase 2 complete** at Stone 241.7. Phase 3 (HARD CUTs) at Stones 241.8-241.10. Phase 4 (INSCRIPTION) at Stone 241.11.

---

## Cross-references

- `SCORE-STONE-241.5.md` — Phase 1 truly closed; named-follow-up doctrine validated
- `SCORE-STONE-241.4.md` § Vigilia Convergence — canonical foundation
- `FORM-COLLAPSE-NOTES.md` — the doctrinal source; `{...}` discriminator + empty-`{}`-illegal + `:field-metadata`/`:variant-metadata` patterns
- `feedback_no_regression_until_arc_done` — why we continue on arc 241 instead of pivoting to arc 237.8b
- `feedback_stone_briefs_cite_prior_score` — BRIEF cites Stone 241.5 SCORE for the doctrine
- `feedback_namespaced_home_vigilia_gate` D7 default — vigilia not cast unless namespaced home minted
- arc 203 — `def-restricted` / `defn-restricted` existing path (preserved in this stone; HARD CUT later)
