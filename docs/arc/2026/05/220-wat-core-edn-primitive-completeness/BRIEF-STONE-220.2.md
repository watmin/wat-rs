# BRIEF — Arc 220 Stone 220.2 — `:wat::core::Char` primitive (BMP-only)

**Stone scope (sonnet portion):** mint `:wat::core::Char` following Uuid precedent (arc 207). 10 Rust arm sites + lexer addition for `\c` literal + constructor function + edn_shim bridge + wat-tests.
**Type:** Sonnet Mode A.
**Time budget:** 60-90 min target; 120 min STOP.
**Depends on:** arc 220 Slice 1 DESIGN.md (`8393722`).
**Calibration:** 11 stones at-or-below band in series. This stone has 3 novel surfaces (lexer `\c`; new variant; constructor function); other items are Uuid-precedent mechanical. Band 60-90.
**Unblocks:** Slice 3 (`'` reader macro) + Slice 4 (List, which inherits the variant+arm+constructor patterns from this slice).

## User resolutions baked in (per arc 220 DESIGN + 2026-05-22 conversation)

- **F:** `\c` literal syntax in wat source — **the Clojure-on-Rust form**. EDN-aligned. (Note: the existing `src/lexer.rs:1-58` doc comment lists `#\a` as future-extension — that was Common-Lisp/Scheme-style and WRONG per the wat-rs lineage. Update the doc comment to `\c` per arc 220.)
- **G:** `'` reader macro at form-start (`'(1 2 3)` → quote-wrapped) is **Slice 3 territory; NOT in this stone**. Note: arc 171's `'` AS KEYWORD-BODY DISCRIMINATOR (e.g. `:foo'2`) is unchanged — Clojure handles both lexically by position, so will wat.
- **H:** `(:wat::core::Char/of "x")` — String length-1 input
- **BMP-only:** inherits wat-edn Stone 218.6b discipline (panic on supplementary-plane; symmetric strictness across lex + construct)

## Pre-flight verified (orchestrator-grep'd 2026-05-22)

### Variant + arm sites (Uuid precedent — Char follows identically)

10 sites total:

| # | File:line | Pattern (Uuid) | Char counterpart |
|---|---|---|---|
| 1 | `src/runtime.rs:616` | `wat__core__Uuid(uuid::Uuid)` (last variant) | Add `wat__core__Char(char)` next to / after |
| 2 | `src/runtime.rs:654` | `(Value::wat__core__Uuid(a), Value::wat__core__Uuid(b)) => a == b` | Same pattern for Char |
| 3 | `src/runtime.rs:761` | `Value::wat__core__Uuid(u) => u.hash(state)` | Same for Char |
| 4 | `src/runtime.rs:1043` | `Value::wat__core__Uuid(_) => "wat::core::Uuid"` | `"wat::core::Char"` |
| 5 | `src/runtime.rs:7102` | structural-eq arm | Same pattern for Char |
| 6 | `src/runtime.rs:15904` | `Value::wat__core__Uuid(u) => format!("#uuid \"{}\"", u)` | `Value::wat__core__Char(c) => format!("\\{}", c)` (EDN char literal; per spec) |
| 7 | `src/edn_shim.rs:411` | `Edn::Uuid(u) => Ok(Value::wat__core__Uuid(*u))` (parse direction) | `Edn::Char(c) => Ok(Value::wat__core__Char(*c))` |
| 8 | `src/edn_shim.rs:589` | second parse-direction use | Same |
| 9 | `src/edn_shim.rs:1630` | `Value::wat__core__Uuid(u) => OwnedValue::Uuid(*u)` (write direction) | `Value::wat__core__Char(c) => OwnedValue::Char(*c)` |
| 10 | `src/closure_extract.rs:1492` | `Value::wat__core__Uuid(u) => Ok(WatAST::List(...))` (closure capture) | Same pattern for Char |

### Constructor function pattern (Uuid precedent — verbatim)

`src/string_ops.rs:252-271` — `eval_uuid_typed_v4`:

