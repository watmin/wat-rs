# BRIEF — Arc 218 Stone 218.6 — L1 substrate fixes (6 fixes + 1 retire + 1 rune)

**Stone scope (sonnet portion):** seven items from VIGILIA-REPORT-2026-05-21-RECAST.md L1 disposition. Six substrate fixes (real bug + naming lie + perf swap + two placement moves + helper combiner) + one targeted retirement (zero-caller public free-function pair) + one strongly-justified perf rune (additive on existing rune).
**Type:** Sonnet Mode A.
**Time budget:** 30-45 min target; 60 min STOP.
**Depends on:** Stone 218.4 (`f17a9e0` area — UUID strictness shipped 9/9) + Arc 219 (`331cfb9` — strict-EDN keywords) + arc 216 antidote (`Value: Hash + Eq` canonical).
**Calibration:** 218.1 ~20 (band 25-45); 218.2 ~15 (band 30-50); 218.3 ~25 (band 40-65); 218.4 ~20 (band 20-40); 219.1 below band. Pattern locked: substrate-pre-grep + locked-decisions + mechanical = at or below lower band. This stone is heavier than 218.4 (7 items spanning 5 source files + interop probe + 23-test migration) — band raised to 30-45.
**Unblocks:** Stone 218.7 (L2 sweep) → Stone 218.5 redefined (re-cast vigilia + INSCRIPTION + arc closure) → arc 217 (Clojure-IPC bridge) → arc 216.8/.9/.10.

## Pre-flight verified (orchestrator-grep'd 2026-05-21, post-219.1 + post-recast)

### Item (a) — `Value::Char` supplementary-plane `\uXXXX` overflow (REAL BUG)

`crates/wat-edn/src/writer.rs:307-318` `write_char`:

```rust
fn write_char(c: char, out: &mut String) {
    out.push('\\');
    if let Some(name) = char_to_name(c) {
        out.push_str(name);
        return;
    }
    if (c as u32) < 0x20 || (c as u32) > 0x7E {
        write!(out, "u{:04X}", c as u32).unwrap();   // ← overflow for U+10000+
        return;
    }
    out.push(c);
}
```

For codepoint `0x1F600` (😀), `{:04X}` emits **5 hex digits** (`u1F600`). EDN spec demands exactly 4 hex digits (BMP only). Round-trip fails: writer emits 5-digit form; `lexer.rs:336` rejects (`body_str.len() == 5 && starts_with('u')`); `clojure.edn/read` rejects. Latent hole in arc 219's empirical proof — shape matrix didn't include supplementary-plane chars.

### Item (b) — `decode_set` uses wrong `JsonError` variant (naming lie at type level)

`crates/wat-edn/src/json.rs:373-379`:

```rust
fn decode_set(v: &JV) -> JsonResult<OwnedValue> {
    let arr = v.as_array()
        .ok_or_else(|| JsonError::InvalidMap(format!("#set body must be array: {}", v)))?;
    ...
}
```

Function is `decode_set`; variant says `InvalidMap`; diagnostic says `#set body`. Three contradicting facts at the type level. `JsonError` enum lives at `crates/wat-edn/src/json.rs:51`.

### Item (c) — `all_scalar` / `len() <= 8` operand order (one-char perf fix)

`crates/wat-edn/src/writer.rs:78`:

```rust
} else if all_scalar(items) && items.len() <= 8 {
```

`all_scalar` (defined writer.rs:62) walks all N items; `len() <= 8` is O(1). Wrong short-circuit order.

### Item (d) — `is_canonical_uuid` placement (recent self-inflicted from 218.4)

`crates/wat-edn/src/parser.rs:455` (definition, `pub(crate)`) → consumed by:
- `parser.rs:297` (in-module call)
- `json.rs:36` (`use crate::parser::is_canonical_uuid;`)
- `json.rs:395` (call site)

The cross-module `pub(crate)` IS the tell. UUID spec rule belongs in `vocab.rs` (same level as `validate_first_char` at vocab.rs:184).

