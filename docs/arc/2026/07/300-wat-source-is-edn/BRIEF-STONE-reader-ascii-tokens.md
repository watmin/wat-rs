# BRIEF — reader ASCII-token totality (true up wat-edn; stop wat-reader panicking)

**The work (one paragraph).** wat's token grammar is ASCII-only (a deliberate "no wide chars in tokens" stance,
stricter than `clojure.edn` which accepts Unicode symbols — grounded differential in the RED probes). `wat-edn`
already enforces it *cleanly* (returns `Err`) but with a cryptic message; `wat-reader` doesn't enforce it at all
— it **panics** (byte-wise `lex_symbol` slices a multi-byte char mid-boundary). Two small, aligned fixes: give
`wat-edn`'s refusal a clear diagnostic, and make `wat-reader` refuse cleanly (never panic). This is the
reader-totality step (#1); it also lets the arc-278 inline-wat gate later drop its `catch_unwind`.

The spec is the two committed RED probes — turn them green:
- `crates/wat-reader/tests/reader_totality.rs` — `non_ascii_in_token_position` (panics today → must be clean `Err`); `non_ascii_inside_string_still_parses` must stay green.
- `crates/wat-edn/tests/token_ascii_stance.rs` — `non_ascii_token_error_is_clear` (cryptic today → must name the reason); `token_ascii_stance_pinned` must stay green.

## Part A — wat-edn: true up the error (behavior already correct)

- **Room:** `crates/wat-edn/src/lexer.rs:183` — the token dispatch's catch-all `_ => Err(Error::at(self.pos, ErrorKind::UnexpectedByte(b)))`. `crates/wat-edn/src/error.rs:16,44` — the `UnexpectedByte` variant + Display.
- **Fix:** before falling through to `UnexpectedByte`, detect a non-ASCII lead byte (`b >= 0x80`) in token-start position and return a *clear* error — a new `ErrorKind::NonAsciiInToken(u8)` (Display: `"non-ASCII byte 0x{b:02x} in token position; wat tokens are ASCII — use a string for text"`), or an equivalent message. Model the tone on the existing char-literal message (lexer.rs:314). Do NOT change what is accepted/refused (the stance is already correct — `token_ascii_stance_pinned` passes); only make the refusal legible.

## Part B — wat-reader: stop panicking (refuse cleanly, like wat-edn)

- **Rooms:** `crates/wat-reader/src/lexer.rs` — the main lex loop's control-char rejection (`if c.is_control() { return Err(... ControlCharacterInSource ...) }`, ~line 279) is the sibling slot; `lex_symbol` (839–876) is where the byte-wise `src[start..i]` panics. `LexErrorKind` (161+) is the error enum.
- **Fix:** add a **non-ASCII-in-token guard** so `lex_symbol` never sees a multi-byte char. The clean place is the main dispatch, AFTER `"`→`lex_string` and `\`→`lex_char` are handled (so UTF-8 string content and BMP char literals still work), mirroring the control-char check: if the token-start char `c` is non-ASCII (`!c.is_ascii()`), return a clean `LexError` (a new `LexErrorKind::NonAsciiInToken` with a message parallel to wat-edn's). Result: non-ASCII token → clean `Err`; the byte-slice is never reached mid-char.
- **Preserve:** `"héllo"` / `"a 😀 b"` (UTF-8 strings) and `\é` (BMP char literal) must still parse — verify the existing lexer tests + `non_ascii_inside_string_still_parses` stay green.

## STOP triggers

- STOP if the non-ASCII guard would also reject valid UTF-8 **string content** or BMP **char literals** → you've placed it before the `"`/`\` dispatch; move it after. Strings and char literals are NOT tokens.
- Do NOT change the ACCEPT/REFUSE stance (ASCII tokens, UTF-8 strings, BMP char literals) — only the panic and the error clarity. `clojure.edn` parity on symbols is a *separate* design call the builder has not made; do not "fix" it by accepting Unicode symbols.
- Scope: `crates/wat-edn/` + `crates/wat-reader/` only. No rete, no `wat/`, no test-source sweeps.

## Done = green

- `cargo test -p wat-reader --test reader_totality` → both tests pass (no panic; clean `Err`; strings still parse).
- `cargo test -p wat-edn --test token_ascii_stance` → both tests pass (clear error; stance intact).
- `cargo test -p wat-reader` and `cargo test -p wat-edn` → no regression (existing lexer/parser tests green).
- Report: the new error variants/messages, files changed, and confirmation the string + char-literal paths are untouched.
