;; wat/parse-errors.wat — excursus 003 sweep S3: every `ParseErrorKind` variant
;; (`crates/wat-reader/src/parser.rs:37`, 11 measured) becomes a declared wat
;; record.
;;
;; ── Measured first: how a parse error reaches the wire, from a SEPARATE crate ──
;;
;; `ParseError` is DEFINED in `wat-reader` (`crates/wat-reader/src/parser.rs`),
;; which does not depend on `wat` and so cannot implement `wat`'s
;; `edn::contract::WatError` trait (the orphan rule: neither the trait nor
;; the type would be local to that crate). `wat-reader` hand-writes
;; `impl wat_edn::ToEdn for ParseError` directly (no `#[derive(ToEdn)]` — the
;; derive macro lives in a crate `wat-reader` does not depend on either) that
;; emits `#wat.parse/<Variant> {<fields> :span {…}}` — variant fields, THEN
;; `:span` (not `:location`), and NO `:message` at all. That raw
;; shape does NOT satisfy `:wat::core::Error` (message/location).
;;
;; The FLOOR shape actually on the wire wherever a `ParseError` crosses as a
;; typed cause (`LoadErrorKind::Parse.cause`, `StdlibErrorKind::ParseFailed.cause`,
;; both via `error_edn_of`) comes from a SECOND impl, in the `wat` crate itself
;; (`src/parser.rs`, re-exporting `wat_reader::parser::*`):
;; `impl crate::edn::contract::WatError for ParseError` — `variant()` calls
;; `strip_span_from_tagged(self.to_edn())` (drops the raw `:span`), and the
;; trait's default `error_edn()` then inserts `:message` (span-free kind
;; `Display`, first line), `:location` (`self.span`) in front (excursus 003
;; strike B1: `causes` left the floor, F3 — the trait's `error_edn()` no
;; longer inserts one). THIS composed floor form — `#wat.parse/<Variant>
;; {:message … :location {…} <fields>}` — is what every record below mirrors.
;; The bare `wat-reader`-side `to_edn()` (`:span`, no floor) is never what
;; crosses the wire as a typed cause; it is PURE DECLARATION's mirror target
;; only through the `WatError`-composed form, exactly as every other
;; taxonomy's `error_edn()` is.
;;
;; Tag namespace: literal `"wat.parse"` in BOTH crates (`wat-reader`'s
;; `parser.rs` hardcodes the string — it cannot see `crate::error_ns::PARSE`,
;; which lives in `wat` — and `wat`'s `error_ns::PARSE = "wat.parse"` must
;; stay in sync by convention, per `error_ns.rs`'s own header). So
;; `:wat::parse::<Name>` is exactly `#wat.parse/<Name>` on the wire.
;;
;; ── `Lex` — a lex error rides inside a parse error as an OPAQUE STRING ────────
;;
;; `ParseErrorKind::Lex(LexError)` is the ONE variant whose Rust field is
;; itself a further taxonomy (`LexErrorKind`,
;; `crates/wat-reader/src/lexer.rs:198`, 10 variants, the brief's other S3
;; taxonomy). Measured: `LexError`/`LexErrorKind` has NO `ToEdn` impl and NO
;; `WatError` impl anywhere in either crate (grepped both trees) — the ONLY
;; place a `LexError` ever reaches EDN is `ParseError`'s hand-written
;; `to_edn()`, which renders it with `OwnedValue::String(e.to_string())` —
;; `LexError`'s OWN `Display` ("lex error at byte N: <kind message>"),
;; flattened to prose. No `#wat.lex/…` tag is EVER produced, by any code
;; path, for any of the 10 `LexErrorKind` variants, today. `Lex`'s own
;; record below is therefore declared with `cause <- :wat::core::String` (a
;; plain string field, not `:wat::core::Error` or `:wat::core::Value`) — the
;; honest mirror of what is actually on the wire.
;;
;; **No sibling "lex-errors" stdlib file exists — this file declares no
;; `:wat::lex::*` records at all.** PURE DECLARATION mirrors what
;; `error_edn()` emits TODAY; since no code path ever calls anything
;; resembling `error_edn()` on a bare `LexError` (it has no such method, and
;; no `ToEdn` either), there is no produced tag set for `LexErrorKind` to
;; mirror — declaring 10 `:wat::lex::<Kind>` records against zero producers
;; would not be declaration of the wire, it would be invention of a wire
;; that does not exist. This is measured empirically below
;; (`excursus_003_s3_gates::g_lex_never_produces_a_tag`,
;; `crates/wat-reader/src/lexer.rs`), not merely asserted here. Flagged for
;; the B worklist: a lex failure loses all structure (no offending
;; character, no error kind, no position) to a caller holding the decoded
;; `:wat::parse::Lex.cause` string — the SAME information the Rust-side
;; `LexError` still carries in full, discarded at this one serialization
;; site.
;;
;; Loads after `wat/core.wat` (`:wat::core::Error`/`Span`/`String`). See
;; `src/load/stdlib.rs`.

;; ─── The 11 declared `ParseErrorKind` records ────────────────────────────────

;; Lex failure — the input couldn't be tokenized. `cause` is the LexError's
;; own `Display` text, flattened to a plain string (see header note above —
;; the one wire defect this taxonomy carries into strike B).
(:wat::core::defrecord :wat::parse::Lex
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   cause <- :wat::core::String])

;; A `)` was found with no matching `(`.
(:wat::core::defrecord :wat::parse::UnexpectedRParen
  [message <- :wat::core::String
   location <- :wat::core::Span
   ])

;; An opening `(` was never closed before end of input.
(:wat::core::defrecord :wat::parse::UnclosedParen
  [message <- :wat::core::String
   location <- :wat::core::Span
   ])

;; A `]` was found with no matching `[`.
(:wat::core::defrecord :wat::parse::UnexpectedRBracket
  [message <- :wat::core::String
   location <- :wat::core::Span
   ])

;; An opening `[` was never closed before end of input.
(:wat::core::defrecord :wat::parse::UnclosedBracket
  [message <- :wat::core::String
   location <- :wat::core::Span
   ])

;; A `}` was found with no matching `{`.
(:wat::core::defrecord :wat::parse::UnexpectedRBrace
  [message <- :wat::core::String
   location <- :wat::core::Span
   ])

;; An opening `{` was never closed before end of input.
(:wat::core::defrecord :wat::parse::UnclosedBrace
  [message <- :wat::core::String
   location <- :wat::core::Span
   ])

;; A brace-form `{...}` was used as a map literal but violated the pinned
;; shape, or as a struct-destructure but a child wasn't a bare Symbol.
(:wat::core::defrecord :wat::parse::MalformedBraceLiteral
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   reason <- :wat::core::String])

;; `parse_one` expected exactly one form; got trailing content after the
;; first complete form.
(:wat::core::defrecord :wat::parse::TrailingContent
  [message <- :wat::core::String
   location <- :wat::core::Span
   ])

;; `parse_one` expected a form but the input was empty (all whitespace).
(:wat::core::defrecord :wat::parse::Empty
  [message <- :wat::core::String
   location <- :wat::core::Span
   ])

;; A symbol token spelled with the reserved `$bound` namespace segment as its
;; namespace — only the substrate's own binder construction may produce one.
(:wat::core::defrecord :wat::parse::ForgedBinderNamespace
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   spelling <- :wat::core::String])