### Item (e) — `translate_wat_to_strict` placement (recent self-inflicted from 219.1)

`crates/wat-edn/src/value.rs:218-220`:

```rust
fn translate_wat_to_strict(ns: &str) -> String {
    ns.replace("::", ".")
}
```

Called at 6 sites in value.rs (lines 246, 275, 323, 350, 389, 411), each paired with `vocab::validate_first_char(&ns_translated)`. The paired-call pattern repeats 6 times. Combining into a single vocab helper `translate_and_validate_ns(ns: &str) -> Result<String, &'static str>` (translate then validate; return the translated form) eliminates the duplication at all 6 sites.

### Item (f) — `to_json_string` / `to_json_string_pretty` double materialization

`crates/wat-edn/src/json.rs:165-184`:

```rust
pub fn to_json_string(v: &Value<'_>) -> String {
    // rune:struere(invariant-coupling) — serde_json::to_string cannot fail ...
    serde_json::to_string(&edn_to_json(v)).expect("serde_json::to_string on Value")
}

pub fn to_json_string_pretty(v: &Value<'_>) -> String {
    // rune:struere(invariant-coupling) — serde_json::to_string_pretty ...
    serde_json::to_string_pretty(&edn_to_json(v))
        .expect("serde_json::to_string_pretty on Value")
}
```

Both build full `serde_json::Value` tree then serialize. The intermediate `JV` tree is pure allocation overhead. Genuine trade-off — alternative requires custom `serde::Serialize` impl for `Value` (full visitor implementation). No caller has surfaced JSON-throughput pressure.

**Strong-rune justification:** `serde_json::to_string` takes `&impl Serialize`. The double-materialization is the SHAPE serde-json's API forces unless we write a custom `Serialize` impl. Naming the trade-off as `rune:temperare(serde-api-shape)` (additive to the existing `struere(invariant-coupling)` rune) prevents a future-self "optimization" that doesn't measure first.

### Item (g) — `parse_wire` / `parse_wire_owned` retire (zero-caller surface)

`crates/wat-edn/src/lib.rs:144-152`:

```rust
pub fn parse_wire(input: &str) -> Result<Value<'_>> {
    Parser::new_wire(input).parse_top()
}

pub fn parse_wire_owned(input: &str) -> Result<OwnedValue> {
    parse_wire(input).map(Value::into_owned)
}
```

Both are one-line convenience wrappers around `Parser::new_wire`. Added in arc 170 slice 1f area, documented in USER-GUIDE in Stone 218.4.

**Caller audit (production):** zero. `tests/wire_encoding.rs` is the only file referencing them. Per `feedback_verbose_is_honest` + user-direction 2026-05-21 (strong-rune only): retire the public surface; the underlying capability (`Parser::new_wire`) stays. If arc 217 (Clojure-IPC bridge) surfaces a real consumer, the substrate mints them back with a real caller pinning the necessity.

