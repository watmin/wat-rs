;; wat/load-errors.wat — excursus 003 sweep S2: every `LoadErrorKind` variant
;; (`src/load/loader.rs:295`, 8 measured) becomes a declared wat record.
;;
;; PURE DECLARATION (the sweep's one invariant): each record below mirrors
;; EXACTLY what `LoadError`'s `WatError::error_edn()` (`src/load/loader.rs`)
;; emits on the wire TODAY — same tag, same keys (the floor `message location
;; causes`, then the kind's own fields), same value shapes. No golden may
;; move; no writer changes. `causes` is always `[]` for every `LoadError`
;; (`LoadError::causes()` is unconditional).
;;
;; Tag namespace: `crate::error_ns::LOAD` = `"wat.load"` (`src/error_ns.rs`),
;; so `:wat::load::<Name>` is exactly `#wat.load/<Name>` on the wire.
;;
;; Two variants rename their sole nested-error field to the EDN key `cause`
;; (`#[to_edn(key = "cause")]`, `src/load/loader.rs:324,333,340`):
;;
;; - `Fetch(LoadFetchError)` — the payload is a genuine sum type
;;   (`src/load/loader.rs:192`) whose hand-written `ToEdn` impl tags each
;;   variant FLAT under `wat.kernel` (`edn_tag`, same shape as S1's
;;   `EnsureFnInvalidReason`) — declared as three independent flat records in
;;   `wat/kernel/diagnostics.wat` (`:wat::kernel::NotFound`/`LoadOther`/
;;   `OutOfScope`). `cause` types `:wat::core::Value` here: no common shape
;;   across the three, decode is tag-driven regardless of the declared field
;;   type (S1's `EnsureFnInvalid.reason` precedent).
;; - `VerificationFailed { path, cause: HashError }` — `HashError`
;;   (`src/hash.rs:471`) is ALSO a genuine sum type with the SAME flat
;;   `wat.kernel` per-variant tagging (`edn_tag`) — declared as eight
;;   independent flat records in `wat/kernel/diagnostics.wat`. `cause` types
;;   `:wat::core::Value` for the same reason.
;;
;; `Parse { path, cause: ParseError }` (`#[to_edn(key = "cause")]`
;; `#[to_edn(via = crate::edn::contract::error_edn_of)]`) recurses through the
;; NESTED error's own `error_edn()` — the full floor
;; (`message location causes`) plus ITS OWN kind fields. `ParseError`
;; (`crates/wat-reader/src/parser.rs:37`) is S3's taxonomy, undeclared here;
;; `cause` types `:wat::core::Error` — the sanctioned "a further, not-yet-
;; concretely-named, Error-conforming value" slot, the SAME shape
;; `wat/eval.wat`'s `:CheckFailed [cause <- :wat::core::Error]` already uses
;; for a foreign nested diagnostic. When S3 declares `:wat::parse::*`, this
;; field silently upgrades from foreign to typed for that nested tree — no
;; change needed here.
;;
;; Loads after `wat/core.wat` and after `wat/kernel/diagnostics.wat` (needs
;; `:wat::kernel::NotFound`/`LoadOther`/`OutOfScope`/`UnsupportedAlgorithm`/
;; `Mismatch`/`UnsupportedSignatureAlgorithm`/`InvalidBase64`/
;; `InvalidSignatureLength`/`InvalidPubKeyLength`/`InvalidPubKey`/
;; `SignatureMismatch`). See `src/load/stdlib.rs`.

;; The `load!` form was malformed — wrong arity, wrong interface keyword,
;; wrong value type, unknown verification algorithm, etc.
(:wat::core::defrecord :wat::load::MalformedLoadForm
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   reason <- :wat::core::String])

;; Excursus 003 D4 item 1 — the fetched (or entry) file's display label is
;; exactly one of the baked stdlib's own reserved path labels.
(:wat::core::defrecord :wat::load::ReservedStdlibLabel
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   label <- :wat::core::String])

;; A loaded (non-entry) file contained a `(:wat::config::set-*!)` form.
;; Entry-file discipline: setters belong to the entry file only.
(:wat::core::defrecord :wat::load::SetterInLoadedFile
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   loaded-path <- :wat::core::String
   setter-head <- :wat::core::String])

;; The same path was loaded twice (a transitive duplicate or direct repeat).
(:wat::core::defrecord :wat::load::DuplicateLoad
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   path <- :wat::core::String])

;; A load chain closed back on itself (A loads B loads A). `cycle` is the
;; full chain of canonical paths that formed the loop.
(:wat::core::defrecord :wat::load::CycleDetected
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   cycle <- (:wat::core::Vector :- [:wat::core::String])])

;; The loader couldn't fetch the file. `cause` is one of the three flat
;; `LoadFetchError` records declared in `wat/kernel/diagnostics.wat` (see this
;; file's header).
(:wat::core::defrecord :wat::load::Fetch
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   cause <- :wat::core::Value])

;; Parsing the fetched source failed. `cause` is the nested `ParseError`'s own
;; full `error_edn()` (S3's taxonomy; see this file's header).
(:wat::core::defrecord :wat::load::Parse
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   path <- :wat::core::String
   cause <- :wat::core::Error])

;; Cryptographic verification of the loaded source failed. `cause` is one of
;; the eight flat `HashError` records declared in `wat/kernel/diagnostics.wat`
;; (see this file's header).
(:wat::core::defrecord :wat::load::VerificationFailed
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   path <- :wat::core::String
   cause <- :wat::core::Value])
