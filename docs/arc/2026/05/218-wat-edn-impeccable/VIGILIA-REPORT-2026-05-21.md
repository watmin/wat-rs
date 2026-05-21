# Vigilia Report — wat-edn — 2026-05-21

**Cast:** 7 defensive spells (intueri / solvere / purgare / struere / sequi / temperare / cernere) in parallel against `crates/wat-edn/src/` — all 8 source files + docs.

**Practitioner's pre-cast assessment:** "near impeccable."

**Verdict:** **DIVERGES (2 L1 + 26 L2)** — assessment holds; both L1s are mechanical (one-line code fix + one doc fix); 26 L2s are polish-level. **sequi CONVERGED** — substrate-level state-threading discipline impeccable.

---

## Per-spell summary

| Spell | Convergence | Concerns |
|---|---|---|
| **intueri** | 9 L2 | Naming polish: `escapes` module name too narrow; 4 lexer var names; doubled section header; arc-provenance in public doc; `sentinel` name |
| **solvere** | 1 L2 | `value.rs:451-469` `write_keyword_segment` duplicates `writer.rs:177` `write_keyword_body` encoding walk |
| **purgare** | 7 L2 | 7 public re-exports with no current external consumers (forward-declarations for arc 217+ bridge work): `write_to`, `to_json_string_pretty`, `edn_to_json`, `json_to_edn`, `JsonError`, `JsonResult`, `parse_wire_owned` |
| **struere** | 5 L2 | `writer.rs` pretty-print map asymmetry; `to_json_string` `.expect()` panic invariant undocumented; `parse_map_key` silent EDN parse fallback; `parser.rs:158` closing tokens reported as `UnexpectedEof`; `lexer.rs:213` over-allocation |
| **sequi** | CONVERGED | Zero L1 + zero L2. Every transformation chain threads state visibly through signatures. No globals/statics/lazy_static/Mutex/Arc carrying domain state. **Substrate-level discipline impeccable.** |
| **temperare** | 1 L1 + 1 L2 | **L1:** `lexer.rs:346-347` double `chars()` walk on char-literal body; **L2:** `parser.rs:382,391` second linear scan of identifier suffix |
| **cernere** | 1 L1 + 3 L2 | **L1:** USER-GUIDE.md:159 + IPC-BRIDGE.md:212 phantom `Parser::parse_next()` method (does not exist); **L2:** UUID uppercase hex accepted despite "strict canonical" doc claim; JSON bridge `decode_uuid` skips canonical check; USER-GUIDE map format claim wrong (commas claimed; spaces emitted) |

## Cross-spell convergence

The strongest signal — two spells independently flagging the same site:

**`value.rs:451-469` `write_keyword_segment` vs `writer.rs:177` `write_keyword_body`** — flagged by both **solvere** (encoding duplication) and **intueri** (name divergence for same algorithm). Two functions, byte-for-byte identical structure, different sink types (`fmt::Formatter` vs `&mut String`). Fix: extract shared `write_keyword_body_to<W: Write>` in `escapes.rs` (the EDN char vocabulary module); both callers delegate.

This is the foundation finding. Highest priority. The duplication risks divergence if the `,` → `_` encoding rule changes; the `display_equivalence` test suite locks them but the discipline says one source of truth, not two locked copies.

## L1 detail

**L1.A — cernere — phantom `Parser::parse_next` in two docs:**
- `crates/wat-edn/docs/USER-GUIDE.md:159` — example code shows `match p.parse_next()? { None => break, Some(v) => ... }`
- `crates/wat-edn/docs/IPC-BRIDGE.md:212` — same phantom referenced
- Reality: `Parser` exposes `new` / `new_wire` / `parse_top` / `parse_all`. NO `parse_next`. Return type `Option<Value>` also wrong.
- Impact: any LLM/user following USER-GUIDE hits compile error
- Fix: rewrite example to use `Parser::new(input).parse_all()?` (drains all forms)