`tests/wire_encoding.rs` test count: 23. Migration path: each `parse_wire(x)` → `Parser::new_wire(x).parse_top()`; each `parse_wire_owned(x)` → `Parser::new_wire(x).parse_top().map(Value::into_owned)`. All 23 tests preserved (the wire-MODE feature is what the tests verify; the wrapper is incidental).

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`
- Substrate trust binary per `feedback_any_defect_catastrophic` — every L1 must land cleanly

## Your scope (sonnet)

### Part (a) — supplementary-plane `Char` fix + shape matrix probe

1. **At `crates/wat-edn/src/writer.rs:307-318`**, rewrite `write_char` so supplementary-plane codepoints (`> 0xFFFF`) emit as literal Unicode (`out.push(c)`) instead of overflowing `\uXXXX`. Recommended shape (sonnet may pick a cleaner form):

   ```rust
   fn write_char(c: char, out: &mut String) {
       out.push('\\');
       if let Some(name) = char_to_name(c) {
           out.push_str(name);
           return;
       }
       let cp = c as u32;
       // BMP control bytes + DEL → \uXXXX (exactly 4 hex digits per spec).
       if cp < 0x20 || cp == 0x7F {
           write!(out, "u{:04X}", cp).unwrap();
           return;
       }
       // BMP non-control non-printable still fits in 4 digits.
       if cp <= 0xFFFF && !(0x20..=0x7E).contains(&cp) {
           write!(out, "u{:04X}", cp).unwrap();
           return;
       }
       // Printable ASCII + supplementary-plane → literal. Supplementary
       // plane MUST be literal: EDN's \uXXXX is BMP-only (4 hex digits).
       out.push(c);
   }
   ```

   Honest delta accepted: a different correct shape that preserves BMP behavior AND fixes the supplementary-plane overflow.

2. **Verify no existing test regresses on BMP char output.** Quick `grep -n '\\\\u00\|\\\\u01\|\\\\u02' crates/wat-edn/` will surface any test asserting on BMP `\uXXXX` escape output. If any are affected, the recommended shape preserves them.

3. **Add probe for supplementary-plane round-trip.** A new test in `crates/wat-edn/tests/round_trip.rs` (or similar): write a `Value::Char('😀')` (U+1F600); assert the output parses back to the same value via `parse`. Also extend the interop-tests shape matrix:
   - `crates/wat-edn/interop-tests/src/bin/shape_matrix.rs` — add `:char-supplementary` shape: `Value::Char('😀')` or similar non-BMP char
   - `crates/wat-edn/interop-tests/clj/consume_shapes.clj` — assert the new shape parses to `\😀` (or equivalent literal Unicode char)
   - `crates/wat-edn/interop-tests/clj/produce_shapes.clj` — include `\😀` in the produced map
   - `crates/wat-edn/interop-tests/src/bin/shape_matrix_reader.rs` — assert from Clojure pr-str

### Part (b) — `JsonError::InvalidSet` variant

1. **At `crates/wat-edn/src/json.rs:51`** (the `JsonError` enum), add a new variant `InvalidSet(String)` in the natural alphabetical / topical position (next to `InvalidMap`).

2. **At `crates/wat-edn/src/json.rs:376`**, replace `JsonError::InvalidMap` with `JsonError::InvalidSet` in `decode_set`'s error path. Diagnostic text can stay.

3. Verify any `Display`/`Debug` impl on `JsonError` covers the new variant (rustc will tell you if not).

### Part (c) — `writer.rs:78` operand swap (one-char fix)

1. **At `crates/wat-edn/src/writer.rs:78`**, swap operands:
   ```rust
   } else if items.len() <= 8 && all_scalar(items) {
   ```
   Short-circuits the O(N) `all_scalar` walk for collections > 8.

### Part (d) — `is_canonical_uuid` parser.rs → vocab.rs

1. **At `crates/wat-edn/src/vocab.rs`**, add `is_canonical_uuid` near `validate_first_char` (around line 184+). Move the docstring + function body verbatim from `parser.rs:441-471`. Visibility `pub(crate)`.

2. **At `crates/wat-edn/src/parser.rs:455-471`**, delete the function. Replace the use site at `parser.rs:297` with `vocab::is_canonical_uuid` (or add `use crate::vocab::is_canonical_uuid;` near the existing `use crate::vocab::validate_first_char;` at line 8).

3. **At `crates/wat-edn/src/json.rs:36`**, change `use crate::parser::is_canonical_uuid;` to `use crate::vocab::is_canonical_uuid;`.

4. **At `crates/wat-edn/src/parser.rs:473-474`** (the comment pointing at vocab for `validate_first_char`), update or remove to reflect that `is_canonical_uuid` also now lives in vocab.

### Part (e) — `translate_wat_to_strict` value.rs → vocab.rs (combine into `translate_and_validate_ns`)

1. **At `crates/wat-edn/src/vocab.rs`**, add a new `pub(crate)` helper:

   ```rust
   /// Translate a wat-rs `::` namespace separator to strict-EDN `.`
   /// form and validate the first-character rule in one step. Returns
   /// the translated namespace on success.
   pub(crate) fn translate_and_validate_ns(ns: &str) -> Result<String, &'static str> {
       let translated = ns.replace("::", ".");
       validate_first_char(&translated)?;
       Ok(translated)
   }
   ```

2. **At `crates/wat-edn/src/value.rs:218-220`**, delete `translate_wat_to_strict`.

3. **At the 6 paired-call sites in value.rs** (lines 246-249, 275-277, 323-325, 350-352, 389-391, 411-413), replace each paired call:

   ```rust
   // BEFORE:
   let ns_translated = translate_wat_to_strict(namespace.as_ref());
   crate::vocab::validate_first_char(&ns_translated)
       .unwrap_or_else(|m| panic!(...));    // or ?  for try_* variants

   // AFTER (panic-flavored constructors `ns`):
   let ns_translated = crate::vocab::translate_and_validate_ns(namespace.as_ref())
       .unwrap_or_else(|m| panic!("invalid {namespace} {:?}: {}", namespace.as_ref(), m));

   // AFTER (try-flavored constructors `try_ns`):
   let ns_translated = crate::vocab::translate_and_validate_ns(namespace.as_ref())?;
   ```

   Panic-message wording stays consistent with existing diagnostics. Three constructor types × two flavors = 6 sites:
   - `Symbol::ns` (panic; value.rs:246) + `Symbol::try_ns` (?; value.rs:275)
   - `Keyword::ns` (panic; value.rs:323) + `Keyword::try_ns` (?; value.rs:350)
   - `Tag::ns` (panic; value.rs:389) + `Tag::try_ns` (?; value.rs:411)

### Part (f) — rune `temperare(serde-api-shape)` (additive on existing struere rune)

1. **At `crates/wat-edn/src/json.rs:165-172`** (`to_json_string`), append a SECOND rune line below the existing `struere(invariant-coupling)` comment block:

   ```rust
   pub fn to_json_string(v: &Value<'_>) -> String {
       // rune:struere(invariant-coupling) — serde_json::to_string cannot
       // fail here because edn_to_json's closed construction emits only
       // well-formed serde_json::Value graphs (no NaN-in-Number). The
       // .expect() panic is structurally unreachable; the coupling is
       // the invariant.
       // rune:temperare(serde-api-shape) — serde_json::to_string takes
       // &impl Serialize; we materialize via edn_to_json into a full
       // serde_json::Value tree before serializing. Alternative is a
       // custom serde::Serialize impl on Value (full visitor). No caller
       // has surfaced JSON-throughput pressure; the simpler shape wins
       // until measurement disagrees.
       serde_json::to_string(&edn_to_json(v)).expect("serde_json::to_string on Value")
   }
   ```

2. **At `crates/wat-edn/src/json.rs:175-184`** (`to_json_string_pretty`), append the equivalent rune (mirror the above; mention `to_string_pretty` instead of `to_string`).

### Part (g) — retire `parse_wire` + `parse_wire_owned`

1. **At `crates/wat-edn/src/lib.rs:130-152`**, delete:
   - The docstring block (lines 130-143)
   - `pub fn parse_wire(input: &str) -> Result<Value<'_>>` (lines 144-146)
   - `pub fn parse_wire_owned(input: &str) -> Result<OwnedValue>` (lines 148-152) including its docstring

