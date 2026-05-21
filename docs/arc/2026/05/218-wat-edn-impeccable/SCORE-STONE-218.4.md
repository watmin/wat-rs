# SCORE — Arc 218 Stone 218.4 — UUID strictness + USER-GUIDE doc fixes

**Mode:** A
**Agent:** claude-sonnet-4-6
**Date:** 2026-05-21

## Result: 9/9 PASS

| # | Row | Status | Citation |
|---|---|---|---|
| 1 | `is_canonical_uuid` lowercase enforcement | PASS | `crates/wat-edn/src/parser.rs:466` — `!b.is_ascii_hexdigit()` replaced with `!(is_dash \|\| b.is_ascii_digit() \|\| (b'a'..=b'f').contains(&b))`. Clippy preferred the De Morgan form over the two-negation form. Function now matches docstring claim at line 451 ("lowercase hexadecimal characters"). |
| 2 | `decode_uuid` (JSON bridge) canonical-strict | PASS | `crates/wat-edn/src/json.rs:390-400` — `is_canonical_uuid(s)` check inserted before `uuid::Uuid::parse_str`. Returns `JsonError::InvalidUuid("...: not canonical 8-4-4-4-12 lowercase form")` on non-canonical form. Symmetric strictness with EDN path at `parser.rs:297`. |
| 3 | `is_canonical_uuid` visibility | PASS | `pub(crate)` in `parser.rs:455`. Option A chosen (minimal disturbance — one keyword change). `json.rs` imports via `use crate::parser::is_canonical_uuid;` added at line 36. No vocab.rs move needed; no ripple beyond the two files. |
| 4 | USER-GUIDE map separator claim fixed | PASS | `crates/wat-edn/docs/USER-GUIDE.md:231-232` — "separated by `, `" → "separated by a single space. EDN treats commas as whitespace, so a reader will accept `{:a 1, :b 2}` and `{:a 1 :b 2}` identically — the writer chooses the compact form." EDN-whitespace-comma teaching context preserved. |
| 5 | USER-GUIDE assertion example fixed | PASS | `crates/wat-edn/docs/USER-GUIDE.md:292` (was 294) — `{:id 1, :name "Alice"}` → `{:id 1 :name "Alice"}`. Comma removed; matches what `write_map` actually emits per `writer.rs:338`. |
| 6 | USER-GUIDE comma-separator sweep | PASS | `grep -n ", :" USER-GUIDE.md` yielded 3 matches. Classification: line 227 (`{:asset :BTC, :side :Buy}` in "Output style" block) — writer-output claim, FIXED (comma removed). Line 292 — writer-output claim, FIXED (row 5 above). Line 556 (Clojure REPL comment `(wat/gen ...)`) — NOT a Rust writer-output claim; it shows hypothetical Clojure-side output. KEPT. Total writer-output claims found: 2 (both fixed). One additional match classified as Clojure-side example; kept. Sweep count: 3 matches, 2 fixed, 1 kept. |
| 7 | USER-GUIDE adds `parse_wire` + `parse_wire_owned` docs | PASS | `crates/wat-edn/docs/USER-GUIDE.md:140-170` area — section header updated to "Four free-function entry points and a `Parser` builder"; `parse_wire` and `parse_wire_owned` added to import list and documented with brief explanation of wire-mode (`,` → `_` swap inside parametric type arglists `<…>`). Cross-reference to existing `Parser::new_wire` entry. |
| 8 | Probes for strictness fixes (2 added) | PASS | **Probe 1:** `rejects_uuid_uppercase_hex` in `crates/wat-edn/tests/spec_strict.rs:227-233` — placed immediately after `accepts_canonical_uuid` (same INV-5 family). Asserts `parse(r#"#uuid "550E8400-E29B-41D4-A716-446655440000""#).is_err()`. Passes. **Probe 2:** `decode_uuid_rejects_uppercase_via_json_bridge` in `crates/wat-edn/src/json.rs` internal `#[cfg(test)]` block — uses `from_json_string` with `{"#uuid": "550E8400-..."}` (escaped-quote form to avoid raw-string ambiguity). Asserts `Err(JsonError::InvalidUuid(_))`. Passes. |
| 9 | wat-edn test suite: 339/339 PASS | PASS | `cargo build --release -p wat-edn` — OK (0 warnings, 0 errors). `cargo test --release -p wat-edn` — **339/339 PASS** (44 unit + 16 accessor + 176 comprehensive + 4 display_equivalence + 8 pretty + 7 round_trip + 23 spec_conformance + 37 spec_strict + 0 uuid_v4_mint + 23 wire_encoding + 1 doc-test). `cargo clippy --release -p wat-edn -- -D warnings` — **0 warnings, 0 errors**. |

