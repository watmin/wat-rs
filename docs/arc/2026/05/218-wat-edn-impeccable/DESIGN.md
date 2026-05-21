# Arc 218 — wat-edn IMPECCABLE

**Opened:** 2026-05-21
**Branch:** `arc-170-gap-j-v5-deadlock-state`
**Blocks:** arc 217 (Clojure-IPC bridge) → arc 216 closure (216.8 / 216.9 / 216.10)
**Trigger:** vigilia cast 2026-05-21 on `crates/wat-edn/src/` returned **DIVERGES (2 L1 + 26 L2)**. Practitioner: *"this is the beginning of our arc to extend wat-edn — get it clean first — then we work upon it."*

## Mission

Bring `crates/wat-edn/` to **CONVERGED** under vigilia — zero L1 + zero L2 across all 7 defensive spells. The 28 findings from the 2026-05-21 cast are the worklist. Each gets either a fix or an honest rune annotation.

After arc 218 closes:
- arc 217 (Clojure-IPC bridge) extends a clean substrate
- arc 216 stones 216.8 (sum-type tagged literals) + 216.9 (EDN tagged scalars) build on the impeccable foundation
- The "near impeccable" practitioner assessment becomes literally impeccable

## The findings

See `VIGILIA-REPORT-2026-05-21.md` (this directory) for the full per-spell aggregate. Summary:

| Spell | Convergence | Count |
|---|---|---|
| sequi | CONVERGED | 0 |
| solvere | DIVERGES | 1 L2 |
| temperare | DIVERGES | 1 L1 + 1 L2 |
| cernere | DIVERGES | 1 L1 + 3 L2 |
| struere | DIVERGES | 5 L2 |
| purgare | DIVERGES | 7 L2 |
| intueri | DIVERGES | 9 L2 |

**Cross-spell convergence:** `value.rs:451` `write_keyword_segment` vs `writer.rs:177` `write_keyword_body` — flagged by both solvere + intueri. Strongest signal in the report.

## The four-questions on the arc