```rust
/// `(:wat::core::Uuid/v4)` → `:wat::core::Uuid`.
///
/// Mints a fresh v4 (random) UUID on every call. Returns a typed
/// `:wat::core::Uuid` value — NOT a string. Arc 207 slice 2.
pub fn eval_uuid_typed_v4(
    args: &[WatAST],
    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::Uuid/v4";
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 0,
            got: args.len(),
            span: args[0].span().clone(),
        });
    }
    Ok(Value::wat__core__Uuid(wat_edn::new_uuid_v4()))
}
```

`src/runtime.rs:4570` dispatch:
```rust
":wat::core::Uuid/v4" => crate::string_ops::eval_uuid_typed_v4(args, env, sym),
```

For Char: add `eval_char_of` following the same shape (1 arg, validate, construct). Take 1 `Value::String` arg, validate `s.len() == 1` (in chars), validate `(c as u32) <= 0xFFFF`, return `Value::wat__core__Char(c)`. Errors with `RuntimeError::TypeMismatch` / clear per-condition diagnostics.

### Closure-extract arm (Uuid precedent — verbatim)

`src/closure_extract.rs:1492-1500`:

```rust
// Arc 207 — Uuid is portable: encode as a `Uuid/from-string` call
// on the canonical 8-4-4-4-12 hyphenated form. Round-trips cleanly.
Value::wat__core__Uuid(u) => Ok(WatAST::List(
    vec![
        WatAST::Keyword(":wat::core::Uuid/from-string".into(), span.clone()),
        WatAST::StringLit(u.to_string(), span.clone()),
    ],
    span,
)),
```

For Char: encode as `(:wat::core::Char/of "x")` reverse-construction:

```rust
// Arc 220 — Char is portable: encode as a `Char/of` call on a
// length-1 String. Round-trips cleanly (BMP guaranteed by construct).
Value::wat__core__Char(c) => Ok(WatAST::List(
    vec![
        WatAST::Keyword(":wat::core::Char/of".into(), span.clone()),
        WatAST::StringLit(c.to_string(), span.clone()),
    ],
    span,
)),
```

### Lexer addition (novel — no existing `lex_char`)

- `src/lexer.rs` has `lex_string` (391), `lex_keyword` (440), `lex_numeric_or_symbol` (579), `lex_symbol` (600)
- Need to add `lex_char` that handles `\c` / `\newline` / `\space` / `\tab` / `\return` / `\uNNNN` per EDN spec § characters
- Tokenizer entry dispatches on byte; add `b'\\' =>` case routing to `lex_char`
- BMP-only check at lex time: reject `\uNNNN` where `NNNN > 0xFFFF` (already structurally impossible since `\uNNNN` is exactly 4 hex digits, max `￿`); reject literal `\😀` (supplementary-plane char in source) with diagnostic

**Reference: `crates/wat-edn/src/lexer.rs:288-355` — verbatim lex_char shape (adapt to wat-rs `Lexer` struct conventions):**

