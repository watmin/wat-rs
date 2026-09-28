;; wat/stdlib-errors.wat — excursus 003 sweep S2: the `StdlibErrorKind`
;; taxonomy (`src/load/stdlib.rs:716`, 1 measured) becomes a declared wat
;; record.
;;
;; PURE DECLARATION (the sweep's one invariant): the record below mirrors
;; EXACTLY what `StdlibError`'s `WatError::error_edn()` (`src/load/stdlib.rs`)
;; emits on the wire TODAY — same tag, same keys (the floor `message location
;; causes`, then the kind's own fields), same value shapes. No golden may
;; move; no writer changes.
;;
;; Tag namespace: `crate::error_ns::STDLIB` = `"wat.stdlib"`
;; (`src/error_ns.rs`), so `:wat::stdlib::<Name>` is exactly
;; `#wat.stdlib/<Name>` on the wire.
;;
;; `ParseFailed { path, cause: ParseError }`
;; (`#[to_edn(via = crate::edn::contract::error_edn_of)]`) recurses through
;; the nested error's own `error_edn()` — the full floor
;; (`message location causes`) plus its own kind fields — exactly like
;; `wat/load-errors.wat`'s `Parse.cause`. `ParseError`
;; (`crates/wat-reader/src/parser.rs:37`) is S3's taxonomy, undeclared here;
;; `cause` types `:wat::core::Error`, the same sanctioned slot
;; `wat/eval.wat`'s `:CheckFailed [cause <- :wat::core::Error]` and
;; `wat/load-errors.wat`'s `Parse.cause` already use.
;;
;; Loads after `wat/core.wat`. See `src/load/stdlib.rs`.

;; A baked stdlib source file failed to parse. Should never fire in
;; production — the stdlib is authored in-repo and its parsing is validated
;; by the test suite — but surfaces cleanly if a stdlib file is malformed.
(:wat::core::defrecord :wat::stdlib::ParseFailed
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   path <- :wat::core::String
   cause <- :wat::core::Error])