| | |
|---|---|
| **Obvious?** | YES — vigilia surfaced 28 specific findings with file:line citations and fix directions. The worklist is unambiguous. Each finding has a YES/NO close: either the fix lands, or a rune annotates the intentional exemption. |
| **Simple?** | YES — composed of mechanical fixes (renames, doc edits, helper extractions, rune annotations). Each stone has a settled foundation (vigilia's per-spell reports); none introduces new substrate; verification is a re-cast of vigilia. |
| **Honest?** | YES — the arc closes only when wat-edn CONVERGES under a re-cast vigilia. No deferral language; no "future polish"; the L2s are addressed or rune'd. Practitioner's "near impeccable" becomes literally impeccable. |
| **Good UX?** | YES — arc 217 (Clojure-IPC bridge) inherits a clean substrate; future wat-edn consumers (arc 216.8/216.9; downstream LLMs reading USER-GUIDE; cross-language bridge users) get a substrate that doesn't lie. The cost is paid once; the benefit cascades. |

**YES × 4.** Arc 218 stands.

## Stone decomposition

Five stones. Each verifiable independently; ordered so foundation lands first.

| # | Stone | Scope | Why |
|---|---|---|---|
| **218.1** | L1 fixes + cross-spell convergence | (a) cernere L1 — rewrite USER-GUIDE.md:159 + IPC-BRIDGE.md:212 phantom `Parser::parse_next` examples to use `Parser::new(input).parse_all()?` (b) temperare L1 — single-iterator pattern at `lexer.rs:346-347` (c) extract shared `write_keyword_body_to<W: Write>` in escapes.rs (or `vocab.rs` if 218.2 lands first); collapse value.rs:451 + writer.rs:177 to one source of truth | Highest-priority. L1s are lies; the cross-spell finding is the strongest signal. Foundation. |
| **218.2** | Naming sweep | (a) rename `escapes.rs` → `vocab.rs` or `chars.rs` (intueri to choose; cast intueri before the rename); update all imports (b) lexer var renames: `e`→`escape_byte` / `acc`→`codepoint` / `owned`→`decoded_body` (c) move `decode_utf8_char` above `#[cfg(test)]` (d) remove doubled section header `value.rs:503` (e) move arc-provenance from `lib.rs:191` public `new_uuid_v4` doc to internal comment | Mechanical renames; settled foundation for stones 218.3+ to use the right names |
| **218.3** | Contract precision | (a) writer pretty-print map symmetry — emit `\n` + indent before EVERY entry OR document + test intentional asymmetry (b) `to_json_string` `.expect()` rune annotation `invariant-coupling` citing closed edn_to_json construction (c) `parse_map_key` strict mode OR documented + tested silent-fallback path (d) parser closer-token diagnostic split — `Eof` stays; `RParen`/`RBracket`/`RBrace` get `UnexpectedByte` (e) `lexer.rs:213` `String::with_capacity` use `self.pos - body_start` (f) parser identifier suffix scan — fold via `splitn(3, '/')` | Correctness + diagnostic precision. Each independent; bundles cleanly. |
| **218.4** | UUID strictness + docs | (a) `is_canonical_uuid` enforces `is_ascii_lowercase \|\| is_ascii_digit` for non-hyphen positions (b) `decode_uuid` (json.rs) applies same canonical-strict check (c) USER-GUIDE map format claim fix — "single space separator" not "comma-space" (d) USER-GUIDE add `parse_wire` / `parse_wire_owned` documentation (real public functions; currently undocumented) | Honest contract: docs match code; UUID strictness symmetric across EDN + JSON paths |
| **218.5** | Public-API runes + INSCRIPTION + re-cast | (a) add `// rune:purgare(public-api)` (or `future-fixture`) annotations to all 7 forward-declared re-exports: `write_to`, `to_json_string_pretty`, `edn_to_json`, `json_to_edn`, `JsonError`, `JsonResult`, `parse_wire_owned` (b) re-cast vigilia on `crates/wat-edn/src/` — verify CONVERGED (zero L1 + zero L2) (c) INSCRIPTION-218.md inscribed; cross-reference VIGILIA-REPORT-2026-05-21.md (immutable record of the divergent cast) + the new CONVERGED cast | Closure paperwork + the impeccability proof. Arc closes only when re-cast vigilia returns CONVERGED across all 7 spells. |

## Stepping-stones discipline applied

Per Recovery Doc § 5: each stone benefits from prior settled foundation.
- 218.1 lands L1 fixes + cross-spell convergence first because they're the strongest signals; subsequent stones operate against a substrate without lies
- 218.2 lands renames second because subsequent fixes (218.3/218.4) reference the correct names
- 218.3 + 218.4 are independent in content but ordered by stake — 218.3 is correctness; 218.4 is contract honesty + docs
- 218.5 is the gate — the arc closes only when vigilia re-casts CONVERGED

## What this arc does NOT do

- **Extend wat-edn** — that's arc 217 (Clojure-IPC bridge). Arc 218 brings it to impeccability; arc 217 builds on it.
- **Touch wat-rs callers** — wat-edn's surface is preserved (or its forward-declared surface gets rune annotations rather than removal). No consumer-side migration.
- **Address arc 216 work** — arc 216 stones 216.8 / 216.9 / 216.10 remain BLOCKED on arc 217 closing.

## Cross-references

- `VIGILIA-REPORT-2026-05-21.md` — full 28-finding aggregate; the worklist
- `crates/wat-edn/` — the target package; 8 source files, ~3456 lines
- Arc 092 (`wat-edn v4 minting`) — wat-edn's substrate origin
- Arc 206 + Arc 207 — Uuid integration; affects 218.4
- Arc 170 1f-W — wire encoding lexical doctrine; affects 218.3 closer-token + 218.4 wire docs
- Task #428 — wat-edn wards (completed; the cast that produced the worklist)
- Task #429 — arc 217 Clojure-IPC bridge (blocked on arc 218 closing)
- Tasks #425 / #426 / #427 — arc 216 closing stones (blocked on arc 217)
- `feedback_inscription_immutable` — VIGILIA-REPORT-2026-05-21.md is historical record; immutable
- `feedback_spells_cast_via_subagent` — wards cast via Agent per SKILL.md protocol
- `feedback_ward_isolation` — one agent per ward; cross-talk illegal
- `feedback_ward_zone_comms_only` — pre-218, ward zone was comms; arc 218 extends the zone to wat-edn
- `project_wat_llm_first_design` — LLM-first; doc honesty is load-bearing for downstream consumers

## Status

Arc 218 opens with this DESIGN + the VIGILIA report inscribed. Stone 218.1 queued as next concrete work. Practitioner: *"the beginning of our arc to extend wat-edn — get it clean first — then we work upon it."*

*The pieces guard each. The whole guards everything. The arc makes the whole impeccable.*