2. **At `crates/wat-edn/tests/wire_encoding.rs`**, migrate every `parse_wire`/`parse_wire_owned` call site to the underlying API:
   - `use wat_edn::{parse, parse_wire, write, Keyword, Value};` → `use wat_edn::{parse, write, Keyword, Parser, Value};`
   - `parse_wire(x)` → `Parser::new_wire(x).parse_top()`
   - `parse_wire_owned(x)` → `Parser::new_wire(x).parse_top().map(Value::into_owned)` (if any)
   - Doc comments in the file mentioning `parse_wire` → rephrase to `Parser::new_wire`
   - Round-trip helper `unwrap_or_else(|e| panic!("parse_wire failed ..."))` → `unwrap_or_else(|e| panic!("Parser::new_wire(...).parse_top() failed ..."))` (or similar)

3. **At `crates/wat-edn/docs/USER-GUIDE.md`**, locate the parse section updated by Stone 218.4 (around lines 140-170) and:
   - Remove the `parse_wire` / `parse_wire_owned` from the imports list and the documentation paragraph
   - Restate the section heading and intro from "Four free-function entry points and a `Parser` builder" back to its 218.3-era form OR amend to "Three free-function entry points (`parse`, `parse_owned`, `parse_all`) and a `Parser` builder (use `Parser::new_wire` for wire-mode)"
   - Keep the wire-mode teaching paragraph but route the reader to `Parser::new_wire(input).parse_top()` as the canonical access

