# DESIGN — Stone 234.4.match — match-arm hash-destructure parity

**Status:** ACTIVE (2026-05-25). Arc 234 RESUMES per spawn-block winding post arc 236 closure. Named follow-up from Stone 234.4 D8.

**Scope:** Extend the let-binding hash-destructure capability shipped in Stone 234.4 (`dab1a5cb`) to match-arm pattern position. `(match record [{var :field ...} body])` recognizes `{var :field ...}` as a pattern that binds field-values per `var` and executes the arm body when the scrutinee shape matches (record / struct / HashMap).

Closes task #402 fully. Brings arc 234 to one stone from INSCRIPTION (234.6 fate decision pending + 234.7 INSCRIPTION).

---

## Origin

Stone 234.4 shipped let-binding hash-destructure (`{var :field ...}` in let-position) across three receivers: `Value::wat__Record`, `Value::Struct`, `Value::wat__std__HashMap`. The match-arm position was deferred per Stone 234.4's D8 (out-of-scope; named follow-up). The parser infrastructure (`BraceKind::HashDestructure` + `parse_hash_destructure_body` producing `WatAST::StructPattern` with mixed Symbol/Keyword children) ALREADY recognizes the shape — extension into match-arm position should reuse the existing parser surface.

This stone closes task #402 ("Hash-destructure in match arm patterns ({...} map-shape matching)").

---

## Locked decisions

### D1 — Shape: `{var :field ...}` in match-arm position

```wat
(:wat::core::match scrutinee
  [{var1 :field1 var2 :field2} body])
```

Same source shape as Stone 234.4 let-binding. The brace + alternating Symbol/Keyword content is the discriminator. Parser already recognizes this via `BraceKind::HashDestructure` → `WatAST::StructPattern` with mixed children.

### D2 — Receivers (mirror Stone 234.4)

Three receivers, identical to Stone 234.4:
- `Value::wat__Record` — record instance; field-value bound to var
- `Value::Struct` — Rust struct via TypeDef; field-value bound to var
- `Value::wat__std__HashMap` — HashMap; `Option<V>` per binding (key absent = None, present = Some(value))

No new receivers. Coverage parity with let-binding.

### D3 — Match semantics: SHAPE match, not field-presence match

The arm MATCHES when scrutinee is one of the three receiver types. Field-presence is NOT a shape-check — it's a data-question. For HashMap, missing keys bind `Option::None` (mirroring let-binding). For record/struct, fields are guaranteed by type system (no binding-fails-due-to-absence case).

The arm FAILS to match (falls to next arm) when scrutinee is neither record nor struct nor HashMap. That's the only fall-through condition for hash-destructure patterns.

