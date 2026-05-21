# EXPECTATIONS — Arc 218 Stone 218.2 — Naming sweep

Mode A target: 11/11 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | File renamed via `git mv` | `crates/wat-edn/src/escapes.rs` → `crates/wat-edn/src/vocab.rs`; `git status` shows `R  ...escapes.rs -> ...vocab.rs` (rename preserves history) |
| 2 | `lib.rs` mod declaration updated | `crates/wat-edn/src/lib.rs:69` `pub mod escapes;` → `pub mod vocab;` |
| 3 | All 24 `crate::escapes::` callsites swept to `crate::vocab::` | parser.rs (1 use + 1 comment) + writer.rs (1 use) + lexer.rs (1 use + 1 comment) + value.rs (18 inline refs) — all flipped; zero `crate::escapes` references remain post-sweep |
| 4 | `process_escape` var `e` → `escape_byte` | `crates/wat-edn/src/lexer.rs:~247-267`: local `let e = ...` renamed; all use sites within fn body updated; semantics unchanged |
| 5 | `read_hex4` var `acc` → `codepoint` | `crates/wat-edn/src/lexer.rs:~269-284`: local `let mut acc = 0u32` renamed; ~3 use sites updated; semantics unchanged |
| 6 | `lex_keyword` var `owned` → `decoded_body` | `crates/wat-edn/src/lexer.rs:~378`: local `let mut owned: Option<String> = None` renamed; all use sites within fn updated; semantics unchanged |
| 7 | `decode_utf8_char` placement audit + report | Audit run; finding outcome reported. Per orchestrator pre-flight (lexer.rs:626 < #[cfg(test)] at :652), the function IS already above tests — the vigilia L2 #5 finding doesn't reproduce post-218.1. SCORE delta documents the outcome honestly. If sonnet's audit contradicts orchestrator, sonnet's audit takes precedence (STOP-3 trigger) |
| 8 | Doubled section header removed | `crates/wat-edn/src/value.rs:479` inner `// ─── Convenience accessors ──` deleted; outer at `:453` preserved |
| 9 | Arc-provenance migrated to internal comment | `crates/wat-edn/src/lib.rs:175-176` "Arc 092..." text moved from `///` doc to `//` internal comment; user-facing doc retains intent (what the fn does + Example) sans arc-history |
| 10 | wat-edn test suite: zero regressions | `cargo test --release -p wat-edn` 336/336 PASS (exact match to 218.1 baseline); `cargo clippy --release -p wat-edn -- -D warnings` clean |
| 11 | SCORE doc inscribed | `docs/arc/2026/05/218-wat-edn-impeccable/SCORE-STONE-218.2.md` — scorecard + deltas + verification + elapsed time |

## Independent prediction (calibration record)

**Target runtime:** 30-50 min Mode A
**Upper bound:** 70 min
**Confidence:** high

**Rationale:**
- Pure rename + 3 variable renames + 1 audit + 1 line delete + 1 comment move
- 24 callsites to sweep but a single substitution pattern (`crate::escapes::` → `crate::vocab::`) covers them all
- Sonnet's 218.1 calibration: predicted 25-45, actual ~20 (below lower band) — substrate-pre-grep + simple sweeps ship fast
- Risk: a missed `escapes` reference somewhere not in orchestrator's grep (clippy --D-warnings will catch unused-import or undefined-module errors; STOP-2 trigger if surfaces)
- Risk: decode_utf8_char audit reveals an actual placement issue orchestrator missed (STOP-3 trigger)
- Risk: vocab.rs's own internal text references "escapes" in a way that the rename touches the WRONG thing (the BRIEF distinguishes module-name references from "string escape" content references)

**Per `feedback_stone_briefs_cite_prior_score`:** BRIEF cites Stone 218.1 SCORE (most recent calibration, mechanical sweep pattern). Pattern lineage: pure naming polish; smaller cognitive surface than 218.1.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Contract precision changes — Stone 218.3
- UUID strictness changes — Stone 218.4
- INSCRIPTION + re-cast vigilia — Stone 218.5
- Renaming variables NOT in the 3-name list (e + acc + owned) — preserve other naming
- Logic changes inside vocab.rs — only module-name references update
- DESIGN.md amendments — orchestrator-direct (none expected)

## Honesty deltas accepted

- decode_utf8_char audit: finding doesn't fire → honest "delta: vigilia L2 #5 does not reproduce post-218.1; function already above tests" in SCORE
- Variable rename spelling: if `codepoint` reads worse than `hex_accum` in context, sonnet picks the cleaner spelling and documents
- Arc-provenance internal-comment placement (just-above-cfg-attr vs first-line-inside-fn-body): sonnet picks the cleaner spot; documents

## Honesty deltas NOT accepted

- Renaming the module to anything OTHER than `vocab.rs` — STOP. Decision locked via four-questions YES×4 in the BRIEF; do not re-litigate.
- Touching content beyond the listed work items — STOP at the boundary; surface and pause
- Removing the doubled header WITHOUT verifying it's actually a duplicate — STOP-4 trigger
- Losing arc-provenance text fidelity — STOP-5 trigger
- Bypassing tests/clippy — never; the 336/336 must hold exactly
