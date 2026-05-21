# BRIEF — Arc 218 Stone 218.6b — Emoji revert + interop-tests warning cleanup

**Stone scope (sonnet portion):** two bundled cleanups discovered post-Stone-218.6 ship via user direction + comprehensive warning audit (2026-05-22). **Part A** retracts Stone 218.6's supplementary-plane char "support" — wat-edn aligns to BMP-only char literals (Clojure can't read `\😀`; wat-edn shouldn't pretend to). **Part B** clears the 4 clippy items surfaced by `cargo clippy --release --all-targets` on `crates/wat-edn/interop-tests/` (2× PI approximation + 2× unused imports).
**Type:** Sonnet Mode A.
**Time budget:** 15-25 min target; 40 min STOP.
**Depends on:** Stone 218.6 (`45419e6` — 12/12 PASS with honest STOP-6 modified).
**Calibration:** 218.6 ~8 min (band 30-45, below). This stone smaller — pure delete/replace + mechanical edits. Band tightens to 15-25.
**Unblocks:** vigilia checkpoint cast on `crates/wat-edn/src/` to prove stability.

## User direction 2026-05-22

> *"uh.. fuck emojis - clojure doens't support them - wat doesn't either"*
>
> *"i think run the wards again to prove we're shockingly stable? i saw some warnings too?.. do we need to go measure for any warn/errors/whatevers from cargo and clippy in wat-edn -- pristine, high performance, algebraic perfection is our goal"*
>
> *"218 has work we haven't expressed yet - but it demands impeccable - so - 1 and 2 i agree to - 3 is not applicable - get wat-edn remarkable and then we'll discuss what 218 actually is"*

The Stone 218.6 supplementary-plane fix made the WRITER stop overflowing `\uXXXX` (real bug). But the chosen resolution — emit literal Unicode — accepts a form Clojure's `clojure.edn/read` rejects ("Unsupported character: \😀"). The honest discipline: wat-edn aligns to the cross-language intersection. BMP-only.

## Pre-flight verified (orchestrator-grep'd 2026-05-22, post-218.6)

### Part A — supplementary-plane char path

**Writer (`crates/wat-edn/src/writer.rs:307-330`):** `write_char` currently handles supplementary-plane by emitting literal Unicode (Stone 218.6 fix). Needs to PANIC instead.

**Lexer (`crates/wat-edn/src/lexer.rs:345-352`):** `lex_char` case 3 (single-character path) currently accepts ANY scalar char including supplementary-plane:

```rust
// 3. Single character? (`\a`, `\1`, etc.) — one iterator walk.
let mut it = body_str.chars();
if let Some(c) = it.next() {
    if it.next().is_none() {
        return Ok(Token::Char(c));
    }
}
```

Existing BMP non-ASCII case proven by lexer test at `lexer.rs:756`: `assert_eq!(lex_all("\\é"), vec![Token::Char('é')]);`. The `é` codepoint (U+00E9) is BMP. The lexer also accepts U+1F600 (😀) today without rejection — that's the gap.

**Test (`crates/wat-edn/tests/round_trip.rs:87-109`):** `supplementary_plane_char_round_trips` is the load-bearing artifact for the wrong behavior. Stone 218.6 added it to PROVE supplementary-plane round-trips; the new discipline REJECTS that round-trip. Replace with two negative tests.

**USER-GUIDE (`crates/wat-edn/docs/USER-GUIDE.md`):** char-literal section needs a BMP-only note for cross-language interop honesty.

### Part B — interop-tests warning cleanup

**4 clippy items (verified via `cargo clippy --release --all-targets` from `crates/wat-edn/interop-tests/`):**

1. **`crates/wat-edn/interop-tests/src/bin/shape_matrix.rs:37`**:
   ```rust
   (kw("primitive-f64"), Value::Float(3.14)),
   ```
   `clippy::approx_constant` — 3.14 is close to PI; lint assumes intent.

2. **`crates/wat-edn/interop-tests/src/bin/shape_matrix_reader.rs:70`**:
   ```rust
   |v| matches!(v, Value::Float(f) if (*f - 3.14).abs() < 1e-10), "3.14");
   ```
   Mirror site; same lint fires.

3. **`crates/wat-edn/interop-tests/src/main.rs:10`** — `use wat_edn::{write, Keyword, Symbol, Tag, Value};` — `Symbol` unused.

4. **`crates/wat-edn/interop-tests/src/bin/typed_reader.rs:5`** — `use wat_edn::{parse, Value};` — `Value` unused.