```rust
fn lex_char(&mut self) -> Result<Token<'a>> {
    debug_assert_eq!(self.peek(), Some(b'\\'));
    let start = self.pos;
    self.pos += 1;
    let body_start = self.pos;

    // Spec: "Backslash cannot be followed by whitespace."
    // Capture `first` here to avoid a second peek below.
    let first = match self.peek() {
        None => return Err(Error::at(start, ErrorKind::InvalidChar("empty".into()))),
        Some(b) if is_whitespace(b) => {
            return Err(Error::at(
                start,
                ErrorKind::InvalidChar("backslash followed by whitespace".into()),
            ))
        }
        Some(b) => b,
    };

    // Single non-alpha non-digit character (`\(`, `\;`, `\é`, etc.).
    // Alphanumeric bodies fall through to the named-char path below
    // (where `\newline`, `\space`, `\a`, `\1` resolve uniformly).
    if !first.is_ascii_alphanumeric() {
        let (c, byte_len) = decode_utf8_char(&self.input[self.pos..])
            .map_err(|e| Error::at(self.pos, ErrorKind::Utf8(e)))?;
        if (c as u32) > 0xFFFF {
            return Err(Error::at(
                start,
                ErrorKind::InvalidChar(format!(
                    "\\{}: supplementary-plane (U+{:X}) not supported; \
                     wat char literals are BMP-only",
                    c, c as u32
                )),
            ));
        }
        self.pos += byte_len;
        return Ok(Token::Char(c));
    }

    // Read alphanumeric body (a name like "newline", "u00A0", or single letter)
    while let Some(b) = self.peek() {
        if b.is_ascii_alphanumeric() {
            self.pos += 1;
        } else {
            break;
        }
    }

    let body = &self.input[body_start..self.pos];
    let body_str = std::str::from_utf8(body)
        .map_err(|e| Error::at(body_start, ErrorKind::Utf8(e.to_string())))?;

    // 1. Named char literal? (`\newline`, `\space`, etc.)
    match body_str {
        "newline" => return Ok(Token::Char('\n')),
        "return"  => return Ok(Token::Char('\r')),
        "space"   => return Ok(Token::Char(' ')),
        "tab"     => return Ok(Token::Char('\t')),
        _ => {}
    }
    // 2. `\uNNNN` Unicode escape?
    if body_str.len() == 5 && body_str.starts_with('u') {
        let acc = u32::from_str_radix(&body_str[1..], 16).map_err(|_| {
            Error::at(start, ErrorKind::InvalidChar(format!("\\{}", body_str)))
        })?;
        let c = char::from_u32(acc).ok_or_else(|| {
            Error::at(start, ErrorKind::InvalidChar(format!("\\{}: not a scalar", body_str)))
        })?;
        return Ok(Token::Char(c));
    }
    // 3. Single character? (`\a`, `\1`, etc.) — one iterator walk.
    let mut it = body_str.chars();
    if let Some(c) = it.next() {
        if it.next().is_none() {
            return Ok(Token::Char(c));
        }
    }
    Err(Error::at(start, ErrorKind::InvalidChar(body_str.into())))
}
```

Adapt to wat-rs's lexer struct (different field names; `LexError` enum vs `Error::at`; different is_whitespace check). The structural shape is the load-bearing reference.

### edn_shim bridge (existing precedent for Uuid)

The 3 edn_shim.rs sites above (lines 411, 589, 1630) are the bridge pattern. Add Char arms in the same places — mirrors symmetric structure.

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`
- BMP-only inherits Stone 218.6b discipline (panic on supplementary-plane char in writer; ErrorKind::InvalidChar on supplementary-plane char in lexer)

## Your scope (sonnet)

Execute in order: A (variant + arms) → B (constructor) → C (lexer) → D (edn_shim bridge) → E (tests).

### A. Variant + 10 match-arm sites

Add `Value::wat__core__Char(char)` variant at `src/runtime.rs:616-617` (after Uuid, before closing `}`). Then add 10 arms per Uuid-precedent map above. Honest delta accepted: cleaner placement / wording if a clear improvement.

### B. Constructor `:wat::core::Char/of`

1. `src/string_ops.rs` — add `pub fn eval_char_of(args: &[Value], env: &Env, sym: &str) -> EvalResult` following `eval_uuid_typed_v4` pattern. Const op `":wat::core::Char/of"`. Take 1 arg `:wat::core::String`; assert length == 1 + first char is BMP; return `Value::wat__core__Char(c)`. Errors with clear diagnostics per BRIEF.
2. `src/runtime.rs:~4570` area — add dispatch entry `":wat::core::Char/of" => crate::string_ops::eval_char_of(args, env, sym),`

### C. Lexer `\c` literal

1. `src/lexer.rs` — add `fn lex_char(src: &str, start: usize) -> Result<(Token, usize), LexError>` after `lex_string`. **Verbatim reference shape in "Pre-flight verified" Lexer-addition section above** (from `crates/wat-edn/src/lexer.rs`).
2. `src/lexer.rs` — add `Token::Char(char)` enum variant if not present
3. Tokenizer entry — dispatch on `b'\\'` to `lex_char` (note: existing `\` in strings is INSIDE `"..."` — string-escape handling is in `lex_string`; standalone `\` outside strings goes to `lex_char`)
4. `src/parser.rs` — handle `Token::Char(c)` → `Value::wat__core__Char(c)` in atom-parsing
5. **Update doc comment** at `src/lexer.rs:1-58` — the existing "Future extensions (not in MVP): character literals `#\a`" line is WRONG per arc 220 (wat is clojure-on-rust; uses `\c`). Replace with `"Character literals: `\c` / `\newline` / `\return` / `\space` / `\tab` / `\uNNNN` per arc 220 (Clojure/EDN convention)."`
6. STOP-1 trigger: if `b'\\'` outside strings is already used for any other purpose, surface + pause