**L1.B — temperare — double `chars()` walk in `lexer.rs:346-347`:**
```rust
if body_str.chars().count() == 1 {
    return Ok(Token::Char(body_str.chars().next().unwrap()));
```
Two iterator constructions + traversals. Hot path (char-literal parsing). Fix:
```rust
let mut it = body_str.chars();
if let Some(c) = it.next() {
    if it.next().is_none() {
        return Ok(Token::Char(c));
    }
}
```

## L2 themes (full list at end)

1. **Naming polish** — `escapes.rs` module name too narrow (holds full EDN char vocab including symbol predicates); 4 lexer var names (`e`/`acc`/`owned`/`decode_utf8_char` placement); `value.rs:503` doubled section header; `lib.rs:191` arc-provenance in public `new_uuid_v4` doc; `sentinel` in json.rs requires module-doc context
2. **Public-API forward-declarations (7)** — `write_to`, `to_json_string_pretty`, `edn_to_json`, `json_to_edn`, `JsonError`, `JsonResult`, `parse_wire_owned` — re-exported but no current external consumers. Rune candidates (`public-api` or `future-fixture`); arc 217 likely consumes some.
3. **Contract precision (5)** — `write_pretty_indented` map asymmetry (first key fused to `{`); `to_json_string` undocumented panic invariant on `.expect()`; `parse_map_key` silent fallback when key looks like EDN but fails to parse; `parser.rs:158` closing tokens diagnostic says `UnexpectedEof`; `lexer.rs:213` `String::with_capacity` over-allocates pathologically
4. **UUID strictness gap** — `is_canonical_uuid` accepts uppercase hex despite "strict canonical" doc claim; JSON bridge `decode_uuid` skips canonical check entirely
5. **Doc rot** — USER-GUIDE map format claim wrong (says comma-separated; actually space-separated); USER-GUIDE doesn't mention `parse_wire`/`parse_wire_owned` (real public functions)
6. **Temperare polish** — `parser.rs:382,391` second linear scan via `body.find('/')` then `name.contains('/')` (cold path; identifier suffix)
7. **Cross-spell convergence** — see above

## Full L2 inventory