Rationale: same syntactic shape `{var :field ...}` should have same semantic across let-position and match-position. If a user wants strict-field-presence semantics, they can add a guard after the match-binding (future feature; out of arc 234's scope).

### D4 — Check-time: polymorphic T per binding (mirror Stone 234.4 D4)

Per-class TypeDef registration not shipped (arc 232.1 future-lift). Each hash-destructure-binding-in-match-arm receives `fresh.fresh()` — polymorphic type variable that unifies from body usage. Same pattern as Stone 234.4 D4.

For HashMap-receiver bindings: the unifies-from-body pattern naturally types as `Option<V>` because the body must `option::expect` or `match` to use the value.

### D5 — Three-file constraint (mirror Stone 234.4 STOP-5)

Touch ONLY:
- `src/parser.rs` — IF needed (parser may already recognize the shape via Stone 234.4's BraceKind machinery; sonnet verifies first)
- `src/check.rs` — `infer_match` extension to recognize hash-destructure StructPattern; type-check per D4
- `src/runtime.rs` — `try_match_pattern` + `try_match_pattern_ast` extensions to dispatch hash-destructure StructPattern in match-arm position; eval scrutinee + receiver-type dispatch + conditional bind

PLUS new probe file:
- `tests/probe_arc234_stone4_match_hash_destructure.rs` — 6 contracts (parity with Stone 234.4's probe; match-arm position)

### D6 — Existing match infrastructure reuse

- `infer_match` at `src/check.rs:6889` already handles StructPattern in match-arm position (for arc-169 struct-destructure). Extend it to detect hash-destructure shape (alternating Symbol/Keyword via `items[1].is_keyword()`) and type-check per D4.
- `try_match_pattern` at `src/runtime.rs:14399` already handles StructPattern matching against `Value::Struct` (lines ~14577-14597 show recursive sub-pattern matching). Extend to detect hash-destructure shape and dispatch per-receiver per D2.
- `try_match_pattern_ast` at `src/runtime.rs:24939` is the WatAST-level mirror — needs parallel extension if it processes match patterns at AST level. Sonnet verifies whether parallel update is required.

Reuse existing helpers from Stone 234.4:
- `keyword_accessor_record` — record field-access (Stone 234.4 used this in `bind_let_binding`)
- `keyword_accessor_struct` — struct field-access (same)
- HashMap keyword-key construction — same pattern as Stone 234.4's runtime path

### D7 — No new wat-side surface

Stone 234.4 added no new `:wat::*` verbs (the syntactic form is the surface). Stone 234.4.match similarly adds no new verbs — the match-arm pattern shape IS the surface.

### D8 — HARD CUT: no transitional `match-with-hash-destructure` form

The shape is recognized directly in `(match ...)` arms via existing infrastructure. No separate macro / no transitional aliases.

### D9 — clippy ≤ 54 (current baseline)

No new warnings expected. The pattern-matching extension follows the same shape as Stone 234.4's let-binding extension.

### D10 — Lib baseline preservation

827/0 must hold. The extension is additive (new pattern recognition + dispatch); existing match patterns must remain functional. Arc 234 regression probes (Stones 234.2b/c, 234.3a/b/c, 234.4) all GREEN; arc 236 regression probes GREEN.

---

## Trap-door audit

### T1 — Parser may need NO changes

Stone 234.4's parser work produced `BraceKind::HashDestructure` recognition + `parse_hash_destructure_body` that produces `WatAST::StructPattern` with mixed children. In match-arm position, the parser should already recognize the same brace shape. Sonnet verifies via probe before touching parser; if zero-touch in parser is achievable, prefer it.

### T2 — `infer_match` discrimination

`infer_match` already processes StructPattern arms (arc-169 struct-destructure uses this). The hash-destructure discriminant is `items[1].is_keyword()` (same as Stone 234.4's check.rs work). Sonnet extends the StructPattern arm in `infer_match` to branch on this discriminant.

### T3 — `try_match_pattern` + `try_match_pattern_ast` parity

`try_match_pattern` is the runtime match-arm dispatcher. `try_match_pattern_ast` mirrors it at the WatAST level (e.g., for macro-time matching). Sonnet checks whether macro-time / quasiquote pattern matching is affected; if yes, parallel update; if not (the AST-level mirror is only for specific runtime paths that don't intersect hash-destructure), the AST-level mirror may not need touching. Verify before touching.

### T4 — Scrutinee-shape dispatch

When `try_match_pattern` encounters a hash-destructure StructPattern, it must:
1. Discriminate the pattern via `items[1].is_keyword()`
2. Examine scrutinee value:
   - `Value::wat__Record` → for each (var, field) pair, call `keyword_accessor_record(&record, field)`; bind result to var
   - `Value::Struct` → same with `keyword_accessor_struct`
   - `Value::wat__std__HashMap` → for each (var, field) pair, construct keyword key + look up; wrap result in `Value::Option`; bind to var
   - Other → return `None` (arm fails to match; fall to next arm)
3. Return `Some(env_extended_with_bindings)` if scrutinee was one of the three receivers

### T5 — Empty hash-destructure pattern

`{}` would be an empty hash-destructure (zero bindings). The parser may reject via Stone 234.4's even-count check (`!items.len().is_multiple_of(2)`). Empty case is len-0, which IS multiple of 2; could be allowed. Sonnet decides whether to allow `{}` as "any record/struct/HashMap with no bindings" OR reject. Recommendation: ALLOW (no harm; arm body runs if scrutinee is record/struct/HashMap; useful as a type-guard).

### T6 — Nested patterns inside hash-destructure

`{var1 :field1 ...}` binds `var1` to a value. If `field1`'s value itself is a record/HashMap, can the user nest `{nested :sub-field}` as the BINDING for `var1`? Per Stone 234.4: let-binding shape is `[var :field]` per pair — pure Symbol + Keyword, no nested patterns. Stone 234.4.match preserves that: alternating Symbol + Keyword; no nested patterns. Nested-pattern hash-destructure is OUT OF SCOPE (named follow-up if requested; not currently planned).

### T7 — Lib regression: existing match arms with StructPattern

Arc-169 struct-destructure uses all-Symbol StructPattern in match-arms (and let-positions). The hash-destructure discriminant `items[1].is_keyword()` returns FALSE for arc-169 form. Existing arc-169 patterns must remain functional; sonnet verifies via lib baseline.

### T8 — Sandbox / capability checks

`check_let_for_scope_deadlock_inferred` filters StructPattern children by `Symbol` (per Stone 234.4 SCORE line 73). Match-arm hash-destructure should similarly be invisible to sandbox-leak checks because Keywords are not Symbols. Sonnet verifies the existing walker code paths handle alternating Symbol/Keyword without issue.

### T9 — `try_match_pattern_ast` AST-level mirror

Lines ~24939-25000 show the AST-level pattern matcher. Sonnet checks whether macro expansion / quasiquote evaluation paths use this matcher in a way that touches hash-destructure. If yes, parallel update. If the mirror is only for specific paths that don't intersect, leave it untouched.

### T10 — Probe coverage (6 contracts mirroring Stone 234.4's let-binding probe)

Six contracts to author for `tests/probe_arc234_stone4_match_hash_destructure.rs`:
1. Match record with single `{var :field}` — binds field-value to var; body uses var
2. Match record with multi `{var1 :f1 var2 :f2}` — binds multiple
3. Match HashMap with `{var :field}` — binds Option<V> per key
4. Match HashMap with multi keys — multiple Option<V> bindings
5. Match-arm fall-through: scrutinee is `Value::i64` → hash-destructure arm fails; falls to next arm (which binds via underscore or different shape)
6. Mixed match: some arms are hash-destructure; some are other patterns — selection works correctly

---

## STOP triggers

- STOP-1 unexpected compile errors not tracing to the three-file extension
- STOP-2 lib baseline regresses below 827 (parity stone; no behavior change expected)
- STOP-3 **120 min elapsed** (Mode A target 60-90 min; STOP-3 is 2× upper-bound)
- STOP-4 holon-rs touched
- STOP-5 Rust changes outside src/parser.rs + src/check.rs + src/runtime.rs + the new probe file
- STOP-6 arc 234 OR arc 236 regression
- STOP-7 clippy > 54
- STOP-8 transitional macro / aliasing for hash-destructure-in-match (D8 forbids)
- STOP-9 nested-pattern hash-destructure introduced (T6 forbids)

Each STOP REJECTION.

---

## Calibration

**Target:** 60-90 min Mode A. **Upper:** 120 min (STOP-3).

Surface:
- Parser: 0-30 lines (may need NO touch if existing BraceKind mechanism already covers match position)
- check.rs `infer_match` StructPattern arm: ~30-50 lines (mirror process_let_binding's hash-destructure detection + type-check)
- runtime.rs `try_match_pattern` + possibly `try_match_pattern_ast`: ~60-120 lines (receiver dispatch + conditional bind + 3 receiver paths)
- New probe file: ~150-200 lines (6 contracts; mirror Stone 234.4 probe structure)

Cascade depth: 1-2 compile rounds expected. The extension is localized to match-arm-pattern infrastructure; should not ripple.

Confidence: HIGH. Stone 234.4's infrastructure provides the parser surface + 3 helpers (`keyword_accessor_record`, `keyword_accessor_struct`, HashMap keyword construction). Match-arm dispatch follows same pattern as let-binding bind. Sonnet mirrors Stone 234.4's SCORE shape.

Risks:
- T3 (`try_match_pattern_ast` AST-level mirror) — may or may not need parallel update; verification first
- T5 (empty `{}` decision) — small semantic call; recommend ALLOW
- T9 (existing arc-169 struct-destructure preservation) — verify lib baseline

---

## What this unblocks

- **Closes task #402** fully (hash-destructure in match arm patterns)
- **Arc 234 INSCRIPTION (Stone 234.7)** — only remaining substrate work after this stone is the migration sweep (Stone 234.6) decision (separate arc 238?)
- **Arc 234 closure** within reach: 234.4.match + 234.6 decision + 234.7 INSCRIPTION

---

## Cross-references

- `src/parser.rs` — Stone 234.4's `BraceKind::HashDestructure` + `parse_hash_destructure_body` (the parser surface to reuse)
- `src/check.rs:6889` — `infer_match` (target of extension)
- `src/runtime.rs:14399` — `try_match_pattern` (primary match-arm dispatcher)
- `src/runtime.rs:24939` — `try_match_pattern_ast` (WatAST-level mirror; verify parallel update need)
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.4.md` — let-binding hash-destructure predecessor (mirror its discipline)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.4.md` — let-binding shipment record (3-file change + 6-contract probe template)
- `docs/arc/2026/05/234-wat-record-hologram/PAUSE-CONTEXT.md` — arc 234 pause + resume protocol
- Task #402 — "Hash-destructure in match arm patterns ({...} map-shape matching)" — closed by this stone
- `feedback_stone_briefs_cite_prior_score` — BRIEF cites Stone 234.4 SCORE for sonnet to mirror
- `feedback_no_known_defect_left_unfixed` — closes the named follow-up from Stone 234.4 D8

After this stone ships:
- Decide arc 234.6 fate (separate arc 238 OR keep inside 234)
- Author Stone 234.7 INSCRIPTION + close arc 234
