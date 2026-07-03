# BRIEF — Unicode-token parity (wat-edn accepts; wat-reader stops panicking)

> Supersedes `BRIEF-STONE-reader-ascii-tokens.md` (retired — it enshrined an ASCII-only stance, which is
> an ILLEGAL non-parity state: `clojure.edn` is the oracle, and it accepts Unicode symbols/keywords).

**The frame.** `clojure.edn` is the oracle; **non-parity is an illegal state.** Grounded differential
(Clojure 1.12.4): clj reads `😀`/`é`/`λ`/`foo→bar` as Symbols and `:a😀`/`:λ` as Keywords; it *refuses*
the supplementary char literal `\😀`. Two separate readers, two separate obligations:

- **`wat-edn`** (the EDN reader) is the clj-parity target — it must **accept** Unicode tokens. Today it
  refuses them (`ErrorKind::UnexpectedByte`). **This is the real bug.**
- **`wat-reader`** (the wat *source* reader) reads a narrower grammar; a Unicode symbol isn't wat source,
  so it need not parse it — but it must **not panic** (today its byte-wise `lex_symbol` mid-slices a
  multi-byte char → panic). It errors cleanly. The 300 convergence (`wat-reader` ← `wat-edn`) unifies them
  later; this strike just stops the panic.

The spec is the two committed RED probes — turn them green:
- `crates/wat-edn/tests/token_unicode_parity.rs` — `unicode_tokens_parse` (wat-edn must ACCEPT `😀`/`é`/`λ`/`:a😀`/…).
- `crates/wat-reader/tests/reader_totality.rs` — `non_ascii_token_errs_not_panics` (clean `Err`, never panic); `unicode_inside_string_still_parses` must stay green.

## Part A — wat-edn: ACCEPT Unicode tokens (parity with clj)

- **Rooms:** `crates/wat-edn/src/lexer.rs:183` — the token dispatch's `_ => Err(UnexpectedByte(b))` catch-all (where a non-ASCII lead byte dead-ends today). `lexer.rs:310-323` — the char-literal path already decodes a multi-byte char via `decode_utf8_char`; **reuse that pattern.** `lexer.rs:~329` — the symbol/keyword body loop gated on `is_ascii_alphanumeric` (must also admit non-ASCII scalars).
- **Fix:** in symbol/keyword lexing, admit a non-ASCII UTF-8 scalar as a token constituent — decode it (`decode_utf8_char`) and include it, exactly as `clojure.edn` does, so `😀`/`é`/`λ`/`foo→bar`/`:a😀`/`:λ` parse to Symbol/Keyword. Do NOT special-case ASCII-only.
- **Preserve (mutual parity with clj — do not change):** char literals stay **BMP-only** (`\😀` refuses — clj refuses it too; `\é` parses); UTF-8 string content parses.

## Part B — wat-reader: stop panicking (clean `Err`, do NOT parse)

- **Rooms:** `crates/wat-reader/src/lexer.rs` — the `if c.is_control()` rejection (~line 279) is the sibling slot; `lex_symbol` (839-876) is where `src[start..i]` panics.
- **Fix:** a non-ASCII byte in token-start position returns a clean `LexError` (a neutral `UnexpectedChar`-style kind — NOT a "tokens are ASCII" stance claim), placed AFTER the `"`→`lex_string` and `\`→`lex_char` dispatch so strings + BMP char literals are untouched. wat-reader does NOT need to parse the Unicode token — just refuse cleanly instead of mid-slicing.

## Deferred (NOT this strike)

- **Ratios** (`1/2`) — clj accepts, wat refuses, but wat has **no rational type yet** (builder: "something we'll work on later"). Known, tracked gap; the clj-oracle differential ward marks it an accepted exemption, not a red.

## STOP triggers

- STOP if the wat-edn change would accept a **supplementary char literal** (`\😀`) — clj refuses it; that's mutual parity, keep it refused.
- STOP if either change touches **string content** or **BMP char literals** — those already match clj.
- wat-reader must NOT try to *parse* the Unicode token (it's not wat source) — only stop the panic.
- Scope: `crates/wat-edn/` + `crates/wat-reader/` only.

## Done = green

- `cargo test -p wat-edn --test token_unicode_parity` → green (Unicode tokens parse; char/string boundary intact).
- `cargo test -p wat-reader --test reader_totality` → green (no panic, clean `Err`; strings parse).
- `cargo test -p wat-edn` and `cargo test -p wat-reader` → no regression.
- Report the wat-edn parse change + the wat-reader error kind, files changed, and confirmation strings/char-literals are untouched.
