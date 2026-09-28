;; wat/config-errors.wat — excursus 003 sweep S2: every `ConfigErrorKind`
;; variant (`src/config.rs:222`, 8 measured) becomes a declared wat record.
;;
;; PURE DECLARATION (the sweep's one invariant): each record below mirrors
;; EXACTLY what `ConfigError`'s `WatError::error_edn()` (`src/config.rs`)
;; emits on the wire TODAY — same tag, same keys (the floor `message location
;; causes`, then the kind's own fields, in `#[derive(ToEdn)]`-produced order:
;; plain field pairs in Rust declaration order, no `#[to_edn]` field
;; attributes on this taxonomy at all), same value shapes. No golden may
;; move; no writer changes. `causes` is always `[]` for every `ConfigError`
;; (`ConfigError::causes()` is unconditional) — no `ConfigErrorKind` variant
;; nests a further typed error.
;;
;; Tag namespace: `crate::error_ns::CONFIG` = `"wat.config"`
;; (`src/error_ns.rs`), so `:wat::config::<Name>` is exactly
;; `#wat.config/<Name>` on the wire.
;;
;; `expected`/`got` (`BadArity`) are `usize` → `:wat::core::i64`
;; (`wat_edn::ToEdn` derive's own mapping). `expected`/`got` (`BadType`) are
;; `&'static str` → `:wat::core::String` (the blanket `&T`/`str` impl,
;; matching `HashError.InvalidBase64.field`'s precedent).
;;
;; Loads after `wat/core.wat`. See `src/load/stdlib.rs`.

;; A `set-*!` form appeared after a non-setter form in the entry file.
;; Entry-file discipline: all setters precede all other forms.
(:wat::core::defrecord :wat::config::SetterAfterNonSetter
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   setter-head <- :wat::core::String])

;; The same config field was set more than once.
(:wat::core::defrecord :wat::config::DuplicateField
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   field <- :wat::core::String])

;; A required field (`dims`, `capacity-mode`) was not set.
(:wat::core::defrecord :wat::config::RequiredFieldMissing
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   field <- :wat::core::String])

;; A setter head didn't match any known `:wat::config::set-*!`.
(:wat::core::defrecord :wat::config::UnknownSetter
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   head <- :wat::core::String])

;; A setter was called with the wrong number of arguments.
(:wat::core::defrecord :wat::config::BadArity
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   head <- :wat::core::String
   expected <- :wat::core::i64
   got <- :wat::core::i64])

;; A setter's argument was the wrong kind of WatAST.
(:wat::core::defrecord :wat::config::BadType
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   field <- :wat::core::String
   expected <- :wat::core::String
   got <- :wat::core::String])

;; A setter's argument was well-typed but out-of-range / not a recognized
;; variant.
(:wat::core::defrecord :wat::config::BadValue
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   field <- :wat::core::String
   reason <- :wat::core::String])

;; A setter form was malformed (empty list, head not a keyword). Unit
;; variant; every field is the floor.
(:wat::core::defrecord :wat::config::MalformedSetter
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])])
