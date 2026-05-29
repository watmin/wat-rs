# DESIGN — Stone 241.7 — Phase 2 closes: mint `:wat::runtime::metadata-of` reflection verb

**Status:** READY (sub-DESIGN). Phase 2 second stone. Single substrate verb mint reading `SymbolTable.binding_metadata` that Stone 241.6 stored. Vigilia gate doctrine NOT cast (legacy flat substrate per D7 default).

## Why this stone

Stone 241.6 shipped the STORAGE: `def` accepts optional `{...}` metadata-map; persists in `SymbolTable.binding_metadata: HashMap<String, HashMap<String, WatAST>>`; defn inherits via substrate fn-peel. This stone ships the **reflection verb** that READS the storage.

Per `FORM-COLLAPSE-NOTES.md` (LOCKED via intueri cast 2026-05-28):

```scheme
(:wat::runtime::metadata-of :my::ns::my-fn)
;; → #wat.core/Some {:doc "..." :restricted-to [:my::ns::]}     ; Some wrapping HashMap
;; → #wat.core/None nil                                           ; None — binding carries no metadata
```

Sits in the `<aspect>-of-<thing>` family next to `body-of` (sibling pattern at runtime.rs:13660).

## What this stone delivers

ONE new verb at `:wat::runtime::metadata-of`. Mirror body-of's structure; read binding_metadata; wrap in Option.

### S1 — Mint `eval_metadata_of` at `src/runtime.rs`

Place next to `eval_body_of` (around runtime.rs:13660-13715). Pattern:

```rust
/// `(:wat::runtime::metadata-of <name :keyword>) -> :Option<HashMap<Keyword, HolonAST>>`
///
/// Stone 241.7. Returns the binding's metadata-map as Option:
/// - Some({:k1 v1 :k2 v2 ...}) when metadata was attached at def time
/// - None when binding exists but no metadata
/// - None when binding doesn't exist
fn eval_metadata_of(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::runtime::metadata-of";
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
            span: Span::unknown(),
        });
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    let name = match name_from_keyword_or_fn(&v) {
        Some(n) => n,
        None => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::keyword or named function",
                got: ValueSnapshot::of(&v),
                span: args[0].span().clone(),
            });
        }
    };
    
    // Look up binding_metadata for this name.
    match sym.binding_metadata.get(&name) {
        Some(meta) if !meta.is_empty() => {
            // Convert HashMap<String, WatAST> → HashMap<Keyword, HolonAST> Value.
            // Sonnet: find the Value::HashMap constructor pattern;
            // wrap each k: String → Value::Keyword, v: WatAST → HolonAST via watast_to_holon.
            // Then wrap the HashMap value in Option::Some.
            Ok(Value::Option(Arc::new(Some(/* HashMap value */))))
        }
        _ => Ok(Value::Option(Arc::new(None))),
    }
}
```

### S2 — Wire dispatch at runtime.rs:5565 area

Where `:wat::runtime::body-of` is dispatched to `eval_body_of`, add a sibling line:

```rust
":wat::runtime::metadata-of" => eval_metadata_of(args, list_span, env, sym),
```

### S3 — `name_from_keyword_or_fn` reuse

The body-of sibling already provides the helper to extract a name String from either a Keyword value or a named Function. Stone 241.7 reuses it; no new helper.

### S4 — HashMap value construction

Sonnet investigates: `Value::HashMap` constructor or `wat::core::HashMap` wrapping pattern. Likely uses an existing function that builds `Value::HashMap` from key/value pairs. The keys are `Value::Keyword`; the values are `Value::holon__HolonAST(Arc<HolonAST>)`.

For each `(String, WatAST)` pair in `binding_metadata.get(&name)`:
- Key: `Value::Keyword(key_string)` — the String already includes the `:` prefix per storage convention
- Value: `Value::holon__HolonAST(Arc::new(watast_to_holon(&watast)))`

Insert into the HashMap value. Return wrapped in Option::Some.

**STOP-6** if HashMap construction requires more than ~15 lines (e.g., new HashMap ctor; element re-conformance checks).

