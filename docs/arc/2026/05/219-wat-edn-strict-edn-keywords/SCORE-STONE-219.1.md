# SCORE — Arc 219 Stone 219.1 — Substrate strict-EDN + constructor translation + test sweep

**Mode:** A
**Agent:** claude-sonnet-4-6
**Date:** 2026-05-21

## Result: 11/11 PASS

| # | Row | Status | Citation |
|---|---|---|---|
| 1 | vocab.rs strict-EDN char set | PASS | `crates/wat-edn/src/vocab.rs:106-124` — `b':'` and `b'#'` removed from `is_symbol_continue`. Final set: alphanumeric + `. * + ! - _ ? $ % & = < > /`. `b'<' | b'>'` preserved (STOP-3 did not trigger). Comment updated to explain the Option β boundary doctrine. |
| 2 | Translation helper added | PASS | `crates/wat-edn/src/value.rs` — `fn translate_wat_to_strict(ns: &str) -> String` added as a private module-level function before the Symbol impl block. Uses `ns.replace("::", ".")`. One-pass; idempotent. |
| 3 | `Symbol::ns` constructor translation | PASS | `value.rs` — `let ns_translated = translate_wat_to_strict(namespace.as_ref())` called first; `validate_first_char(&ns_translated)` and storage both use the translated form. |
| 4 | `Symbol::try_ns` constructor translation | PASS | Same pattern as `Symbol::ns`; `ns_translated` computed before `validate_first_char`. Validation runs against the strict form. |
| 5 | `Keyword::ns` + `Keyword::try_ns` constructor translation | PASS | Both sites updated with `ns_translated` pattern. |
| 6 | `Tag::ns` + `Tag::try_ns` constructor translation | PASS | Both sites updated. `Tag::ns` stores into `namespace: CompactString::from(ns_translated)` (non-optional, as Tag always has a namespace). |
| 7 | `from_parts_unchecked` UNCHANGED | PASS | All three `from_parts_unchecked` implementations (Symbol at line 289, Keyword at line 363, Tag at line 424) verified by grep + read. No translation added. STOP-1 did not trigger. |
| 8 | Wat-edn-internal test fixture sweep | PASS | Orchestrator found 5 sites; sonnet grep found the same 5 fixture sites PLUS 4 additional parse()-call sites in `wire_encoding.rs` that expected `::` to succeed (now illegal). All 9 sites updated: (1) `writer_preserves_underscore_outside_brackets`: `kw("rust::crossbeam_channel::Sender")` → `kw("rust.crossbeam_channel.Sender")`; write assertion updated. (2) `writer_preserves_underscore_with_brackets_outside`: `kw("rust::sync::Mutex<i64>")` → `kw("rust.sync.Mutex<i64>")`; write assertion updated. (3) `wire_decode_preserves_underscore_outside_brackets`: `parse_wire(":rust::crossbeam_channel::Sender")` → `.` form; name assertion updated. (4) `roundtrip_namespaced_parametric`: `kw("wat::core::HashMap<...>")` → `.` form; write assertion updated. (5) `source_accepts_underscore_outside_brackets`: three `::` cases → `.` form; `:foo_bar_baz` preserved. (6) `rust_mirror_underscore_forms_still_parse`: all six `::` cases → `.` form. (7) `source_namespaced_with_comma_parses`: parse call and name assertion → `.` form. No fixture sweep sites found in any other test file. `comprehensive.rs:520` (`parse("::foo").is_err()`) is a rejection test; unchanged and still correct. |
| 9 | Three new probes added | PASS | `spec_strict.rs` — `Keyword` import added to use-line. Three probes appended under `// ─── Arc-219: strict-EDN keyword bodies ───`: (1) `is_symbol_continue_rejects_colon` — asserts `!wat_edn::vocab::is_symbol_continue(b':')` and `!wat_edn::vocab::is_symbol_continue(b'#')`; (2) `parser_rejects_double_colon_in_keyword` — asserts `parse(":wat::core::HashMap").is_err()`; (3) `keyword_ns_translates_wat_to_strict` — `Keyword::ns("wat::core", "HashMap")` asserts `namespace() == Some("wat.core")` and `name() == "HashMap"`. All three pass. |
| 10 | wat-edn test suite: 342/342 PASS | PASS | `cargo build --release -p wat-edn` — OK (0 warnings, 0 errors). `cargo test --release -p wat-edn` — **342/342 PASS** (44 unit + 16 accessor + 176 comprehensive + 4 display_equivalence + 8 pretty + 7 round_trip + 23 spec_conformance + 40 spec_strict + 0 uuid_v4_mint + 23 wire_encoding + 1 doc-test). `cargo clippy --release -p wat-edn -- -D warnings` — **0 warnings, 0 errors**. |
| 11 | wat-rs workspace sanity check | PASS | `cargo build --release` — workspace builds clean (5 pre-existing dead_code warnings in `wat` crate `src/`; not new). `cargo test --release --lib` — 822 passed; 2 failed; 1 ignored. The 2 failures (`runtime::tests::hashmap_composite_key_errors` and `runtime::tests::hashset_rejects_composite_element`) are pre-existing: verified by git-stash isolation (they fail identically on HEAD before this stone's changes). STOP-4 did NOT trigger. |

## Deltas from EXPECTATIONS

**Delta 1 — Fixture sweep wider than orchestrator's 5 sites.**
Orchestrator found 5 fixture sites in `wire_encoding.rs` (lines 166, 174, 203, 246, 337). Sonnet's broader review found 4 additional parse()-call sites that expected `::` to succeed: `source_accepts_underscore_outside_brackets` (lines 106-108), `wire_decode_preserves_underscore_outside_brackets` (line 199), `rust_mirror_underscore_forms_still_parse` (lines 295-300), and `source_namespaced_with_comma_parses` (line 331). Total updated sites: 9. All 9 use `.` form post-sweep. The orchestrator's 5 were the constructor-call sites; the additional 4 were parse()-call sites that tested the old wat-extension behavior. All 9 are correct post-219.

**Delta 2 — `Keyword::new()` fixture cases left as non-namespaced keywords.**
`kw("rust::crossbeam_channel::Sender")` calls `Keyword::new()`, NOT `Keyword::ns()`. The translation helper only applies to the 6 `ns`/`try_ns` constructors. `Keyword::new()` stores the whole string as the `name` field (no namespace). After dropping `:` from `is_symbol_continue`, the PARSER would reject `:rust::crossbeam_channel::Sender`, but `Keyword::new()` itself does not invoke the parser. The constructor call sites for `kw()` were updated to use `.` form so that the write output and roundtrip tests remain correct under strict-EDN.

**No other deltas.** STOP triggers 1-6 did not fire.

## Verification summary

```
cargo build --release -p wat-edn          — OK (0 warnings, 0 errors)
cargo test --release -p wat-edn           — 342/342 PASS (339 baseline + 3 new probes; additive)
cargo clippy --release -p wat-edn -- -D warnings  — 0 warnings, 0 errors
cargo build --release                     — OK (5 pre-existing dead_code warnings in wat crate; not new)
cargo test --release --lib                — 822 PASS; 2 pre-existing failures; STOP-4 not triggered
```

## Files changed

- `crates/wat-edn/src/vocab.rs` — `is_symbol_continue` drops `b':'` and `b'#'`; comment updated with Option β doctrine
- `crates/wat-edn/src/value.rs` — `fn translate_wat_to_strict` helper added; `Symbol::ns`, `Symbol::try_ns`, `Keyword::ns`, `Keyword::try_ns`, `Tag::ns`, `Tag::try_ns` all call helper on namespace before validate + store; `from_parts_unchecked` for all three types UNCHANGED
- `crates/wat-edn/tests/wire_encoding.rs` — 9 fixture sites swept: 5 constructor-call sites + 4 parse()-call sites updated from `::` to `.` form; comments updated to reflect post-219 strict-EDN doctrine
- `crates/wat-edn/tests/spec_strict.rs` — `Keyword` import added; 3 new probes appended under Arc-219 section header

## STOP triggers

- **STOP-1 (`from_parts_unchecked` translation accidentally added):** DID NOT TRIGGER. All three unchecked paths verified by grep + read — no translation in any of them.
- **STOP-2 (a wat-edn test regresses without sweep-flip explanation):** DID NOT TRIGGER. 342 PASS; all 339 baseline tests continue to pass; 3 new probes are additive.
- **STOP-3 (`b'<' | b'>'` accidentally dropped):** DID NOT TRIGGER. Both bytes preserved in `is_symbol_continue`.
- **STOP-4 (wat-rs callers regress beyond crates/wat-edn/):** DID NOT TRIGGER. 2 failures in `runtime::tests` are pre-existing (verified by git-stash isolation before and after).
- **STOP-5 (clippy new warnings):** DID NOT TRIGGER. 0 warnings on `-p wat-edn`.
- **STOP-6 (95 min elapsed):** DID NOT TRIGGER.

## Elapsed time

Target: 45-75 min. Actual: ~35 min. Below lower bound.

## Calibration check

- Target runtime: 45-75 min
- Actual runtime: ~35 min
- Within prediction band? Below lower bound
- Rationale: Orchestrator pre-greps were accurate and complete for the constructor sites. The additional fixture sweep sites (4 parse()-call sites beyond the orchestrator's 5 constructor sites) required careful reading of the test suite but were mechanical to update once identified. One stash-and-verify cycle to confirm pre-existing failures. Pattern continues: substrate-pre-grep + locked-decisions + mechanical edits ships below lower bound. Calibration now five points: 218.1 ~20, 218.2 ~15, 218.3 ~25, 218.4 ~20, 219.1 ~35. The 219.1 surface was genuinely larger (dual-surface: substrate + extended fixture sweep), so the absolute time is higher, but still below the predicted lower bound.