The `3.14` is a test FIXTURE for "a float scalar" in the shape matrix; the specific value carries no semantic meaning. Cleanest fix: use `2.5` (exact float representation, no PI ambiguity, both Rust and Clojure print/read it identically). Update both Rust sites + both Clojure sides (`consume_shapes.clj` assertion + `produce_shapes.clj` produce-value).

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`
- Per `feedback_no_known_defect_left_unfixed` — substrate trust binary; every L1 must land cleanly

## Your scope (sonnet)

### Part A — supplementary-plane char rejection

1. **At `crates/wat-edn/src/writer.rs:307` `write_char`**, modify so supplementary-plane codepoints (`> 0xFFFF`) panic with a clear diagnostic. Recommended shape:

   ```rust
   fn write_char(c: char, out: &mut String) {
       out.push('\\');
       if let Some(name) = char_to_name(c) {
           out.push_str(name);
           return;
       }
       let cp = c as u32;
       // wat-edn aligns to BMP-only chars for cross-language interop
       // (clojure.edn/read rejects supplementary-plane char literals).
       // Surface the constraint at write time rather than emitting a form
       // downstream readers can't consume.
       if cp > 0xFFFF {
           panic!(
               "wat-edn char literal U+{:X} is supplementary-plane; \
                wat-edn aligns to BMP-only (U+0000..=U+FFFF) for \
                cross-language EDN interop",
               cp
           );
       }
       // BMP control bytes + DEL → \uXXXX (exactly 4 hex digits per spec).
       if cp < 0x20 || cp == 0x7F {
           write!(out, "u{:04X}", cp).unwrap();
           return;
       }
       // BMP non-control non-printable still fits in 4 digits.
       if !(0x20..=0x7E).contains(&cp) {
           write!(out, "u{:04X}", cp).unwrap();
           return;
       }
       // Printable ASCII → literal.
       out.push(c);
   }
   ```

   Honest delta accepted: a cleaner shape that fires the panic AND preserves BMP behavior.

2. **At `crates/wat-edn/src/lexer.rs:345-352`**, gate the single-character path on BMP. Recommended shape:

   ```rust
   // 3. Single character? (`\a`, `\1`, etc.) — one iterator walk.
   let mut it = body_str.chars();
   if let Some(c) = it.next() {
       if it.next().is_none() {
           if (c as u32) > 0xFFFF {
               return Err(Error::at(
                   start,
                   ErrorKind::InvalidChar(format!(
                       "\\{}: supplementary-plane (U+{:X}) not supported; \
                        wat-edn char literals are BMP-only",
                       body_str, c as u32
                   )),
               ));
           }
           return Ok(Token::Char(c));
       }
   }
   ```

   Honest delta accepted: cleaner error wording; the `ErrorKind::InvalidChar` variant exists already (no enum addition needed).

3. **At `crates/wat-edn/tests/round_trip.rs:87-109`**, REPLACE the entire `supplementary_plane_char_round_trips` test with two negative tests:

   ```rust
   #[test]
   #[should_panic(expected = "supplementary-plane")]
   fn writer_panics_on_supplementary_plane_char() {
       // wat-edn aligns to BMP-only chars for Clojure/EDN cross-language
       // interop. The writer refuses to emit forms downstream readers
       // can't consume; see also lexer rejection probe.
       let _ = write(&Value::Char('😀'));
   }

   #[test]
   fn parser_rejects_supplementary_plane_char_literal() {
       // Symmetric strictness: source authors writing \😀 in EDN text
       // get a clear InvalidChar diagnostic. wat-edn char literals are
       // BMP-only (U+0000..=U+FFFF).
       let err = parse("\\😀").expect_err("supplementary-plane char must reject");
       let msg = format!("{}", err);
       assert!(
           msg.contains("supplementary-plane") || msg.contains("BMP"),
           "diagnostic must surface the BMP constraint; got: {}",
           msg
       );
   }
   ```

4. **At `crates/wat-edn/docs/USER-GUIDE.md`**, locate the character-literal section (search for `char` or `\char` or named-char content). Add a one-paragraph BMP-only note explaining the constraint:

   > **wat-edn char literals are BMP-only** (Unicode U+0000 through U+FFFF). Supplementary-plane codepoints (e.g. emoji) are rejected by both the parser and the writer for Clojure/EDN cross-language interop honesty — `clojure.edn/read` does not support `\😀`-style literals. Encode supplementary-plane content as strings instead.

   Exact placement + wording is sonnet's call.

### Part B — interop-tests warning cleanup

5. **At `crates/wat-edn/interop-tests/src/bin/shape_matrix.rs:37`** — change `Value::Float(3.14)` → `Value::Float(2.5)`.

6. **At `crates/wat-edn/interop-tests/src/bin/shape_matrix_reader.rs:70`** — change the assertion to compare against `2.5`:
   ```rust
   |v| matches!(v, Value::Float(f) if (*f - 2.5).abs() < 1e-10), "2.5");
   ```

7. **At `crates/wat-edn/interop-tests/clj/consume_shapes.clj`** — locate the `:primitive-f64` assertion (likely something like `= 3.14`) and update to `2.5`.

8. **At `crates/wat-edn/interop-tests/clj/produce_shapes.clj`** — locate the `:primitive-f64` produce value (likely `3.14`) and update to `2.5`.

9. **At `crates/wat-edn/interop-tests/src/main.rs:10`** — drop `Symbol`:
   ```rust
   use wat_edn::{write, Keyword, Tag, Value};
   ```

10. **At `crates/wat-edn/interop-tests/src/bin/typed_reader.rs:5`** — drop `Value`:
    ```rust
    use wat_edn::{parse};
    ```
    (Or whatever cleaner form sonnet picks; rustfmt may suggest just `use wat_edn::parse;` since it's a single-item import.)

### Verification (must run; no exceptions)

After all edits land:

1. `cargo build --release -p wat-edn` — 0 warnings, 0 errors
2. `cargo test --release -p wat-edn` — count delta: 343 baseline (post-218.6) - 1 (removed `supplementary_plane_char_round_trips`) + 2 (writer_panics + parser_rejects) = expected 344. Report actual.
3. `cargo test --release --lib -p wat` — 824/0 PASS (no regressions)
4. `cargo clippy --release --all-targets -p wat-edn -- -D warnings` — 0 warnings, 0 errors
5. **From `crates/wat-edn/interop-tests/`:**
   - `cargo build --release` — 0 warnings, 0 errors
   - `cargo clippy --release --all-targets -- -D warnings` — 0 warnings, 0 errors
6. **Interop-tests 4 handshakes (mandatory per `feedback_wat_edn_touch_runs_interop_tests`):**
   ```sh
   cd crates/wat-edn/interop-tests
   cargo build --release
   cargo run --release --bin wat-edn-interop-tests | clojure -M clj/consume.clj
   clojure -M clj/produce.clj | cargo run --release --bin reader
   cargo run --release --bin shape_matrix | clojure -M clj/consume_shapes.clj
   clojure -M clj/produce_shapes.clj | cargo run --release --bin shape_matrix_reader
   ```
   All four must pass; shape matrix now exchanges `:primitive-f64 = 2.5` both ways.

## STOP triggers

- **STOP-1 (panic test doesn't fire):** if `#[should_panic(expected = "supplementary-plane")]` doesn't catch the panic, the panic message wording diverged — adjust test or panic to align; report.
- **STOP-2 (parser-side rejection breaks an existing test):** if any existing test in `lexer.rs`, `parser.rs`, or `tests/` uses a supplementary-plane char in input, the rejection will surface it. Report + adjust the test (it was testing wrong behavior).
- **STOP-3 (USER-GUIDE has more char-literal claims than the section located):** grep for `\\u`, `char`, or `\\\\\\\\` patterns; if a teaching example uses a supplementary-plane char, fix it (it was incorrect).
- **STOP-4 (interop-tests handshakes fail on the 2.5 update):** ensure all 4 sites (Rust + Clojure × shape_matrix + reader) use 2.5 consistently. The matrix is bidirectional.
- **STOP-5 (clippy surfaces additional warnings post-edit):** if dropping the unused imports surfaces other warnings (e.g. a USE statement collapses into a simpler form), fix them too — the goal is `-D warnings` clean.
- **STOP-6 (40 min elapsed):** wall-clock STOP; ship what's clean, surface what's not.

## Out-of-scope

- L2 vigilia findings (struere, solvere L2, cernere L2, temperare L2, intueri, purgare L2) — those land after the checkpoint vigilia cast informs final 218.7 scope
- INSCRIPTION — arc 218 closure deferred per user direction ("218 has work we haven't expressed yet")
- Any tag/struct/macro surface — only char-literal alignment + interop warning cleanup
- New public surface — retire-then-mint discipline holds