## Deltas from EXPECTATIONS

**Delta 1 — Clippy required De Morgan form for boolean.**
BRIEF offered `b.is_ascii_digit() || (b'a'..=b'f').contains(&b)` under `!is_dash && !( )`. Clippy's `nonminimal_bool` lint (`-D warnings`) rejected the double-negation structure and required `!(is_dash || b.is_ascii_digit() || (b'a'..=b'f').contains(&b))`. Semantically identical; clippy-clean form used.

**Delta 2 — Probe 2 raw-string ambiguity — used escaped-quote form.**
JSON for `{"#uuid": "..."}` contains the `"#uuid"` key, which embeds a double-quote inside a raw string (`r#"..."#`). The `"` inside the `#uuid` key terminates the raw string before the closing `"#`. Fix: use standard escaped-string literals `"{\"#uuid\": \"...\"}`. No semantic change; same JSON value.

**Delta 3 — USER-GUIDE comma sweep found 3 matches, not 2.**
Vigilia cited lines 233 + 294 specifically. The sweep at lines 227 + 231-232 + 292 revealed: the `{:asset :BTC, :side :Buy}` in the "Output style" verbatim block (line 227) was a third writer-output claim not mentioned by vigilia. Fixed (comma removed). Line 556 (Clojure REPL) was the third hit by the grep; classified as non-Rust-writer and kept. STOP-3 did NOT trigger — the extra claim was mechanical to fix (one comma removal in a verbatim block).

**No other deltas.** STOP triggers 1-6 did not fire.

## Verification summary

```
cargo build --release -p wat-edn          — OK (0 warnings, 0 errors)
cargo test --release -p wat-edn           — 339/339 PASS (337 baseline + 2 new probes; additive)
cargo clippy --release -p wat-edn -- -D warnings  — 0 warnings, 0 errors
```

## Files changed

- `crates/wat-edn/src/parser.rs` — `is_canonical_uuid` visibility `fn` → `pub(crate)`; lowercase-only check in hex-digit branch
- `crates/wat-edn/src/json.rs` — `use crate::parser::is_canonical_uuid` import added; `decode_uuid` canonical check inserted before `parse_str`; new probe `decode_uuid_rejects_uppercase_via_json_bridge`
- `crates/wat-edn/tests/spec_strict.rs` — new probe `rejects_uuid_uppercase_hex` after `accepts_canonical_uuid`
- `crates/wat-edn/docs/USER-GUIDE.md` — parse section updated (four entry points + parse_wire docs); Output style verbatim block corrected (comma removed); map-separator claim text fixed; assertion example comma removed

## STOP triggers

- **STOP-1 (existing UUID test regresses on strictness):** DID NOT TRIGGER. All existing UUID tests use lowercase canonical form; strictness change is transparent to them.
- **STOP-2 (`is_canonical_uuid` visibility change ripples beyond parser.rs + json.rs):** DID NOT TRIGGER. `pub(crate)` in parser.rs + one import in json.rs. Zero other consumers.
- **STOP-3 (USER-GUIDE has more comma-separator claims than vigilia cited):** DID NOT TRIGGER (monitoring condition: "surface count"). Sweep found 3 matches total (1 extra beyond vigilia's 2). The extra was in the "Output style" verbatim block — one comma removal, mechanical. No pause needed; reported in Delta 3.
- **STOP-4 (wat-edn test regresses):** DID NOT TRIGGER. 339 PASS (337 + 2 additive).
- **STOP-5 (clippy new warnings):** Clippy fired once on the boolean form (Delta 1). Fixed immediately. Final run: 0 warnings.
- **STOP-6 (55 min elapsed):** DID NOT TRIGGER.

## Elapsed time

Target: 20-40 min. Actual: ~20 min. At lower bound.

## Calibration check

- Target runtime: 20-40 min
- Actual runtime: ~20 min
- Within prediction band? At lower bound
- Rationale: Orchestrator pre-greps were accurate and complete (all line numbers confirmed; file layout as expected). One compile-then-clippy cycle for the boolean form (one edit); one compile error on the raw-string probe (one edit). Remainder was mechanical doc edits and test reads. Pattern mirrors 218.1-218.3: substrate-pre-grep + locked-decisions + mechanical edits ships at or below lower band. Calibration now four points: 218.1 ~20, 218.2 ~15, 218.3 ~25, 218.4 ~20. All at or below lower bound. The band itself is calibrated correctly for the work type; actual execution is consistently near the lower end.
