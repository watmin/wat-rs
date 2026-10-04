;; wat/lex-errors.wat — excursus 003 strike B2, item 4: `LexErrorKind`
;; (`crates/wat-reader/src/lexer.rs:198`, 10 measured) and its wrapper
;; `LexError` become declared wat records.
;;
;; S3 found that a lex failure rode `ParseErrorKind::Lex.cause` as an OPAQUE
;; STRING — `LexError`'s own `Display` text, flattened — with no `ToEdn`/
;; `WatError` impl anywhere, so the offending character, the byte position,
;; and the kind were all discarded on the wire (`wat/parse-errors.wat`'s own
;; header documents the S3 measurement in full; `g_lex_never_produces_a_tag`,
;; `src/parser.rs`, was the driven proof). This strike closes it:
;;
;; - `LexErrorKind` is now ONE wat `defenum` (`:wat::lex::LexErrorKind`),
;;   `#[derive(ToEdn)]`'s `qualified` directive dot-joining every variant's
;;   wire tag (`#wat.lex/LexErrorKind.<Variant>`) — declared in
;;   `crates/wat-reader/src/lexer.rs` itself (the SAME crate `LexError`/
;;   `LexErrorKind` are defined in, mirroring how `ParseError`'s own `ToEdn`
;;   impl lives in `wat-reader`, not `wat` — the orphan rule).
;; - `LexError` (`{position, kind}`) is now its own tagged record
;;   (`#wat.lex/LexError {:position :kind}`, also `#[derive(ToEdn)]`). It is
;;   DATA, not an Error: no `message`/`location` of its own —
;;   `ParseErrorKind::Lex` already supplies the floor at the outer level, the
;;   SAME shape `LoadFetchError` (strike B2, item 2) has relative to
;;   `LoadErrorKind::Fetch`.
;; - `ParseErrorKind::Lex.cause` (`wat/parse-errors.wat`) is retyped from
;;   `:wat::core::String` to the concrete record, `:wat::lex::LexError`.
;;
;; Tag namespace: `crates/wat-reader/src/lexer.rs`'s own `LEX_NS` constant
;; (a bare Rust path — the derive's `namespace` directive forbids a string
;; literal) equals `"wat.lex"`; `:wat::lex::<Name>` is exactly
;; `#wat.lex/<Name>` on the wire. Must stay in sync with that constant by
;; convention — the SAME "two crates, one literal" shape `"wat.parse"`
;; already has between `wat-reader` and `wat` (see `wat/parse-errors.wat`'s
;; header).
;;
;; `usize`/`u32` fields (`position`, `codepoint`) type `:wat::core::i64`,
;; matching every other such field in this sweep. The offending `char` on
;; `UnexpectedChar`/`UnknownEscape` types `:wat::core::char` itself (excursus
;; 003 strike G item 3 — `:wat::core::char` registered as a `TypeEnv` leaf,
;; closing the TABLE-STONE-Q hole this field used to measure around). Before
;; that strike it typed `:wat::core::String` (a one-character string stand-in,
;; via `crate::lexer::char_to_edn_string` on the Rust side) because a field
;; declared `:wat::core::char` refused at decode (`UndeclaredFieldType`) —
;; `:wat::core::char` was a real runtime `Value` via `(:wat::core::char "x")`
;; but never a `TypeEnv` member.
;;
;; Loads after `wat/core.wat` (`:wat::core::i64`/`String`). See
;; `src/load/stdlib.rs`. Loads BEFORE `wat/parse-errors.wat` (`Lex.cause`
;; names `:wat::lex::LexError`).

;; The 10 structural failure modes a lex pass can raise.
(:wat::core::defenum :wat::lex::LexErrorKind :wat::enum::Pure
;; An unrecognized character at the current lex position.
  :UnexpectedChar           [char <- :wat::core::char]
;; A string literal's closing quote was never found before end of input.
  :UnterminatedString
;; `\<c>` inside a string names an escape sequence the lexer does not know.
  :UnknownEscape            [char <- :wat::core::char]
;; A numeric token failed to parse as either `i64` or `f64`.
  :InvalidNumber            [literal <- :wat::core::String]
;; Whitespace inside an unclosed `(` in a keyword body.
  :UnclosedBracketInKeyword
;; A comma inside a keyword body (retired, arc 171 — use `'` instead).
  :CommaInKeywordBody
;; A comma inside a bare symbol body (retired, arc 109 — use the `:-` binder).
  :CommaInSymbolBody
;; A `<` opened a type head inside a name (retired, arc 109 — use `:-`).
  :AngleTypeHeadInName
;; An invalid `\c` character literal (empty body, supplementary-plane
;; codepoint, unknown named char, or backslash followed by whitespace).
  :InvalidChar              [reason <- :wat::core::String]
;; A raw control character (other than `\t`/`\n`/`\r`) appeared in source.
  :ControlCharacterInSource [codepoint <- :wat::core::i64])

;; The wrapper: the byte position the failure was found at, plus the
;; structural failure itself. DATA, not an Error — see this file's header.
(:wat::core::defrecord :wat::lex::LexError
  [position <- :wat::core::i64
   kind     <- :wat::lex::LexErrorKind])
