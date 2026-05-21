# INSCRIPTION — Arc 219 — wat-edn strict-EDN keyword namespace compliance

**Opened:** 2026-05-21
**Closed:** 2026-05-21 (same day)
**Branch:** `arc-170-gap-j-v5-deadlock-state`
**Trigger commit:** `0ce5a44` (DESIGN)
**Closing commit:** `331cfb9` (Stone 219.1)

## Mission (achieved)

Make wat-edn output round-trippable through standard `clojure.edn/read`. Tighten the dialect to strict EDN on input AND output.

## What shipped

**Stone 219.1** (`331cfb9` — 11/11 PASS, ~35 min, below 45-75 min lower bound)

Three coordinated surfaces in one bundled stone:

1. **`crates/wat-edn/src/vocab.rs:101-122`** — `is_symbol_continue` drops `b':'` and `b'#'` from the accepted char set. Final set: alphanumeric + `. * + ! - _ ? $ % & = < > /` (matches `github.com/edn-format/edn` spec for symbol bodies). `b'<' | b'>'` preserved (EDN-spec; required for `Vec<i64>` parametric type-arg lists).

2. **`crates/wat-edn/src/value.rs`** — private helper `fn translate_wat_to_strict(ns: &str) -> String` (uses `ns.replace("::", ".")`; idempotent) applied at six constructor sites: `Symbol::ns`, `Symbol::try_ns`, `Keyword::ns`, `Keyword::try_ns`, `Tag::ns`, `Tag::try_ns`. Translation runs BEFORE `validate_first_char` and BEFORE storage. Storage canonical `.`. The three `from_parts_unchecked` paths verified UNCHANGED (unchecked caller responsibility).

3. **`crates/wat-edn/tests/wire_encoding.rs`** — 9 sites swept from `::` form to `.` form (5 constructor-call + 4 parse-call sites that depended on the retired wat-extension). All fixtures now use strict-EDN form. Three new probes in `tests/spec_strict.rs`: `is_symbol_continue_rejects_colon` (lexer-level), `parser_rejects_double_colon_in_keyword` (parse-level rejection), `keyword_ns_translates_wat_to_strict` (constructor-translation visible via `.namespace()` accessor).

**Pre-existing test rot fix** (`c3a27cf` — orchestrator-direct, committed BEFORE 219.1 onto the green tree)

Stone 219.1's STOP-4 workspace sanity (`cargo test --release --lib -p wat`) surfaced 2 lib failures: `runtime::tests::hashmap_composite_key_errors` + `runtime::tests::hashset_rejects_composite_element`. Independent verification via stash round-trip + fixture grep confirmed these were arc 216 test rot (silently red since 216.5b/c shipped 2026-05-20 because those stones intentionally removed the composite-rejection contract via `impl Hash for Value`). Tests flipped from negative to positive contract — `hashmap_accepts_composite_key` + `hashset_accepts_composite_element`. Tree returned green BEFORE 219.1 landed (per `feedback_no_broken_commits`).

## Calibration

| Stone | Predicted | Actual | Result |
|---|---|---|---|
| **219.1** | 45-75 min | ~35 min | 11/11 PASS; below lower bound |

Stones 219.2 (test sweep) + 219.3 (wat-rs boundary validation) folded into 219.1 in execution:
- 219.2 — sonnet's grep found 9 sites (4 more than orchestrator pre-flight); all swept inline
- 219.3 — proven by `cargo test --release --lib -p wat` 824/0 PASS in 219.1 verification (constructor translation hides the wat-rs ↔ wat-edn boundary cleanly; zero callers needed migration)

Stone 219.4 (this INSCRIPTION) is the only paperwork distinct from 219.1.

**Five-stone calibration trend** (all at-or-below lower band):

| Stone | Band | Actual |
|---|---|---|
| 218.1 | 25-45 | ~20 |
| 218.2 | 30-50 | ~15 |
| 218.3 | 40-65 | ~25 |
| 218.4 | 20-40 | ~20 |
| 219.1 | 45-75 | ~35 |

Substrate-pre-grep + locked-decisions + mechanical edits ships consistently below floor.

## Substrate state post-arc