## Locked decisions

### D1 — Verb name `:wat::runtime::metadata-of` (intueri-locked per FORM-COLLAPSE-NOTES)

Locked by intueri cast 2026-05-28. Family: `<aspect>-of-<thing>` (sibling of body-of). Rejected: `:meta-of` (Clojure-terseness loses noun), `:get-metadata` (verb-first wrong family).

### D2 — Return `Option<HashMap<Keyword, HolonAST>>` per `feedback_no_semantic_abuse_of_option`

Empty `{}` is ILLEGAL at declaration (FORM-COLLAPSE-NOTES); therefore absence-of-metadata distinct from empty-metadata; the distinction IS literally presence/absence; legitimate Option<T> semantics. NOT bare `:nil`.

### D3 — Wire encoding `#wat.core/Some {...}` / `#wat.core/None nil` per arc 216.7 + 218.2

The Option-wrapping uses the FQDN tagged-literal doctrine. This is handled at the print/EDN-encode layer (existing infrastructure for arc 216.7+218.2); Stone 241.7's substrate returns `Value::Option(Arc<Option<Value>>)` and the encoding layer wraps appropriately.

### D4 — `binding_metadata` is the ONLY source

Stone 241.6 stored metadata in `SymbolTable.binding_metadata`. Stone 241.7 reads ONLY from this field. No other paths (e.g., arc 203's restricted-to legacy storage) are consulted — those are HARD CUT in Stone 241.10 territory.

### D5 — Empty inner-map returns None

If `binding_metadata.get(&name)` returns `Some(meta)` where `meta.is_empty()`, return `Option::None`. This shouldn't happen per Stone 241.6's storage logic (empty `{}` is rejected at parse), but defensive: empty inner = no metadata.

### D6 — Vigilia gate doctrine does NOT apply

Single substrate verb mint; no new namespaced home. Commit on SCORE-green.

### D7 — Lib + Stone 241.x probes preserved

Standard preservation set.

### D8 — `body-of` UNCHANGED

Stone 241.7 mints a NEW verb. body-of stays.

---

## Trap-door audit

### T1 — `Value::HashMap` constructor depth

If constructing the return HashMap requires more than ~15 lines, surface as STOP-6 honest delta. Sonnet investigates.

### T2 — String → Keyword conversion

`binding_metadata` storage uses String keys (full `:` prefix per storage convention from Stone 241.6). Stone 241.7 converts to `Value::Keyword` directly (the String IS the keyword's text representation).

### T3 — WatAST → HolonAST conversion

Use existing `watast_to_holon` (the same helper body-of uses at runtime.rs:13696). No new converter.

### T4 — `name_from_keyword_or_fn` helper

Body-of uses this helper. Stone 241.7 reuses. No new helper.

### T5 — Empty inner-map (shouldn't happen but defensive)

Per D5, return None. Stone 241.6 should reject empty `{}` at parse; this is a safety net.

### T6 — Dispatch entry placement

Add the new dispatch arm at runtime.rs:5565 area where body-of is. Sonnet finds the exact match arm.

### T7 — Test cascade

Per Stone 241.x calibration: tests assert structurally; cascade likely zero. The new verb is additive; existing tests don't depend on it.

### T8 — Print/EDN encoding integration

The `#wat.core/Some {...}` / `#wat.core/None nil` encoding is the print/encode layer's concern. If the layer already handles `Value::Option` correctly (via arc 216.7+218.2), Stone 241.7's substrate just returns the Option; the encoding takes care of it. Verify the encoding round-trips at probe time.

### T9 — Empty `{}` regression

Stone 241.6 rejected empty `{}` at parse. Stone 241.7's verb never sees an empty inner map. The defensive D5 covers it; sonnet's probe verifies.

### T10 — Macro-binding metadata visibility

Stone 241.6's fn-peel stored metadata at the BINDING level when defn was used. Stone 241.7 reads from binding_metadata for the BINDING name. Verify defn-with-metadata round-trips: `(defn :f {meta} [args] body)` → metadata-of returns Some({meta}).

---

## STOP triggers (REJECTION)

1. Compile errors not traced to verb mint
2. Lib < 834
3. 30 min elapsed (smaller scope than 241.6)
4. holon-rs touched
5. Files outside `src/runtime.rs` (verb mint + dispatch), `tests/probe_arc241_stone7_*`, SCORE doc. `src/argspec/*` + `src/lib.rs` MUST stay unchanged; Stone 241.x probes MUST stay at current PASS counts.
6. Scope creep: new namespaced home; HARD CUTs of legacy surface (241.8-241.10); new ClauseFailureReason / ArgSpecError variants; modifying body-of or other reflection verbs; HashMap construction > ~15 lines
7. Stone 241.7 probe < N/N PASS
8. Stone 241.x probes regress; arc 237/238 probes regress
9. Clippy > 902

---

## FM 2-bis evidence

`tests/probe_arc241_stone7_metadata_of_reflection.rs` (NEW). ~5-6 contracts covering:

1. `(metadata-of :name-with-meta)` returns Some({:k :v ...})
2. `(metadata-of :name-without-meta)` returns None
3. `(metadata-of :nonexistent)` returns None
4. defn-with-metadata: `(defn :f {:doc ...} [args] body)` → `metadata-of` returns Some({:doc ...}) (verifies 241.6 fn-peel round-trip)
5. Multi-entry metadata round-trips correctly
6. (Optional) Primitive binding returns None (substrate primitives have no def-time metadata)

Pre-stone: all contracts FAIL — the verb doesn't exist; calls error with "unknown verb" or similar.

Post-stone: N/N PASS.

---

## SCORE doc spec

`docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.7.md`. Mirror Stone 241.6's SCORE shape. Include:

- Header (Mode A/B; runtime; one-line summary)
- Phase A scorecard ~10 rows
- Structural verification ~4 rows
- Migration audit
- Final verb body (verbatim)
- HashMap construction approach (line count + which existing constructor used)
- Honest deltas
- **PHASE 2 CLOSES inscription**: metadata-map mechanism + reflection verb both shipped; Phase 3 (HARD CUTs) opens
- NO Vigilia Convergence section

---

## Calibration

**Target band:** 15–30 min Mode A.
**Upper bound:** 30 min (STOP-3).

**Surface estimate:**

| File | Pre | Post | Delta |
|---|---|---|---|
| `src/runtime.rs` (verb mint + dispatch) | (current) | (+~50 lines) | **+50** |
| `tests/probe_arc241_stone7_metadata_of_reflection.rs` (NEW) | 0 | ~130 | **+130** |
| **Net delta** | — | — | **~+180 lines** |

**Confidence: HIGH.** Single verb mint mirroring body-of's exact pattern. Storage already exists from Stone 241.6. Main risk: HashMap value construction depth (STOP-6 budget at ~15 lines).

---

## What this unblocks

**Phase 2 CLOSES** with this stone: metadata-map storage (241.6) + metadata-of reflection (241.7) both shipped.

**Phase 3 opens** at Stone 241.8: defstruct HARD CUT (struct + struct-restricted retire; use the metadata-map mechanism for ctor restrictions + per-field metadata).

**Phase 4** (Stone 241.11 INSCRIPTION) closes the arc. Arc 237.8b reopens after 241.11.

---

## Cross-references

- `SCORE-STONE-241.6.md` — metadata-map storage shipped; SymbolTable.binding_metadata defined; the fn-peel honest delta
- `FORM-COLLAPSE-NOTES.md` § Reflection — `:wat::runtime::metadata-of` locked verdict
- `feedback_no_semantic_abuse_of_option` — Option<T> semantics for absence/presence distinction
- arc 216.7 + 218.2 — FQDN tagged-literal encoding for `#wat.core/Some` / `#wat.core/None nil`
- `feedback_no_regression_until_arc_done` — Phase 2 continues; arc 237.8b waits
- runtime.rs:13660-13715 `eval_body_of` — the sibling pattern Stone 241.7 mirrors
