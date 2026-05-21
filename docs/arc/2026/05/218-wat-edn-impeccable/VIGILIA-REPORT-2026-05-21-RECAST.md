# Vigilia Re-cast Report — wat-edn — 2026-05-21 (post-arcs 218.1-4 + 219)

**Cast:** 7 defensive spells in parallel against `crates/wat-edn/src/` — all 8 source files.

**Practitioner's pre-cast assessment:** "we think it's clean?"

**Verdict:** **DIVERGES (7 L1 + 26 L2)**. The substrate-as-teacher cascade fires honestly: arcs 218.1-218.4 + 219 + arc 216 antidote shipped, fixing 28 findings; recast surfaces 33 new ones at deeper layers (and via larger surface area — `is_canonical_uuid` added in 218.4; `translate_wat_to_strict` added in 219.1).

**Baseline comparison (2026-05-21 pre-arc-218):** 2 L1 + 26 L2 → 7 L1 + 26 L2. **L1 count went up by 5** because shipped stones EXPOSED new surface; substrate-as-teacher pattern holds.

---

## Per-spell summary

| Spell | Verdict | L1 | L2 |
|---|---|---|---|
| **sequi** | CONVERGED ✓ | 0 | 0 |
| struere | DIVERGES (polish) | 0 | 4 |
| solvere | DIVERGES (placement) | 2 | 3 |
| cernere | DIVERGES (substrate bug) | 1 | 3 |
| temperare | DIVERGES (perf + trade-off) | 2 | 4 |
| intueri | DIVERGES (naming lie) | 1 | 3 |
| purgare | DIVERGES (rune annotations) | 1 | 9 |
| **TOTAL** | DIVERGES | **7 L1** | **26 L2** |

**sequi CONVERGED** — substrate-level state-threading discipline impeccable (same as 2026-05-21 baseline). Every transformation chain threads state visibly through signatures; no globals/statics/Mutex carrying domain state. Constructor translations (added in 219.1) preserve state-threading; from_parts_unchecked unchanged; recursive walks honest.

---

## L1 — Real findings (must address before IMPECCABLE)

### L1-CERN — Supplementary-plane `Value::Char` overflows `\uXXXX` (REAL SUBSTRATE BUG)

**`writer.rs:313-314`** — for `Value::Char` with codepoint > U+FFFF (e.g. emoji `U+1F600`):

```rust
if (c as u32) < 0x20 || (c as u32) > 0x7E {
    write!(out, "u{:04X}", c as u32).unwrap();  // ← format width is MIN, not MAX
```

For codepoint `0x1F600` (😀), `{:04X}` emits **5 hex digits** (`ὠ0`). EDN spec requires `\uXXXX` to be **exactly 4 hex digits** (BMP only). Round-trip fails:
- wat-edn writes `ὠ0`
- `clojure.edn/read` rejects (not 4 digits)
- wat-edn's own lexer also rejects (`lexer.rs:336` checks `body_str.len() == 5 && starts_with('u')`)

**Latent hole in arc 219's empirical proof.** The shape matrix (23/23 PASS) didn't include supplementary-plane chars. The "strict-EDN compliant" claim is FALSE for `Char(U+10000..)`.

**Fix direction:** emit non-control supplementary-plane chars as literal Unicode (`out.push(c)`); only emit `\uXXXX` for BMP control chars (which fit in 4 digits). Add probe to shape matrix.

---

### L1-INT — `decode_set` uses wrong `JsonError` variant (naming lie at type level)

**`json.rs:376`** `decode_set` uses `JsonError::InvalidMap` for a `#set` body violation:

```rust
let arr = v.as_array().ok_or_else(|| JsonError::InvalidMap(format!("#set body must be array: {}", v)))?;
```

Function is `decode_set`; error variant says `InvalidMap`; diagnostic says `#set body`. All three contradict. Caller matching on `JsonError` to distinguish bad set from bad map gets wrong arm.

**Fix direction:** add `JsonError::InvalidSet(String)` variant; use it in `decode_set`.

---

### L1-TEMP-1 — `all_scalar` / `len() <= 8` operand order (one-char perf fix)

**`writer.rs:78`**:

```rust
} else if all_scalar(items) && items.len() <= 8 {
```

`all_scalar` walks all N items; `len() <= 8` is O(1). Wrong order — for collections > 8, the scan is wasted.

**Fix direction:** swap operands → `items.len() <= 8 && all_scalar(items)`. Single-char fix.

---

