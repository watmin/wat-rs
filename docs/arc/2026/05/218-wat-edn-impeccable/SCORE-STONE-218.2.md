# SCORE — Arc 218 Stone 218.2 — Naming sweep

**Mode:** A
**Agent:** claude-sonnet-4-6
**Date:** 2026-05-21

## Result: 11/11 PASS

| # | Row | Status | Citation |
|---|---|---|---|
| 1 | File renamed via `git mv` | PASS | `git status --short crates/wat-edn/src/` shows `R  crates/wat-edn/src/escapes.rs -> crates/wat-edn/src/vocab.rs`. History preserved via `git mv`. |
| 2 | `lib.rs` mod declaration updated | PASS | `crates/wat-edn/src/lib.rs:69` `pub mod escapes;` → `pub mod vocab;`. Single targeted edit. |
| 3 | All `crate::escapes::` callsites swept to `crate::vocab::` | PASS | 4 callsite files updated: parser.rs (1 use import), writer.rs (1 use import), lexer.rs (1 use import), value.rs (18 inline refs via replace_all). Additionally found and updated a comment in lexer.rs:331 that referenced `escapes::NAMED_CHARS` by module path — updated to `vocab::NAMED_CHARS`. Post-sweep grep confirms zero `::escapes`, `mod escapes`, `use.*escapes`, or `escapes::` patterns remain in src/. |
| 4 | `process_escape` var `e` → `escape_byte` | PASS | `crates/wat-edn/src/lexer.rs:~248-266`: `let e = self.advance()...` renamed to `let escape_byte = ...`; three use sites updated (`decode_string_escape(escape_byte)`, `if escape_byte == b'u'`, `ErrorKind::InvalidEscape(escape_byte)`). The separate `let acc = self.read_hex4()?` within the same function is a distinct variable; correctly left untouched. |
| 5 | `read_hex4` var `acc` → `codepoint` | PASS | `crates/wat-edn/src/lexer.rs:~269-284`: `let mut acc = 0u32` renamed to `let mut codepoint = 0u32`; two use sites updated (`codepoint = (codepoint << 4) \| (v as u32)`, `Ok(codepoint)`). |
| 6 | `lex_keyword` var `owned` → `decoded_body` | PASS | `crates/wat-edn/src/lexer.rs:~378`: `let mut owned: Option<String> = None` renamed to `let mut decoded_body: Option<String> = None`; all four use sites updated within the function body (`decoded_body.is_none()`, `decoded_body = Some(...)`, `decoded_body.as_mut().unwrap().push(',')`, `if let Some(buf) = decoded_body.as_mut()`, and `match decoded_body { Some(s) => ..., None => ... }`). |
| 7 | `decode_utf8_char` placement audit + report | PASS (finding doesn't fire) | Grep confirms: `decode_utf8_char` defined at `lexer.rs:626`; `#[cfg(test)]` at `lexer.rs:652`. Function IS already above the test block. **Vigilia L2 #5 finding does not reproduce post-218.1 substrate state.** Delta: the function was already correctly placed before stone 218.1 landed (218.1's single-iterator fix was in `lex_char`, not this function). Nothing to move. |
| 8 | Doubled section header removed | PASS | `crates/wat-edn/src/value.rs:~479` — inner `// ─── Convenience accessors ──────────────────────────────────` banner removed. Outer banner at line 453 preserved. The descriptive comment block below the inner header (explaining `as_*` return contract) was retained — it carries meaning and is now an inline comment under the outer section. No content loss. |
| 9 | Arc-provenance migrated to internal comment | PASS | `crates/wat-edn/src/lib.rs:~175-176` — "Arc 092. The first consumer is `wat-measure`'s `WorkUnit`, which keys every measurement scope by uuid." removed from `///` doc and inserted as `//` internal comment between the closing doc-test ` ``` ` and the `#[cfg(feature = "mint")]` attribute. Text preserved exactly. User-facing `///` doc retains: what the fn does (UUID output format, RFC 9562 + strictness), availability note (`mint` feature), and `# Example` block. Arc-history no longer appears in rustdoc output. |
| 10 | wat-edn test suite: zero regressions | PASS | `cargo build --release -p wat-edn` — OK (0 warnings, 0 errors). `cargo test --release -p wat-edn` — **336/336 PASS** (42 unit + 16 accessor + 176 comprehensive + 4 display_equivalence + 8 pretty + 7 round_trip + 23 spec_conformance + 36 spec_strict + 0 uuid_v4_mint + 23 wire_encoding + 1 doc-test). `cargo clippy --release -p wat-edn -- -D warnings` — **0 warnings, 0 errors**. |
| 11 | SCORE doc inscribed | PASS | This file. |

