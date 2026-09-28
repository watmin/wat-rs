;; wat/types-errors.wat — excursus 003 sweep S2: every `TypeErrorKind` variant
;; (`src/types/error.rs:76`, 23 measured) becomes a declared wat record.
;;
;; PURE DECLARATION (the sweep's one invariant): each record below mirrors
;; EXACTLY what `TypeError`'s `WatError::error_edn()` (`src/types/error.rs`)
;; emits on the wire TODAY — same tag, same keys (the floor `message location
;; causes`, then the kind's own fields, in `#[derive(ToEdn)]`-produced order:
;; plain field pairs in Rust declaration order — no variant here carries a
;; literal-prepended or computed `via` field), same value shapes. No golden
;; may move; no writer changes. `causes` is always `[]` for every `TypeError`
;; (`TypeError::causes()` is unconditional, `src/types/error.rs:470`) — no
;; `TypeErrorKind` variant nests a further typed error.
;;
;; Tag namespace: `crate::error_ns::TYPE` = `"wat.type"` (`src/error_ns.rs`),
;; so `:wat::type::<Name>` is exactly `#wat.type/<Name>` on the wire — the
;; SAME flat, one-tag-per-variant shape `wat/check-errors.wat` used for
;; `CheckErrorKind` (no `defenum` wrapper; each variant IS its own record).
;;
;; `usize` fields (`expected`/`got`/`field_count`/`budget`) type
;; `:wat::core::i64` (`wat_edn::ToEdn` derive's own mapping,
;; `crates/wat-to-edn-derive/src/lib.rs:183`).
;;
;; `MalformedVariant.remedies` reuses `:wat::kernel::Remedy`
;; (`wat/kernel/diagnostics.wat`, declared by S1) — the same structured
;; remediation-candidate shape `CheckErrorKind::TypeMismatch`/
;; `ReturnTypeMismatch`/`MalformedForm` already embed.
;;
;; Loads after `wat/core.wat` (`:wat::core::Error`/`Span`/`String`/`i64`/
;; `Option`/`Vector`) and after `wat/kernel/diagnostics.wat` (needs
;; `:wat::kernel::Remedy`). See `src/load/stdlib.rs`.

;; Arc 138 slice 2 — a second type declaration reused an already-registered
;; name. `name` is the OFFENDING (new) declaration's name.
(:wat::core::defrecord :wat::type::DuplicateType
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   name <- :wat::core::String])

;; Arc 138 slice 2 — a type name used a reserved prefix.
(:wat::core::defrecord :wat::type::ReservedPrefix
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   name <- :wat::core::String])

;; A top-level type name reached a registration gate with no namespace.
(:wat::core::defrecord :wat::type::UnnamespacedName
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   name <- :wat::core::String])

;; Arc 296 stone H-1 — a type name's segment after the last `::` contains a
;; `.`, the wire discriminator for a tagged-enum variant.
(:wat::core::defrecord :wat::type::DottedName
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   name <- :wat::core::String])

;; Arc 138 slice 2 — a malformed type-declaration form (wrong outer shape).
(:wat::core::defrecord :wat::type::MalformedDecl
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   head <- :wat::core::String
   reason <- :wat::core::String])

;; Arc 138 slice 2 — a malformed type name keyword.
(:wat::core::defrecord :wat::type::MalformedName
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   raw <- :wat::core::String
   reason <- :wat::core::String])

;; Arc 138 slice 2 — a malformed field item inside a type declaration.
(:wat::core::defrecord :wat::type::MalformedField
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   reason <- :wat::core::String])

;; Arc 130 follow-up / Stone 241.10 — a malformed enum variant, with ranked
;; structured remediation candidates (empty vec = no remedy offered).
(:wat::core::defrecord :wat::type::MalformedVariant
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   enum-name <- :wat::core::String
   offending <- :wat::core::String
   reason <- :wat::core::String
   remedies <- (:wat::core::Vector :- [:wat::kernel::Remedy])])

;; Arc 138 slice 2 — a malformed type expression.
(:wat::core::defrecord :wat::type::MalformedTypeExpr
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   raw <- :wat::core::String
   reason <- :wat::core::String])

;; 058-030 — user source wrote the banned `:Any` escape hatch.
(:wat::core::defrecord :wat::type::AnyBanned
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   raw <- :wat::core::String])

;; A typealias's expansion, traced through the currently-registered aliases,
;; reaches the alias's own name.
(:wat::core::defrecord :wat::type::CyclicAlias
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   name <- :wat::core::String])

;; A parametric typealias was referenced with the wrong number of type
;; arguments.
(:wat::core::defrecord :wat::type::AliasArityMismatch
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   name <- :wat::core::String
   expected <- :wat::core::i64
   got <- :wat::core::i64])

;; Arc 115 slice 2 — a type argument inside a compound (`<>`, `()`, `fn(...)`,
;; fn return after `->`) carried an illegal leading `:` it shouldn't.
(:wat::core::defrecord :wat::type::InnerColonInCompoundArg
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   raw <- :wat::core::String
   offending <- :wat::core::String])

;; Stone 237.1 — a typeunion's member graph closes a cycle.
(:wat::core::defrecord :wat::type::CyclicUnion
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   name <- :wat::core::String])

;; Stone 237.1 — a typeunion was declared with zero members.
(:wat::core::defrecord :wat::type::EmptyUnion
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   name <- :wat::core::String])

;; Stone 237.1 — a typeunion was declared with exactly one member (a
;; typealias in disguise).
(:wat::core::defrecord :wat::type::SingleMemberUnion
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   name <- :wat::core::String])

;; Stone 237.1 — a typeunion's member list contained a shape typeunion does
;; not accept (only Path, Parametric, Tuple are sound).
(:wat::core::defrecord :wat::type::InvalidUnionMember
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   union-name <- :wat::core::String
   member-form <- :wat::core::String
   reason <- :wat::core::String])

;; Stone S-A — a `register_subtype(child, parent, span)` call would close a
;; cycle in the typesub hierarchy.
(:wat::core::defrecord :wat::type::CyclicSubtype
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   child <- :wat::core::String
   parent <- :wat::core::String])

;; Arc 293.W — a portable aggregate (Record | HolonRecord) declared a field
;; whose type is non-portable (e.g. a Struct) — the containment rule.
(:wat::core::defrecord :wat::type::ImpureFieldInPureAggregate
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   aggregate <- :wat::core::String
   field <- :wat::core::String
   field-ty <- :wat::core::String])

;; Arc 293.W.2b — the enum counterpart of the containment rule: a `Pure` enum
;; declared an impure variant field.
(:wat::core::defrecord :wat::type::ImpureVariantFieldInPureEnum
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   enum-name <- :wat::core::String
   variant <- :wat::core::String
   field <- :wat::core::String
   field-ty <- :wat::core::String])

;; BRIEF-construction-inside-a-fn.md gap (b) — a `HolonRecord` aggregate's own
;; declared field count exceeds the encoding budget at the frozen dimension.
(:wat::core::defrecord :wat::type::HolonRecordCapacityExceeded
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   aggregate <- :wat::core::String
   field-count <- :wat::core::i64
   budget <- :wat::core::i64])

;; DESIGN-STONE-a-param-spec-must-be-consumed — a type declaration's
;; param-spec named a type parameter no member type reaches.
(:wat::core::defrecord :wat::type::UnconsumedTypeParam
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   decl <- :wat::core::String
   param <- :wat::core::String])

;; Arc 296 P-1 — an annotation named a type that does not exist.
(:wat::core::defrecord :wat::type::UnknownNamedType
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   path <- :wat::core::String])