### D. edn_shim bridge (3 sites)

Per the Uuid-precedent map: add Char arms at `edn_shim.rs:411`, `:589`, `:1630`. Mirrors symmetric structure.

### E. Tests

1. **`tests/wat_arc220_char.rs`** — new Rust integration test file (or extend an existing relevant test):
   - Lexer accepts `\c`, `\newline`, `\space`, `\tab`, `\return`, `\uNNNN`
   - Lexer rejects supplementary-plane char literal `\😀` with clear diagnostic
   - `(:wat::core::Char/of "x")` returns Value::wat__core__Char('x')
   - `(:wat::core::Char/of "")` errors with "length-1 String" diagnostic
   - `(:wat::core::Char/of "ab")` errors with "length-1, got 2" diagnostic
   - `(:wat::core::Char/of "😀")` errors with "supplementary-plane" diagnostic
   - Round-trip: parse `\x` in wat source → Value → write to EDN → reparse → identical
2. **`wat-tests/holon/char_round_trip.wat`** — new wat-source test exercising `\c` literal + `Char/of` constructor; uses existing `assert-eq!` discipline
3. **`crates/wat-edn/interop-tests/src/bin/shape_matrix.rs`** — add `:char-bmp` shape: `Value::Char('x')` (BMP, safe). Mirror in `shape_matrix_reader.rs` + `consume_shapes.clj` + `produce_shapes.clj`

### Verification (must run before SCORE)

1. `cargo build --release` — workspace clean, 0 warnings
2. `cargo test --release --lib -p wat` — passes with new test count (delta: +N for the new Char tests)
3. `cargo test --release -p wat-edn` — 344 PASS (untouched by Char addition — only the wat-rs side adds; wat-edn's Value::Char already exists)
4. `cargo clippy --release --all-targets -p wat -- -D warnings` — 0 warnings
5. **Interop-tests 4 handshakes** (mandatory per `feedback_wat_edn_touch_runs_interop_tests` — Stone B touched interop-tests/shape_matrix):
   - `cd crates/wat-edn/interop-tests`
   - `cargo run --release --bin wat-edn-interop-tests | clojure -M clj/consume.clj`
   - `clojure -M clj/produce.clj | cargo run --release --bin reader`
   - `cargo run --release --bin shape_matrix | clojure -M clj/consume_shapes.clj` (now with `:char-bmp`)
   - `clojure -M clj/produce_shapes.clj | cargo run --release --bin shape_matrix_reader`

**NOTE on handshakes:** if sub-agent piped-bash permission wall denies (218.6b/c/d/e precedent), ship the rest cleanly + write SCORE marking row as "pending orchestrator-side verification". Orchestrator runs during scoring.

**Write `docs/arc/2026/05/220-wat-core-edn-primitive-completeness/SCORE-STONE-220.2.md`** mirroring SCORE-STONE-218.6e shape.

## STOP triggers

- **STOP-1 (lexer `b'\\'` conflict):** if `\` is already used for any wat lexer purpose, surface + pause for orchestrator guidance
- **STOP-2 (variant addition cascade breaks more than ~10 sites):** if cascade exceeds Uuid-precedent count, report — may indicate non-Uuid-like usage pattern that needs investigation
- **STOP-3 (existing wat test uses Char-like syntax for something else):** unlikely; report if any test breaks unexpectedly
- **STOP-4 (Holon-encoding bridge breaks):** Value::wat__core__Char should flow through Bundle-encoding per arc 216 doctrine; if encoder rejects Char (no impl), report — may need HolonRepresentable<char> impl in src/comms/mod.rs
- **STOP-5 (interop handshakes fail beyond known permission wall):** report
- **STOP-6 (120 min elapsed):** wall-clock STOP

## Out-of-scope

- `:wat::core::List` (Slice 4)
- `'` reader macro (Slice 3)
- BigInt / BigDec wat-core types (deferred per DESIGN)
- Any wat-edn modifications (wat-edn's Value::Char already exists; only the wat-rs side adds)
- New public surface beyond `:wat::core::Char/of` constructor + `\c` literal
- HolonAST extension (collections-as-holons handles via Bundle; Char as scalar uses existing String leaf? Verify via STOP-4)