### L1-TEMP-2 — `to_json_string` double materialization (acknowledged trade-off)

**`json.rs:172`** + **`:182`**:

```rust
serde_json::to_string(&edn_to_json(v))
serde_json::to_string_pretty(&edn_to_json(v))
```

Both build full `serde_json::Value` tree, then serialize. The intermediate JV tree is pure allocation overhead. Genuine trade-off — alternative requires custom `serde::Serialize` impl for `Value`.

**Disposition:** rune as `// rune:temperare(simplicity-win)` citing serde_json API choice OR defer to future arc when JSON throughput becomes bottleneck. Not deletion-urgent.

---

### L1-SOL-A — `is_canonical_uuid` placement (recent self-inflicted from 218.4)

**`parser.rs:455`** — pure UUID spec predicate, consumed by `json.rs:36` via `use crate::parser::is_canonical_uuid`. The `pub(crate)` cross-module visibility IS the tell — JSON bridge depends on parser's "internals" when actual dependency is "both depend on UUID spec rule."

Stone 218.4 picked Option A (`pub(crate)` in parser.rs) for minimal disturbance over vocab.rs move. Solvere disagrees with the placement.

**Fix direction:** move `is_canonical_uuid` to `vocab.rs`; both callers import from vocab; remove cross-module pub(crate).

---

### L1-SOL-B — `translate_wat_to_strict` placement (recent self-inflicted from 219.1)

**`value.rs:218-220`** — wat↔EDN namespace encoding rule. Called 6 times in constructors paired with `vocab::validate_first_char`. Stone 219.1 placed it in value.rs (Option β); solvere says it's a vocab concern (same level as `is_symbol_continue` / `validate_first_char`).

**Fix direction:** move `translate_wat_to_strict` to `vocab.rs` as `pub(crate)`. Better: introduce `vocab::translate_and_validate_ns(ns: &str) -> Result<String, &'static str>` which chains translate → validate, eliminating paired-call duplication at all 6 sites.

---

### L1-PURG — `parse_wire` / `parse_wire_owned` zero external callers

**`lib.rs:144-152`** — two public free functions exist (parse_wire / parse_wire_owned). Stone 218.4 documented them in USER-GUIDE §3. Zero callers outside `crates/wat-edn/tests/wire_encoding.rs`. The substrate writes via `wat_edn::write` and reads via `wat_edn::parse_owned` everywhere in production.

**Disposition:** either retire (no consumer materialized) OR `// rune:purgare(future-fixture) — wire-mode decode entry point; held for migration when stdio pipe switches from parse_owned to parse_wire for parametric keyword round-trips`.

---

## L2 — Polish-level findings (26 total)

### struere (4)

1. `writer.rs:68-73` — `write_pretty_indented` outer/inner match coupling with `unreachable!()`; extract `collection_brackets()` helper
2. `writer.rs:45-58` — `is_scalar` missing `BigInt`/`BigDec` (silent contract gap; comment says "scalar enough to inline")
3. `json.rs:238-257` + `:360-371` — `string_to_edn` + `decode_symbol` duplicate `find('/')` split (echoes cross-spell duplication pattern); extract `split_ns()` helper to vocab
4. `parser.rs:97-99` — `parse_value` wrapper + `discarding: bool` flag is wrong-level abstraction

### solvere (3)

1. Depth-tracking triplicated — `vocab.rs:159-170` write side (218.1 extracted) + `lexer.rs:377-409` read side (load-bearing inline w/ allocation) + `parser.rs:427-443` `reject_underscore_in_brackets` (pure walk, could extract); move parser's to vocab; rune lexer's with load-bearing-coupling
2. Display vs write_keyword dual path at `value.rs:453-467` + `writer.rs:161-168` — irreducible-tangle; needs formal rune
3. `json.rs:138` encode side unnamed (asymmetric with `parse_map_key`); extract `encode_map_key()` for symmetry

### cernere (3)

1. USER-GUIDE.md:818 § ErrorKind table omits `UnexpectedToken` (added 218.3) + `Utf8` variants
2. USER-GUIDE §6 + README don't document `new_uuid_v5` (arc 206 slice 1.5)
3. Writer emits uppercase `\uXXXX` but Clojure emits lowercase; case-mismatch (Clojure accepts both; conformance grey area)

### temperare (4)

