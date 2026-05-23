# EXPECTATIONS — Arc 230 Stone 230.1 — Substrate variant retirement (Symbol/Keyword/Tag/Nil → pure Bind compositions)

Mode A target: 16/16 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `HolonAST::Symbol` variant DELETED from holon-rs | `/home/watmin/work/holon/holon-rs/src/kernel/holon_ast.rs:66` — enum variant removed; all cascade arms removed |
| 2 | `HolonAST::Keyword` variant DELETED from holon-rs | Line 100 — variant + cascade arms removed |
| 3 | `HolonAST::Tag` variant DELETED from holon-rs | Line 119 — variant + cascade arms removed |
| 4 | `HolonAST::Nil` variant DELETED from holon-rs | Line 108 — variant + cascade arms removed |
| 5 | PRIM_TAG constants for retired variants REMOVED | PRIM_TAG_SYMBOL / KEYWORD / TAG / NIL constants gone (if existed); structural Bind encoding is the discriminator now |
| 6 | Constructor helpers updated to produce Bind compositions | `HolonAST::symbol(s)` / `keyword(s)` / `tag(s)` / `nil()` now produce `Bind(Atom(String("Symbol")), Atom(String(s)))` etc. — same API surface; different internals |
| 7 | holon-rs builds + tests green | `cargo build --release` 0 errors; `cargo test --release` PASS; `cargo clippy --release -- -D warnings` 0 warnings |
| 8 | wat-rs runtime.rs match arms updated | All `HolonAST::Symbol|Keyword|Tag|Nil` arms either deleted (variant gone) or replaced with classifier-extraction pattern via `extract_classifier` |
| 9 | wat-rs check.rs registrations updated | TypeScheme + special-case handlers for retired-variant-aware verbs adjusted; no references to the deleted variants |
| 10 | wat-rs freeze.rs + lower.rs updated | All cascade sites in these files adjusted |
| 11 | `to_holon_inner` keyword + Unit arms updated | `Value::wat__core__keyword` arm produces Bind-composition via updated `keyword` constructor; `Value::Unit` arm produces Bind-composition via updated `nil` constructor; same end-state for callers |
| 12 | `eval_holon_from_holon` + `from_holon_item` recognize new compositions | Classifier-dispatch extended for "Symbol" / "Keyword" / "Tag" + special-case for nil-symbol; unified with collection classifier-dispatch from arc 228 |
| 13 | wat-rs full test suite green | `cargo build --release -p wat` 0 errors; `cargo test --release --lib -p wat [--skip 5 signal]` PASS; arc 216 probes PASS; arc 221 + arc 143 + mvp_end_to_end PASS |
| 14 | wat-edn unchanged | `cargo test --release -p wat-edn` PASS; `cargo clippy --release --all-targets -p wat-edn -- -D warnings` 0 warnings; `git -C /home/watmin/work/holon/wat-rs/crates/wat-edn/ diff --name-only` should be empty (no changes to wire-format code) |
| 15 | VSA vector identity preserved | Round-trip tests for Symbol/Keyword/Tag/Nil values still pass; structural Bind encoding produces distinct vectors per classifier (no collision) |
| 16 | Doc refresh as discovered | Adjacent doc comments referencing retired variants updated (no global hunt — fix what you touch); note arc 230 supersession |

## Independent prediction (calibration record)

**Target runtime:** 240-420 min Mode A
**Upper bound:** 480 min (8 hours)
**Confidence:** medium (this is the largest substrate refactor in arc 170+'s history; calibration band wider than recent stones)

**Rationale:**
- Touches TWO repos (holon-rs + wat-rs) — first time arc 170+
- ~300-500 estimated touch points (vs 150-200 for Stone 225.1 v3, ~100 for Stone 228.1)
- Substrate-as-teacher cascade locked pattern; calibration trend favors faster-than-target
- Risk: holon-rs Phase A's enum removal will cascade through ALL holon-rs consumers (test fixtures, helper traversals, Display impl, canonical_bytes mechanism)

**Risks:**
- VSA vector identity (Row 15) — new Bind-encoded Symbol("foo") must produce distinct vector from Bind-encoded Keyword("foo"). The classifier-atom is "Symbol" vs "Keyword" — that string difference DOES propagate to canonical bytes if Bind's bytes-encoding traverses children (it does, per holon-rs design). Verify post-retirement.
- Round-trip semantics — Symbol/Keyword/Tag values that previously round-tripped via variant match now round-trip via Bind-composition match; from-holon's classifier-dispatch must handle the new shape
- arc 221 supersession in code comments — sonnet should touch as discovered, not global-hunt

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- wat-edn wire format changes
- Type predicates (arc 226)
- User-defined types (arc 227)
- EDN-form parser-level minting (arc 222)
- WatAST honesty (arc 223)
- Quasiquote evaluator (arc 229)
- INSCRIPTION (Stone 230.4)
- Aliases (HARD CUT)
- from-holon -> :T type-hint propagation (Task #469)

## Honesty deltas accepted

- Constructor fn signatures may evolve slightly if sonnet finds a more honest shape (e.g., `HolonAST::symbol(s)` could become `HolonAST::symbol_classified(s)` if name clarity demands; sonnet picks)
- Number of touch points may exceed pre-flight estimate; cascade absorbs
- Some arc 221 in-code comment headers may be deferred to a future doc-sweep stone if the touch surface is large (sonnet documents in SCORE)

## Honesty deltas NOT accepted

- "Pre-existing failure" framing for arc 221 probe failures broken by this stone — STOP per Stone 221.3 Delta 1a; broken-by-this-stone IS the cascade we expect
- Skipping any variant retirement per "didn't want to touch X" — STOP. Hard-cut means hard-cut. The arc 159 / arc 162 precedent: sweep everything.
- Adding aliases (e.g., `pub use HolonAST::Symbol = SymbolComposition`) — STOP. The "fractal of correctness" principle: dishonesty is illegal. HARD CUT.
- Extending scope to other variant retirements (Bool / I64 / etc.) — STOP per STOP-5. Arc 230 is scoped to Symbol/Keyword/Tag/Nil only. Other carriers stay.
- Touching wat-edn — STOP per STOP-4

## STOP triggers (cross-ref from BRIEF)

- **STOP-1:** unexpected substrate compile errors (not from rename cascade)
- **STOP-2:** test failure beyond cascade-rename consequences after green build
- **STOP-3:** 480 min elapsed
- **STOP-4:** wat-edn touched accidentally
- **STOP-5:** scope-extension surfaced
- **STOP-6:** round-trip semantics break
- **STOP-7:** bash discipline — cargo hang from accidental pipes
- **STOP-8:** VSA vector identity collision under new encoding