## Deltas from EXPECTATIONS

**Delta 1 — Extra comment updated: `lexer.rs:331` `escapes::NAMED_CHARS`.**
The orchestrator's pre-flight grep covered `crate::escapes::` paths but the comment at lexer.rs:331 referenced `escapes::NAMED_CHARS` without the `crate::` prefix. This is still a module-path reference (not "string escape" domain content), so it was updated to `vocab::NAMED_CHARS` for consistency. Zero semantic change; comment accuracy improved.

**Delta 2 — decode_utf8_char audit: finding doesn't fire.**
Vigilia L2 #5 ("placed below `#[cfg(test)]` block") does not reproduce post-218.1. Function at line 626; `#[cfg(test)]` at line 652. Already correct. No move required. SCORE documents honest delta: "nothing to move."

**Delta 3 — `lex_keyword` rename: 5 use sites, not "audit all."**
The BRIEF said "audit all use sites within the function." The variable appears at: declaration, `.is_none()` check, assignment, `.as_mut().unwrap().push(',')` call, `if let Some(buf) = decoded_body.as_mut()` arm, and the final `match decoded_body`. All updated correctly. No missed sites.

**No other deltas.** STOP triggers 1-6 did not fire.

## Verification summary

```
cargo build --release -p wat-edn          — OK (0 warnings, 0 errors)
cargo test --release -p wat-edn           — 336/336 PASS (zero regressions)
cargo clippy --release -p wat-edn -- -D warnings  — 0 warnings, 0 errors
```

`display_equivalence.rs`: **4/4 PASS** — keyword writer path through `vocab::write_keyword_body_to` (renamed module) still byte-identical to `Display`. Rename didn't disturb the semantic lock.

## Files changed

- `crates/wat-edn/src/escapes.rs` → `crates/wat-edn/src/vocab.rs` (git mv, history preserved)
- `crates/wat-edn/src/lib.rs` — `pub mod escapes;` → `pub mod vocab;`; arc-provenance migrated from `///` to `//`
- `crates/wat-edn/src/lexer.rs` — import updated; comment text updated (×2); `e` → `escape_byte`; `acc` → `codepoint` (in `read_hex4`); `owned` → `decoded_body` (in `lex_keyword`)
- `crates/wat-edn/src/parser.rs` — import updated; comment text updated
- `crates/wat-edn/src/value.rs` — 18 inline `crate::escapes::` refs → `crate::vocab::`; doubled section header removed
- `crates/wat-edn/src/writer.rs` — import updated

## STOP triggers

- **STOP-1 (test regresses):** DID NOT TRIGGER. 336/336.
- **STOP-2 (clippy new warnings):** DID NOT TRIGGER. 0 warnings, 0 errors.
- **STOP-3 (decode_utf8_char below #[cfg(test)]):** DID NOT TRIGGER. Function at :626, cfg(test) at :652. Already correctly placed.
- **STOP-4 (doubled header marks different section):** DID NOT TRIGGER. Both headers identical text ("Convenience accessors"); inner was the true duplicate; outer section context confirmed by reading surrounding lines.
- **STOP-5 (arc-provenance text lost):** DID NOT TRIGGER. Text preserved exactly character-for-character.
- **STOP-6 (70 min elapsed):** DID NOT TRIGGER.

## Elapsed time

Target: 30-50 min. Actual: ~15 min. Below lower bound.

## Calibration check

- Target runtime: 30-50 min
- Actual runtime: ~15 min
- Within prediction band? Below lower end — faster than predicted
- Rationale: Orchestrator pre-greps were accurate and complete (one minor extra: the `escapes::NAMED_CHARS` comment without `crate::` prefix). All variable rename sites were within bounded function bodies. The `replace_all` on value.rs's 18 inline refs took one edit call. Audit findings (Part C) resolved instantly via grep confirmation. No ambiguity at any step. Pattern mirrors Stone 218.1 calibration: substrate-pre-grep + mechanical sweeps ship faster than predicted.