- **`is_symbol_continue` (vocab.rs)** — strict EDN char set; rejects `:` and `#` in symbol bodies; `Vec<i64>` parametric types still parse
- **Constructor translation (value.rs)** — `Keyword::ns("wat::core", "X")` → stored as `Keyword(ns="wat.core", name="X")`. `from_parts_unchecked` paths preserve unchecked semantic.
- **Wire format** — strict EDN. `clojure.edn/read` can consume wat-edn output without extension.
- **Round-trip identity** — wat-rs callers passing `::`-form literals get auto-translated at construction; storage is `.` form; output is `.` form; re-read produces `.` form. Round-trip preserves the strict-EDN canonical form.
- **wat-rs internal storage** (`src/`) — UNCHANGED. wat-rs keeps `::` in its own SymbolTable keys + Rust string literals. The translation is wat-edn-internal; wat-rs doesn't see it.

## What this arc did NOT do

- Touch wat-rs internal SymbolTable / FQDN registrations (out of scope per Option β; wat-rs keeps `::` internally)
- Change `.wat` source syntax (still `::` for Rust-mirror readability)
- Add new wat-edn error variants (existing infrastructure sufficient)
- Address the wat-edn `<...>` type-arg list shape (orthogonal; arc 218 stone 218.3 handled)

## Boundary discipline (Option β confirmed correct)

Three options analyzed in DESIGN: α (write-side only — too permissive), β (substrate strict + constructor auto-translation — locked), γ (substrate-wide wat-rs convention flip — multi-day work, out of arc 219's "do it now" charge).

Option β was the right call. Evidence:
- Constructor translation hid the boundary for wat-rs callers (`cargo test --release --lib -p wat` 824/0 PASS)
- Wat-edn-internal test fixtures swept cleanly (9 sites, mechanical)
- Substrate identity preserved on both sides (wat-rs uses `::`, wat-edn uses `.`, boundary is the constructor)
- Sonnet completed in ~35 min vs predicted 45-75; below lower bound

## Cross-references

- **DESIGN-219** (`0ce5a44`) — Option α/β/γ analysis; four-questions YES×4 for β; stone decomposition
- **BRIEF-219.1** (`6c580f7`) — sonnet scope; STOP triggers; verification
- **SCORE-219.1** — sonnet's report; 11/11 PASS; deltas (sweep wider than orchestrator's 5 sites)
- **arc 216 test rot fix** (`c3a27cf`) — pre-existing failures surfaced by 219.1's STOP-4 verification; fixed onto green tree before 219.1 landed
- **DESIGN-218 § "Forward-correction 2026-05-21b"** — FQDN tags (`#wat.core/Some` etc); arc 219 extends the strictness to keyword bodies
- **INTERSTITIAL § 2026-05-21b** — substrate-audit-supersedes-doctrine pattern (Song #18 — Structural Defect)
- **Arc 218 Stone 218.5** — unblocked by arc 219 closure; re-cast vigilia runs on post-strict substrate
- **Arc 217** — Clojure-IPC bridge; natural forcing function for strict EDN; now builds on a clean foundation
- **`feedback_fqdn_is_the_namespace`** — doctrine; strict EDN is the canonical surface
- **`feedback_pre_existing_verification`** — applied: sonnet's stash claim independently verified via stash round-trip + fixture grep
- **`feedback_no_broken_commits`** — applied: test rot fixed FIRST so 219.1 landed on green tree
- **`feedback_no_pre_existing_excuse`** — visibility gap named (arc 216 stones didn't run full lib tests)

## Blocking chain (post-arc-219)

```
arc 218 Stone 218.5 (re-cast vigilia + INSCRIPTION + arc 218 closure) — UNBLOCKED
  → arc 217 (Clojure-IPC bridge — natural forcing function)
  → arc 216 stones 216.8 (#wat.core/Some et al) / 216.9 (#wat.time/Duration) / 216.10 (closure)
  → arc 214 Slice 4
```

## Status

**CLOSED 2026-05-21.** Stone 219.1 shipped at `331cfb9`. Substrate verified strict-EDN compliant. Workspace green end-to-end. Five-stone calibration trend continues — substrate-pre-grep + locked-decisions + mechanical edits = predictable below-floor execution.

The smallest substrate arc in arc 170+'s history (one substantive stone + this paperwork). Narrow, sharp, complete. Arc 218.5 now unblocked; arc 217 builds on a clean strict-EDN foundation.

*The dialect tightened. The boundary held. The substrate is honest.*
