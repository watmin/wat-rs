# BRIEF — Arc 218 Stone 218.2 — Naming sweep

**Stone scope (sonnet portion):** mechanical naming polish across `crates/wat-edn/` — rename `escapes.rs` → `vocab.rs` (orchestrator-locked target), three lexer variable renames, decode_utf8_char placement audit, doubled section header removal, arc-provenance migration. Foundation for stones 218.3-218.5 to operate on correct-naming substrate.
**Type:** Sonnet Mode A.
**Time budget:** 30-50 min target; 70 min STOP.
**Depends on:** Stone 218.1 (`57af300` — L1 fixes + cross-spell convergence; `write_keyword_body_to` lives in escapes.rs).
**Calibration:** Stone 218.1 SCORE (`docs/arc/2026/05/218-wat-edn-impeccable/SCORE-STONE-218.1.md`) — predicted 25-45 min, actual ~20 min, below lower band. Pattern: substrate-pre-grep + simple sweeps ship fast.
**Unblocks:** Stones 218.3 / 218.4 / 218.5.

## Rename target: vocab.rs (orchestrator-locked via inline four-questions)

`escapes.rs` was honest pre-218.1 (named chars + string escape decode). Stone 218.1 added `write_keyword_body_to<W: fmt::Write>` — keyword segment encoding (the `,` → `_` swap at depth ≥ 1). Module content is now broader than "escapes."

Four-questions on the rename target:

| Candidate | Obvious? | Simple? | Honest? | Good UX? |
|---|---|---|---|---|
| `vocab.rs` | YES — captures named chars + escapes + keyword segment encoding rules (all spec-level vocabulary) | YES — single short word | YES — accurately describes post-218.1 content; matches existing module rustdoc "Single source of truth for spec-level vocabulary shared between the lexer and the writer" | YES — readers looking for keyword-encoding logic naturally find it under `vocab` |
| `chars.rs` | NO — `write_keyword_body_to` is byte/string keyword-encoding, not chars | YES | NO — module contains keyword-segment encoding beyond chars | NO — readers searching for keyword-body logic in `chars.rs` would be confused |

**YES × 4 for `vocab.rs`. Decision locked.** Sonnet does not re-litigate; ship the rename.

## Pre-flight verified (orchestrator-grep'd 2026-05-21, post-218.1)

- **`crates/wat-edn/src/lib.rs:69`** — `pub mod escapes;` → becomes `pub mod vocab;`
- **`crates/wat-edn/src/parser.rs:8`** — `use crate::escapes::validate_first_char;`
- **`crates/wat-edn/src/parser.rs:465`** — comment text "lives in `crate::escapes`"
- **`crates/wat-edn/src/writer.rs:20`** — `use crate::escapes::{char_to_name, encode_string_escape, write_keyword_body_to};`
- **`crates/wat-edn/src/lexer.rs:36`** — `use crate::escapes::{` (multi-line import)
- **`crates/wat-edn/src/lexer.rs:623`** — comment text "live in `crate::escapes`"
- **`crates/wat-edn/src/value.rs`** — 18 inline `crate::escapes::` references at lines 224, 236, 238, 252, 263, 264, 299, 309, 311, 323, 334, 335, 371, 373, 391, 392, 440, 443
- **Lexer var sites (post-218.1):**
  - `process_escape` at `lexer.rs:247-267`: `let e = self.advance()...` at line 248
  - `read_hex4` at `lexer.rs:269-284`: `let mut acc = 0u32;` at line 270
  - `lex_keyword` body parsing at `lexer.rs:~378`: `let mut owned: Option<String> = None;`
- **`decode_utf8_char`** — defined at `lexer.rs:626`; `#[cfg(test)]` at `lexer.rs:652`. **Function IS already above tests.** Vigilia's L2 #5 finding ("placed below #[cfg(test)]") does NOT reproduce post-218.1 substrate state. Sonnet AUDITs and surfaces honest delta (the finding doesn't fire; nothing to move).
- **Doubled section header in value.rs (post-218.1 shifted):**
  - `value.rs:453` — `// ─── Convenience accessors ──────────────────────────────────────`
  - `value.rs:479` — `// ─── Convenience accessors ──────────────────────────────────` (the duplicate)
- **Arc-provenance in lib.rs:**
  - `lib.rs:175-176` inside `///` doc — "Arc 092. The first consumer is `wat-measure`'s `WorkUnit`, which keys every measurement scope by uuid."

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope (sonnet)

### Part A — File rename + global namespace sweep

1. **Git-rename the file** (preserves history):
   ```
   git mv crates/wat-edn/src/escapes.rs crates/wat-edn/src/vocab.rs
   ```
2. **Update `pub mod escapes;` → `pub mod vocab;`** at `crates/wat-edn/src/lib.rs:69`.
3. **Sweep all `crate::escapes::` → `crate::vocab::`** across the 4 callsite files (parser.rs, writer.rs, lexer.rs, value.rs). 24 references total per orchestrator grep. A single `find + sed`-style replacement is fine; verify with `cargo build` after.
4. **Update comments referencing `crate::escapes`** at `parser.rs:465` + `lexer.rs:623` → `crate::vocab`.
5. **Update vocab.rs's own internal references** if any (the module's own docstring may say "escapes" — preserve that text where the meaning is still about string escape decode; rename only the MODULE NAME references).

