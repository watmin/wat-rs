# DESIGN — Arc 234 Stone 234.3b.fix — `RuntimeError::UnknownField` variant

**Status:** ACTIVE (2026-05-24).

**Origin:** Stone 234.3b SCORE noted: "UnknownField error uses MalformedForm with reason string... Future cleanup arc could mint RuntimeError::UnknownField as proper variant." Orchestrator accepted this framing — that's a deferral protocol violation per `feedback_no_known_defect_left_unfixed`. "Future cleanup arc" without a NAMED successor is the failure pattern. Stone 234.3b.fix is the named fix, NOW.

---

## Scope

Mint `RuntimeError::UnknownField` variant + migrate `eval_record_assoc`'s use of `MalformedForm` to the proper variant.

Three files:
1. **`src/runtime.rs`** — add variant to enum (line ~1897); update `eval_record_assoc` to construct the new variant
2. **`src/runtime_error_edn.rs`** — add EDN serializer arm + variant-name match (~line 113 for serializer, ~line 304 for name lookup)
3. **Other exhaustive matches** — substrate-as-teacher will surface compile errors; add arms uniformly

No probe change needed — Stone 234.3b probe 3's lenient assert (`msg.contains("unknown") || msg.contains("nonexistent")`) passes regardless of variant; the new variant's message will still contain those substrings.

---

## Variant shape

```rust
UnknownField {
    record_class: String,   // "myapp::Voltage" (no leading colon)
    field: String,          // "nonexistent" (bare field name)
    available: Vec<String>, // ["magnitude"] — known field names on this class
    span: Span,
}
```

Mirrors the existing pattern for other identifier-not-found errors (`UnboundSymbol`, `UnknownFunction`). The `available: Vec<String>` field gives the user actionable info: which fields ACTUALLY exist on the record they're trying to update.

---

## Locked decisions

### D1 — Variant lives in `src/runtime.rs::RuntimeError`

Same enum as all other runtime errors; consistent.

### D2 — EDN serializer arm

`src/runtime_error_edn.rs` line ~48-113 has the per-variant serializer match. Add an `UnknownField` arm following the existing pattern (`tagged("UnknownField", map3(...))` or similar based on field count).

Also line ~299-304: the variant-name → string lookup. Add `RuntimeError::UnknownField { .. } => "UnknownField"`.

### D3 — eval_record_assoc migration

In `src/runtime.rs::eval_record_assoc`: replace the current `MalformedForm` construction with `RuntimeError::UnknownField { record_class, field, available, span }`. The available-fields vec is already being computed (or can be) during the holon_form walk.

### D4 — Probe 234.3b stays unchanged

Probe 3's assert is lenient enough to accept both error messages. Verify post-migration that probe still PASSES 6/6.

### D5 — Other exhaustive matches surface via substrate-as-teacher

If any other site has `match err { ... }` exhaustively over RuntimeError, compile errors surface. Add arms uniformly. Sonnet handles per substrate-as-teacher cascade.

### D6 — No defaults, no backwards-compat shim

HARD CUT — the MalformedForm fallback is REMOVED from eval_record_assoc; only UnknownField fires for the missing-field case. Per `feedback_wat_llm_first_design` (no synonym features).

---

## Trap-door audit

### T1 — Compile-time exhaustive matches

`runtime_error_edn.rs` has two exhaustive matches (serializer + name lookup). Any other site with full enumeration surfaces a compile error. Add arms.

### T2 — Available-fields vector construction

The `available: Vec<String>` needs to be computed during the holon_form walk. The eval_record_assoc currently walks Bundle/children + extracts each name; collect them into a Vec during the same pass. ~3-5 extra lines.

### T3 — The `record_class` field has no leading colon

Per Stone 234.2a SCORE D5: class_fqdn is stored without leading colon. The error variant carries the bare form (matches the `:wat::core::type` return shape).

### T4 — Probe 3's assert robustness

The probe asserts `msg.contains("unknown") || msg.contains("nonexistent")`. New error message will be something like `"UnknownField: field 'nonexistent' not on record myapp::Triple; available: a, b, c"` (or however the EDN serializer renders it). Both substrings present; assert passes.

### T5 — Display impl

If `RuntimeError` has a `Display` or `fmt` impl with exhaustive variant match, add an `UnknownField` arm there too. Sonnet checks.

### T6 — Future variants follow same path

The discipline this restores: every NEW error semantics gets its OWN variant. Reason-string-stuffing into existing variants (MalformedForm-as-catchall) is the anti-pattern. Substrate stays honest.

---

## STOP triggers

- STOP-1 — unexpected compile errors not tracing to the new variant
- STOP-2 — lib baseline < 827
- STOP-3 — 30 min elapsed (small focused fix)
- STOP-4 — `holon-rs` touched
- STOP-5 — Rust changes outside `src/runtime.rs` + `src/runtime_error_edn.rs` + any OTHER files that exhaustive-match RuntimeError (sonnet surfaces via compile errors; add arms; don't pursue unrelated changes)
- STOP-6 — scope creep: new error variants beyond UnknownField; refactoring unrelated error variants
- STOP-7 — Stone 234.3b probe regresses (must stay 6/6 PASS)
- STOP-8 — any other arc 234 regression guard regresses
- STOP-9 — clippy > 54

Each STOP is REJECTION.

---

## Calibration

**Target:** 15–30 min Mode A. **Upper:** 45 min (STOP-3).

Surface: ~20-40 lines across 2-3+ files (variant decl + 2 EDN-rs arms + use-site migration + any other exhaustive matches the compiler surfaces).

Confidence: HIGH. Mechanical addition; well-precedented (other variants exist with similar shape).

---

## What this closes

- Honors `feedback_no_known_defect_left_unfixed` — the deferral is converted to a named-now-shipping stone
- Restores the discipline that every error variant carries its OWN semantics (no MalformedForm catch-all stuffing)
- Stone 234.3b's SCORE delta is now historical-only; the substrate carries the proper variant

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.3b.md` — predecessor (where the deferral was committed)
- `src/runtime.rs::eval_record_assoc` — use site
- `src/runtime.rs:1897` — RuntimeError enum
- `src/runtime_error_edn.rs:48+` — EDN serializer
- `feedback_no_known_defect_left_unfixed.md` — the discipline this honors
- `feedback_wat_llm_first_design.md` — no MalformedForm-as-catchall (synonym anti-pattern at the error layer)
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs; sonnet writes