1. `parser.rs:426` + `lexer.rs:377` — duplicate depth-scan over keyword body (echoes solvere L2-A); lexer could set flag on Keyword token; parser checks flag instead of re-scanning
2. `value.rs:102` + `:125` — `map_eq`/`set_eq` allocate `Vec<bool>` per equality check; bitmask for n ≤ 8 would avoid heap
3. `value.rs` — translate + validate + CompactString::from = double allocation; minor
4. `writer.rs:38` — `push_indent` loop calls `out.push_str(INDENT)` once per level; minor

### intueri (3)

1. `json.rs:42` — `JV` alias too cryptic; use `JsonValue`
2. `parser.rs:384` — `first` variable means two things in different branches (bare name vs namespace prefix)
3. `writer.rs:267` — `chunk_end` vs `end_clean` synonymous-sounding; rename to `memchr_stop` + `safe_end`

### purgare (9)

Mostly **public-API forward-declarations** — items intentionally exported for downstream library consumers but with no current external callers:

1. `value.rs:506` `Value::as_bool` — only `src/lower.rs:380` (test) calls externally
2. `value.rs:534` `Value::as_char` — only `crates/wat-edn/tests/accessors.rs:103`
3. `value.rs:555` `Value::as_list` — only tests
4. `value.rs:576` `Value::as_set` — only tests
5. `value.rs:604` `Value::is_nil` — only tests
6. `value.rs:590` `Value::as_inst` — interop-test + accessors test
7. `lib.rs:84-87` — `json_to_edn`, `edn_to_json`, `from_json_string`, `to_json_string_pretty`, `JsonError`, `JsonResult` re-exports — zero external callers (within-crate tests exercise them); only `to_json_string` is live via `src/edn_shim.rs`
8. `vocab.rs` public items — `NAMED_CHARS`, `name_to_char`, `char_to_name`, `decode_string_escape`, `encode_string_escape`, `is_symbol_start`, `is_symbol_continue`, `is_whitespace`, `hex_value`, `validate_first_char` — all `pub` with no external callers; should be `pub(crate)` (honest visibility)
9. `lib.rs:90` `write_to` re-export — buffer-reuse ergonomic; zero external callers

**Disposition:** `// rune:purgare(public-api)` annotations for the library surface items; `pub(crate)` tightening for vocab.rs implementation primitives.

---

## What this means for arc 218 closure

**Arc 218.5 (re-cast vigilia + INSCRIPTION + arc closure) CANNOT honestly close on this aggregate.** Per `feedback_any_defect_catastrophic` — substrate trust is binary; >0 defects = 0 trust. 7 L1 lies + 26 L2 mumbles means IMPECCABLE is not yet achieved.

**Proposed disposition:**

- **Stone 218.6** — absorb the L1 substrate fixes:
  - (a) cernere L1 — writer.rs supplementary-plane char fix + shape matrix probe added
  - (b) intueri L1 — `JsonError::InvalidSet` variant + decode_set fix
  - (c) temperare L1-1 — operand swap (one-char)
  - (d) solvere L1-A — move `is_canonical_uuid` parser.rs → vocab.rs
  - (e) solvere L1-B — move `translate_wat_to_strict` value.rs → vocab.rs (or combine into `translate_and_validate_ns`)
  - (f) temperare L1-2 — rune as simplicity-win OR open future-perf-arc
  - (g) purgare L1 — rune `parse_wire`/`parse_wire_owned` as future-fixture OR retire

- **Stone 218.7** — L2 sweep:
  - All `purgare` public-api runes + `vocab.rs` pub→pub(crate) tightening
  - USER-GUIDE additions (ErrorKind variants + new_uuid_v5)
  - Solvere/struere extract helpers (collection_brackets, split_ns, encode_map_key)
  - Intueri renames (JV → JsonValue; first → prefix_or_name; chunk_end/end_clean → memchr_stop/safe_end)
  - Other L2 polish

- **Stone 218.5** (redefined) — re-cast vigilia AGAIN on post-218.6+218.7 substrate; if CONVERGED, INSCRIPTION ships and arc 218 closes IMPECCABLE.

---

## Substrate-as-teacher meta-finding

The recast L1 count went from 2 (pre-arc-218) to 7 (post-arc-218 + 219). This is HONEST — the shipped work expanded substrate surface (UUID strictness added is_canonical_uuid; arc 219 added translate_wat_to_strict; more code = more places for L1s to live). Closing 28 findings exposed 33 new ones at deeper layers.

The vigilia cast IS the discipline that demands the closure. arc 218 IMPECCABLE means CONVERGED across all spells. We're not there yet; the work continues.

*The full guard stood. The pieces guard each. The whole did not yet guard everything.*