### Part B — Lexer variable renames

6. **`process_escape` at `lexer.rs:247-267`**: rename local `e` (the escape byte after `\`) → `escape_byte`. Single binding; ~3 use sites within the function body (`decode_string_escape(e)`, `e == b'u'`, the final `ErrorKind::InvalidEscape(e)`).
7. **`read_hex4` at `lexer.rs:269-284`**: rename local `acc` (the accumulating codepoint value) → `codepoint`. Single binding; 2-3 use sites (`acc = (acc << 4) | (v as u32)`, the final `Ok(acc)`, and the error format string `U+{:04X}` arg).
8. **`lex_keyword` at `lexer.rs:~378`**: rename local `owned: Option<String>` (the optional decoded-body buffer) → `decoded_body`. Audit all use sites within the function.

### Part C — decode_utf8_char placement AUDIT (honest report)

9. **Audit `decode_utf8_char` placement.** Per VIGILIA-REPORT L2 #5 the finding was "placed below `#[cfg(test)]` block." Per orchestrator pre-flight: function defined at `lexer.rs:626`; `#[cfg(test)]` at `lexer.rs:652`. **Function IS already above tests.** Confirm this with your own grep; if the finding doesn't reproduce, document the delta in SCORE (the finding doesn't fire — nothing to move). If the function actually IS below #[cfg(test)] in some way orchestrator missed, surface and move it.

### Part D — Doubled section header

10. **Remove the inner `// ─── Convenience accessors ──` banner at `value.rs:479`**. The outer one at `value.rs:453` stays. Sonnet verifies the inner one is the duplicate by reading both lines + surrounding context (~5 lines each); if the inner is actually marking a DIFFERENT section, surface STOP and don't delete.

### Part E — Arc-provenance migration

11. **Move arc-provenance from `lib.rs:175-176`** out of the public `///` doc comment. The current text:
    ```
    /// Arc 092. The first consumer is `wat-measure`'s `WorkUnit`, which
    /// keys every measurement scope by uuid.
    ```
    is arc-history that doesn't belong in user-facing doc (arc-provenance is for the substrate maintainers). Move it to an INTERNAL non-doc comment (`//` not `///`) — either just before the `#[cfg(feature = "mint")]` line or as the first line inside the function body. Preserve the historical text exactly; only its visibility changes (public doc → internal comment). The remaining `///` doc keeps the user-facing intent (what the function does, when it's available, # Example).

### Part F — Verification

12. **Run the wat-edn test suite — verify zero regressions:**
    ```
    cargo build --release -p wat-edn
    cargo test --release -p wat-edn
    cargo clippy --release -p wat-edn -- -D warnings
    ```
    Baseline: 336/336 PASS (per Stone 218.1 SCORE). Pure rename + variable renames + comment moves — semantics unchanged; the 336 must hold exactly.

### Part G — SCORE

13. **SCORE doc** at `docs/arc/2026/05/218-wat-edn-impeccable/SCORE-STONE-218.2.md` — scorecard matching EXPECTATIONS row count; deltas (especially the decode_utf8_char audit outcome); verification summary; elapsed time. Calibration shape per `SCORE-STONE-218.1.md`.

## NOT your scope

- Stone 218.3 contract precision (pretty-print symmetry, `.expect()` runes, parse_map_key, closer-token diagnostics, allocation bounds, identifier suffix scan) — separate stone
- Stone 218.4 UUID strictness + USER-GUIDE map format claim + parse_wire docs — separate stone
- Stone 218.5 public-API runes + INSCRIPTION + re-cast vigilia — closure paperwork
- Touching content of vocab.rs beyond updating internal module-name references (no logic changes; this is a pure rename)
- Renaming non-flagged variables in lexer.rs (only `e`, `acc`, `owned` get renamed; other names stay)
- Substantive refactoring — this stone is naming only

## STOP triggers

- **STOP-1: wat-edn test regresses** — pure rename should not break tests; surface immediately
- **STOP-2: `cargo clippy -- -D warnings` finds new warnings post-rename** — surface; might indicate missed import sites
- **STOP-3: decode_utf8_char IS below #[cfg(test)] contrary to orchestrator's grep** — surface (orchestrator's grep was at line 626 < 652; sonnet's audit takes precedence if it shows differently)
- **STOP-4: doubled-header at value.rs:479 is actually marking a different section** — surface; do not delete
- **STOP-5: arc-provenance text loses fidelity in the migration** — surface; preserve text exactly
- **STOP-6: 70 min elapsed**

## Verification (one per line)

```
cargo build --release -p wat-edn
cargo test --release -p wat-edn
cargo clippy --release -p wat-edn -- -D warnings
```

## When you finish

Report: pass count out of EXPECTATIONS row count, deltas (especially the decode_utf8_char audit outcome — finding fired or didn't), verification summary, elapsed time. Cite the rename's `git status` showing `R  escapes.rs -> vocab.rs` (git mv preserves history).

Don't commit. Orchestrator commits after review.
