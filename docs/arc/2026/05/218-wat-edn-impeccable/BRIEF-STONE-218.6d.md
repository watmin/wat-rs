# BRIEF — Arc 218 Stone 218.6d — L2 sweep (13 mechanical fixes; no rune candidates)

**Stone scope (sonnet portion):** 13 L2 findings from VIGILIA-REPORT-2026-05-22-CHECKPOINT.md. Per user direction (runes require significant justification), NONE of the L2 findings warrant runes — all get root-fix or rename or extract. After this stone ships, vigilia recasts on `crates/wat-edn/{src,tests}/` for the IMPECCABLE proof.
**Type:** Sonnet Mode A.
**Time budget:** 25-40 min target; 60 min STOP.
**Depends on:** Stone 218.6c (`605565f` — 12/12 PASS; all 9 L1 closed).
**Calibration:** 9 stones in series below lower band. This one is largest scope yet (13 items + 1 helper extract); band 25-40.
**Unblocks:** Vigilia recast on `crates/wat-edn/{src,tests}/` — if CONVERGED across all 7 spells (0 L1 + 0 L2), arc 218 closure conversation opens.

## Pre-flight verified (orchestrator-grep'd 2026-05-22, post-218.6c)

### Section A — solvere (1 item)

**A.1 `vocab::split_namespaced` extract — 3 sites in `crates/wat-edn/src/json.rs`**

Three sites duplicate the namespace-slash-split idiom:
- `json.rs:258-262` — `string_to_edn` (keyword decode path)
- `json.rs:379-384` — `decode_symbol`
- `json.rs:424-430` — `decode_tagged`

Each does `find('/')`, split into prefix/name, validate, construct. The canonical `parse_namespaced` exists at `parser.rs:384-409` but is private. Per vigilia: extract `pub(crate) fn split_namespaced(body: &str) -> Option<(&str, &str)>` in `vocab.rs` (alongside `validate_first_char` + `translate_and_validate_ns` + `is_canonical_uuid`); call it from all 3 json.rs sites + the parser if cleaner.

### Section B — intueri (6 items)

**B.2 `sentinel` → `single_key_object` rename — `json.rs:222`**

`fn sentinel(key: &str, body: JV) -> JV` — function builds a one-key JSON object wrapper; "sentinel" is wire-convention jargon that doesn't say what the function does. Rename to `single_key_object` (or `wrap_single_key`); update all call sites within json.rs.

**B.3 `parse_map_key` → `decode_map_key` rename — `json.rs:304`**

The function does more than parse: it classifies EDN-or-string before parsing. The current name loses the classification. Rename to `decode_map_key`; update call sites.

**B.4 `open_pos` → `quote_start` rename — `lexer.rs:212`**

`fn lex_string_escaped(&mut self, open_pos: usize, body_start: usize)` — parameter `open_pos` is the position of the opening `"`, only used for the `UnclosedString` error span. Rename to `quote_start` (or `open_quote_pos`); update the function body + the 1 call site at `lexer.rs:206`.

**B.5 `is_scalar` → `is_inline_value` rename + Tagged WHY comment — `writer.rs:47`**

The function name is borrowed from the broader HDC domain; within the pretty-printer the intent is "does this value inline without breaking?" Rename to `is_inline_value`. Also add a WHY comment explaining `Value::Tagged` is absent because the tagged variant has a dedicated arm in `write_pretty_indented` (lines 127-136). Update all callers within writer.rs (likely 1-2 sites in `all_scalar` + `write_pretty_indented`).

**B.6 `parse_value` / `parse_value_inner` restructure — `parser.rs:98-99` + `:107`**

Current: `parse_value()` (public-ish wrapper) calls `parse_value_inner(false)`; `_inner` suggests spatial nesting but the actual difference is the `discarding: bool` parameter. Two clean options (sonnet picks):
- **Option α (rename)**: `parse_value_inner` → `parse_value_discarding`; the `_discarding` suffix names what differs
- **Option β (inline)**: delete `parse_value` wrapper; surface `discarding: bool` at the 1 internal call site (or whatever count grep shows)

Pick the option that reads cleaner; preserve the docstring on `parse_value_inner` (or move to whichever fn carries the contract).