### Verification (must run; no exceptions)

After all edits land:

1. `cargo build --release -p wat-edn` — must be 0 warnings, 0 errors
2. `cargo test --release -p wat-edn` — current baseline 342 PASS (verified pre-spawn 2026-05-22 — note: 218.4 SCORE cited 339, but arc 219 added 3 spec_strict tests since then; current truth is 342). Expected post-stone: 342 + supplementary-plane probe + any optional `InvalidSet` decode probe = 343 or 344. Report actual.
3. `cargo test --release --lib -p wat` — must stay green at 824/0 (verified pre-spawn 2026-05-22)
4. `cargo clippy --release -p wat-edn -- -D warnings` — 0 warnings
5. **Interop-tests four handshakes (mandatory per `feedback_wat_edn_touch_runs_interop_tests`):**
   ```sh
   cd crates/wat-edn/interop-tests
   cargo build --release
   cargo run --release --bin wat-edn-interop-tests | clojure -M clj/consume.clj
   clojure -M clj/produce.clj | cargo run --release --bin reader
   cargo run --release --bin shape_matrix | clojure -M clj/consume_shapes.clj
   clojure -M clj/produce_shapes.clj | cargo run --release --bin shape_matrix_reader
   ```
   All four must pass. The shape matrix now exercises the supplementary-plane char (new probe from Part (a)).

## STOP triggers

- **STOP-1 (write_char fix regresses BMP test):** If any existing test fails on BMP char output, the shape preserves BMP behavior — re-check the recommended structure. If a real regression on BMP is the only achievable path, STOP and report.
- **STOP-2 (`is_canonical_uuid` move surfaces unexpected consumer):** vigilia identified two call sites; if a third surfaces, report and STOP before completing the move.
- **STOP-3 (paired-call replacement breaks an assertion):** if any value.rs unit test asserts on `translate_wat_to_strict`'s specific diagnostic shape, the combiner may change wording — report and STOP for orchestrator guidance.
- **STOP-4 (wire_encoding.rs migration fails on a test):** if any `Parser::new_wire(x).parse_top()` doesn't produce the same result as `parse_wire(x)`, the wrapper had hidden semantics — report and STOP.
- **STOP-5 (USER-GUIDE has cross-references to parse_wire beyond §3):** if other USER-GUIDE sections reference parse_wire, report all sites and STOP for scope-pause.
- **STOP-6 (interop-tests fail on the new probe):** the shape matrix emoji probe is the empirical proof of the writer fix. If the handshakes fail, the fix is incomplete; STOP.
- **STOP-7 (60 min elapsed):** wall-clock budget exceeded; ship what's clean, surface what's not.

## Out-of-scope

- L2 findings (struere, solvere L2, cernere L2, temperare L2, intueri, purgare L2) — Stone 218.7
- INSCRIPTION + re-cast vigilia — Stone 218.5 (redefined)
- Touching tagged-literal naming — arc 216.8/.9 territory
- Performance optimization beyond surfaced items
- Adding new public surface (the discipline is retire-then-mint-with-caller, not pre-mint)