| # | Source spell | File | Line | Finding | Fix direction |
|---|---|---|---|---|---|
| 1 | intueri | `escapes.rs` | 1-5 | Module name too narrow (holds full EDN char vocabulary) | Rename to `vocab.rs` or `chars.rs` |
| 2 | intueri | `lexer.rs` | 249 | `e` variable name in `process_escape` | Rename to `escape_byte` |
| 3 | intueri | `lexer.rs` | 271 | `acc` in `read_hex4` | Rename to `codepoint` or `hex_accum` |
| 4 | intueri | `lexer.rs` | 375 | `owned: Option<String>` names allocation strategy not intent | Rename to `decoded_body` |
| 5 | intueri | `lexer.rs` | 647 | `decode_utf8_char` placed below `#[cfg(test)]` block | Move above `#[cfg(test)]` |
| 6 | intueri | `value.rs` | 451 (also solvere) | `write_keyword_segment` name + algorithm dup with `writer.rs:177` | Extract shared helper |
| 7 | intueri | `value.rs` | 503 | Doubled `// ─── Convenience accessors ──` banner | Remove inner |
| 8 | intueri | `lib.rs` | 191-193 | Arc-provenance in public `new_uuid_v4` doc | Move to internal comment |
| 9 | intueri | `json.rs` | 197 | `sentinel` name requires module-doc context | Soft — rename or accept |
| 10 | solvere | `value.rs` | 451-469 (also intueri) | Encoding walk duplicated — `write_keyword_segment` vs `writer.rs:177` `write_keyword_body` | Extract `write_keyword_body_to<W: Write>` in escapes.rs (or vocab.rs after rename) |
| 11 | struere | `writer.rs` | 106-125 | Pretty-print map asymmetry — first key fused to `{`; subsequent keys indented | Emit `\n` + indent before EVERY entry, or document + test the intentional asymmetry |
| 12 | struere | `writer.rs` | 162-170 (json wrapper) | `to_json_string` `.expect()` on serde_json call; panic site undocumented | Add `// rune:struere(invariant-coupling)` with reason citing closed `edn_to_json` construction |
| 13 | struere | `json.rs` | 279-294 | `parse_map_key` silently swallows EDN parse failures via `if let Ok(v) = parse(k)` | Either return `JsonError::InvalidMap` strict, or document + test silent fallback |
| 14 | struere | `parser.rs` | 158 | All four closer/EOF variants reported as `UnexpectedEof` | Split arm — `Eof` stays; `RParen`/`RBracket`/`RBrace` get `UnexpectedByte` or `UnexpectedToken` |
| 15 | struere | `lexer.rs` | 213 | `String::with_capacity(self.input.len() - body_start)` over-allocates | Use `self.pos - body_start` or `min(remaining, 64)` floor |
| 16 | purgare | `writer.rs` | 215 | `write_to` re-export with zero external callers | Add `// rune:purgare(public-api)` — buffer-reuse API for downstream consumers |
| 17 | purgare | `json.rs` | 167 | `to_json_string_pretty` exported with zero callers | Rune `public-api` or drop re-export |
| 18 | purgare | `json.rs` | 93 | `edn_to_json` exported with only recursive callers | Rune `public-api` or drop re-export |
| 19 | purgare | `json.rs` | 175 | `json_to_edn` exported with only recursive callers | Rune `public-api` or drop re-export |
| 20 | purgare | `json.rs` | 49 | `JsonError` exported, zero external consumers | Rune `public-api` |
| 21 | purgare | `json.rs` | 88 | `JsonResult` exported, zero external consumers | Rune `public-api` |
| 22 | purgare | `lib.rs` | 150 | `parse_wire_owned` exported, zero callers | Rune `future-fixture` — wire-mode OwnedValue variant; arc 217 consumer expected |
| 23 | temperare | `parser.rs` | 382, 391 | Second linear scan of identifier suffix via `name.contains('/')` after `body.find('/')` | Fold into one pass via `splitn(3, '/')` |
| 24 | cernere | `parser.rs` | 447-462 `is_canonical_uuid` | Accepts uppercase hex despite "strict canonical" doc claim | Enforce `is_ascii_lowercase \|\| is_ascii_digit` for non-hyphen positions |
| 25 | cernere | `json.rs` | 375 `decode_uuid` | Skips `is_canonical_uuid` check on JSON bridge path | Apply same canonical-strict check; consistency with EDN path |
| 26 | cernere | docs | USER-GUIDE.md:233,294 | Map output format claim wrong — says `, ` separator; writer emits ` ` (space-only) | Fix doc claim to "single space separator"; update example assertion |
| (note) | cernere | docs | USER-GUIDE.md | `parse_wire` / `parse_wire_owned` (real public functions) not documented | Add to USER-GUIDE — wire-mode parser variant |

---

## Convergence-with-substrate observation

sequi CONVERGED — zero L1 + zero L2 — confirms wat-edn's core architectural discipline is intact. Every transformation chain (parse → tokens → values; values → strings; EDN ↔ JSON) threads state visibly through signatures. The `wire_decode: bool` is set at construction and never mutated; the lexer cursor lives on `&mut self`; the writer buffer is an explicit `&mut String` at every recursive call. No hidden coordination.

This is the substrate's discipline holding under audit. The findings above are all polish-level; the foundation is sound.

## Practitioner's standing assessment confirmed

"Near impeccable" was honest. The arc 218 closure work makes it fully impeccable; arc 217 (Clojure-IPC bridge) then builds on a clean foundation.

*The full guard stood. The pieces guard each. The whole guarded everything.*