**B.7 tests/comprehensive.rs compressed write+parse — 3 sites at lines 1063, 1218-1219, 1224-1225**

Each site compresses `let s = write(&v1); let v2 = parse(&s).unwrap();` onto one line. The file already has a `round_trip` helper at line 1124. Replace the 3 compressed lines with the helper call.

### Section C — temperare (2 items)

**C.8 `all_variants()` `LazyLock` — `tests/accessors.rs:13`**

`all_variants()` is called 18× across the file (lines 40, 41, 47, 54, 66, 78, 90, 102, ...); each call constructs `Vec<17 (label, Value)>` with 2 `Box<BigInt>` + `Box<BigDecimal>` heap allocs per Value. 36 Box allocs per test binary run. Wrap in `std::sync::LazyLock<Vec<(&'static str, Value<'static>)>>` initialized once; call sites take `&'static` slice.

Recommended shape:

```rust
use std::sync::LazyLock;

static ALL_VARIANTS: LazyLock<Vec<(&'static str, Value<'static>)>> =
    LazyLock::new(|| vec![ /* existing 17 entries */ ]);

// Call sites: replace all_variants() with &*ALL_VARIANTS
```

Verify `Value<'static>` works as the slice element type (it's `OwnedValue` aliased per `lib.rs`). If `Clone` is needed per call site, add `.clone()` or iterate with `&`.

**C.9 tests/wire_encoding.rs double-write — 5 sites at lines 263, 271, 281, 231, 249**

Each site: `let wire = write(&k); assert_eq!(wire, expected); roundtrip_wire(&k);` where `roundtrip_wire` (defined at line 40) ALSO calls `write(&k)` internally. The second `write` allocation is wasted.

Two clean options (sonnet picks):
- **Option α** — change `roundtrip_wire` signature to accept the already-bound wire string: `roundtrip_wire_str(wire: &str, original: &Value<'_>)`; at each of the 5 sites bind wire once, pass both
- **Option β** — keep `roundtrip_wire(&k)` calling `write` internally, but inline the assertion: `let wire = write(&k); assert_eq!(wire, expected); let parsed = Parser::new_wire(&wire).parse_top().unwrap(); assert_eq!(*k, parsed);`

### Section D — struere (2 items)

**D.10 lexer.rs:309 double-peek in `lex_char`**

`lex_char` peeks at `self.pos` at lines 296-303 (None/whitespace check) then again at line 309 (`let first = self.peek().unwrap()`). The intervening branch is just `_ => {}` (no advance). Capture `first` at the first peek; pass it through:

```rust
// At line 296 area, instead of just checking, bind:
let first = match self.peek() {
    None => return Err(Error::at(start, ErrorKind::InvalidChar("empty".into()))),
    Some(b) if is_whitespace(b) => return Err(Error::at(start, ErrorKind::InvalidChar("backslash followed by whitespace".into()))),
    Some(b) => b,
};
// Then at line 309, use `first` directly instead of self.peek().unwrap()
```

Honest delta accepted: a cleaner refactor that eliminates the double-peek; sonnet picks shape.

**D.11 parser.rs:111 `pos` capture invariant comment**

`let pos = self.lexer.pos()` is captured after `skip_discards()` but before `next_token()`. The keyword case handles position via lexer-captured `body_start` (well-documented at lines 139-141). Other peeked token paths silently inherit a stale pos. Add a comment at line 111 surfacing this:

```rust
// pos points at the start of the next unconsumed input byte. NOTE: for
// peeked tokens (where `self.peeked.is_some()`), pos may lag by one
// token — the position reflects the BYTE we're about to read, not the
// token we're about to consume. The Keyword arm captures its own
// body_start from the lexer to get exact spans; other arms accept this
// invariant trade-off for diagnostics (callers needing exact spans
// should capture at lex time, see Keyword handling at lines 139-141).
let pos = self.lexer.pos();
```

Exact wording is sonnet's call; preserve the surfaced invariant.

### Section E — cernere (2 items)

**E.12 USER-GUIDE.md:383 JSON i64 range row**

Current: `i64 (> 2^53)       string  "9007199254740993"`. The code uses `SAFE_INT_MIN..=SAFE_INT_MAX` (± 2^53−1); negatives below `-(2^53−1)` also serialize as strings. Update to `i64 (out of ±2^53 range)` or `|i64| > 2^53`. Adjust the example to be range-honest (e.g., add a second example for negative overflow, or pick wording that covers both).

**E.13 USER-GUIDE.md:740 aspirational serde claim**

Current text (around line 740): `serde` integration (`Serialize/Deserialize` for `Value`) presented as "available behind no flag yet; v0.2 candidate." No serde feature in Cargo.toml; no serde impl in the codebase. Rephrase as a future consideration ("A future v0.2 may add direct `serde::{Serialize, Deserialize}` impls on `Value`...") or remove entirely. Sonnet picks; preserve the user-facing tone of the section.

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`
- Per user direction 2026-05-22: NO new runes — all 13 L2s get root-fixes
- The 4 existing runes (2 temperare + 2 purgare) stay intact

## Your scope (sonnet)

Execute Sections A-E. Sections are independent and can be ordered for cleanest commit hygiene (e.g., do all renames first, then test updates, then doc updates).

### Section ordering guidance

1. **Renames first** (B.2 + B.3 + B.4 + B.5 + B.6) — single-file scope; verify compiles + tests after each
2. **Substrate edits** (A.1 + D.10 + D.11) — multi-file; A.1 is the largest mechanical change (helper extract + 3-site update)
3. **Test updates** (B.7 + C.8 + C.9) — tests/ scope
4. **Doc updates** (E.12 + E.13) — USER-GUIDE only

### Verification (must run before SCORE)

1. `cargo build --release -p wat-edn` — 0 warnings
2. `cargo test --release -p wat-edn` — expected 344 (no count change; all fixes correctness-preserving)
3. `cargo test --release --lib -p wat` — 824/0 PASS
4. `cargo clippy --release --all-targets -p wat-edn -- -D warnings` — 0 warnings
5. From `crates/wat-edn/interop-tests/`:
   - `cargo build --release` — 0 warnings
   - `cargo clippy --release --all-targets -- -D warnings` — 0 warnings
6. **Interop-tests 4 handshakes** (mandatory per `feedback_wat_edn_touch_runs_interop_tests`):
   - `cd crates/wat-edn/interop-tests`
   - `cargo run --release --bin wat-edn-interop-tests | clojure -M clj/consume.clj`
   - `clojure -M clj/produce.clj | cargo run --release --bin reader`
   - `cargo run --release --bin shape_matrix | clojure -M clj/consume_shapes.clj`
   - `clojure -M clj/produce_shapes.clj | cargo run --release --bin shape_matrix_reader`

**NOTE on handshakes:** if sub-agent permissions deny the piped form, ship the rest cleanly + write SCORE marking row 14 as "pending orchestrator-side verification". Orchestrator runs them during independent scoring per 218.6b/218.6c precedent. Do NOT block.

## STOP triggers

- **STOP-1 (A.1 surfaces unexpected slash-split-pattern consumer):** vigilia named 3 sites; if a 4th surfaces, report.
- **STOP-2 (rename cascades into wat downstream or interop-tests):** `sentinel` / `parse_map_key` / `is_scalar` / `parse_value_inner` are all internal — no external consumers expected. If grep finds any, report + STOP.
- **STOP-3 (LazyLock signature mismatch):** if `Value<'static>` doesn't work as the slice element type (lifetime issues), the path forward is `OwnedValue` or `Arc<Value>` per sonnet's judgment.
- **STOP-4 (wire_encoding restructure breaks an existing assertion):** the 5 sites bind to a specific wire string; ensure the refactor preserves the assertion shape.
- **STOP-5 (USER-GUIDE doc-test affected):** if the doc has any tested code blocks, ensure the rephrased wording doesn't break them.
- **STOP-6 (60 min elapsed):** wall-clock STOP.

## Out-of-scope

- INSCRIPTION + arc 218 closure — deferred per user direction
- New runes — discipline is no new runes for this L2 sweep; all 13 items get root-fixes
- Touching the 4 existing runes (2 temperare + 2 purgare) — they stay intact
- Encoding doctrine / wat-edn syntax changes
- New public surface beyond what already exists
