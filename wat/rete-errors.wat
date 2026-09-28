;; wat/rete-errors.wat — excursus 003 sweep S3: every `ReteCheckErrorKind` variant
;; (`src/rete/validate/error.rs:23`, 18 measured) and the `ReteCheckErrors`
;; aggregate (`src/rete/validate/error.rs`) become declared wat records.
;;
;; PURE DECLARATION (the sweep's one invariant): each record below mirrors
;; EXACTLY what `ReteCheckError`/`ReteCheckErrors`'s `WatError::error_edn()`
;; (`src/rete/validate/error.rs`) emits on the wire TODAY — same tag, same
;; keys (the floor `message location causes`, then the kind's own fields, in
;; Rust declaration order — every `ReteCheckErrorKind` variant is a plain
;; named-field struct, no tuple variant, no `#[to_edn(key = ...)]` rename),
;; same value shapes. No golden may move; no writer changes. `causes` is
;; always `[]` for every `ReteCheckError` (`ReteCheckError::causes()` is
;; unconditional) — no `ReteCheckErrorKind` variant nests a further typed
;; sub-value the way `EnsureFnInvalidReason` (S1) or `LoadFetchError`/
;; `HashError` (S2) do; every field here is a `String`, `i64` (Rust `usize`),
;; or `Vector<String>` — no untagged map, no flat sum-type sub-record.
;;
;; Tag namespace: `crate::error_ns::RETE` = `"wat.rete"` (`src/error_ns.rs`),
;; so `:wat::rete::<Name>` is exactly `#wat.rete/<Name>` on the wire.
;;
;; This taxonomy reaches the wire through ONE producer
;; (`src/freeze/validator.rs` registers exactly one `FreezeValidator` in the
;; whole tree — the `defrule` wall), boxed as a `StartupError::Validator`
;; behind the `FreezeValidatorError` trait object (`src/macros/error_edn.rs`);
;; the box's `WatError` impl delegates by dynamic dispatch, so the concrete
;; `#wat.rete/…` tag survives untouched.
;;
;; ── The `ReteCheckErrors` aggregate (`#wat.rete/ReteCheckErrors {…}`) ────────
;;
;; `validate_rete_rules`'s batch of findings. Carries NO variant-specific
;; fields beyond the floor (`ReteCheckErrors::variant()` returns an empty
;; map) — every finding lives in `:causes`, each one a fully-floored
;; `#wat.rete/<Variant> {…}` record in its own right. Same shape as
;; `wat/check-errors.wat`'s `CheckErrors`.
(:wat::core::defrecord :wat::rete::ReteCheckErrors
  [message <- :wat::core::String
   location <- :wat::core::Span
   errors <- (:wat::core::Vector :- [:wat::core::Error])])

;; Loads after `wat/core.wat` (`:wat::core::Error`/`Span`/`String`/`i64`/
;; `Vector`). See `src/load/stdlib.rs`.

;; ─── The 18 declared `ReteCheckErrorKind` records ────────────────────────────

;; A `:when` condition's fact-type head, a `:then` fact-form's fact-type head,
;; or an accumulate's `:from` fact-type head, is not a registered aggregate type.
(:wat::core::defrecord :wat::rete::UnknownFactType
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   fact-type <- :wat::core::String])

;; A `:when` clause does not match any recognized rete-DSL shape.
(:wat::core::defrecord :wat::rete::MalformedClause
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   fact-type <- :wat::core::String
   clause <- :wat::core::String])

;; A `(?v :- :field)` bind clause, a constraint operand's `:field` reference,
;; or a `:then` kwargs field name does not name a real field of `fact_type`.
(:wat::core::defrecord :wat::rete::UnknownField
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   fact-type <- :wat::core::String
   field <- :wat::core::String
   available-fields <- (:wat::core::Vector :- [:wat::core::String])])

;; A `::`-qualified keyword CONSTANT in a `:when` constraint whose prefix
;; names a known enum but whose variant that enum does not declare.
(:wat::core::defrecord :wat::rete::UnknownEnumVariant
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   fact-type <- :wat::core::String
   enum-path <- :wat::core::String
   variant <- :wat::core::String
   available-variants <- (:wat::core::Vector :- [:wat::core::String])])

;; A positional `:then` fact-form's argument count does not match the fact
;; type's declared field count (also reused for nested aggregate operands
;; and bare enum-variant heads — see the Rust doc for the full scope).
(:wat::core::defrecord :wat::rete::RhsArityMismatch
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   fact-type <- :wat::core::String
   expected <- :wat::core::i64
   got <- :wat::core::i64])

;; A `:then` fact-form's VALUE-position operand can never resolve at fire
;; time, whatever the bindings.
(:wat::core::defrecord :wat::rete::RhsUnresolvableOperand
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   fact-type <- :wat::core::String
   operand <- :wat::core::String
   accepted <- (:wat::core::Vector :- [:wat::core::String])])

;; A `:then` operand whose DECLARED type does not match the field it lands in.
(:wat::core::defrecord :wat::rete::RhsOperandTypeMismatch
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   fact-type <- :wat::core::String
   field <- :wat::core::String
   declared <- :wat::core::String
   actual <- :wat::core::String])

;; A kwargs `:then` RHS under-supplies the fact type's declared fields.
(:wat::core::defrecord :wat::rete::RhsMissingFields
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   fact-type <- :wat::core::String
   missing <- (:wat::core::Vector :- [:wat::core::String])])

;; A `:then` field VALUE whose type is KNOWABLE and disagrees with the
;; destination field's declared type.
(:wat::core::defrecord :wat::rete::RhsFieldTypeMismatch
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   fact-type <- :wat::core::String
   field <- :wat::core::String
   field-type <- :wat::core::String
   field-rete-type <- :wat::core::String
   operand <- :wat::core::String
   operand-type <- :wat::core::String])

;; A nested surface aggregate-constructor operand written with MORE THAN ONE
;; positional argument — raw positional construction at a bare aggregate
;; name is retired.
(:wat::core::defrecord :wat::rete::RhsPositionalConstructionRetired
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   fact-type <- :wat::core::String
   got <- :wat::core::i64])

;; An inline alpha constraint clause spelled with the GENERIC core comparator
;; instead of a monomorphic rete primitive.
(:wat::core::defrecord :wat::rete::NonReteConstraint
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   fact-type <- :wat::core::String
   head <- :wat::core::String
   twin <- :wat::core::String])

;; A per-type rete constraint applied to a field of a DIFFERENT declared type.
(:wat::core::defrecord :wat::rete::ConstraintTypeMismatch
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   fact-type <- :wat::core::String
   head <- :wat::core::String
   field <- :wat::core::String
   op-type <- :wat::core::String
   field-type <- :wat::core::String])

;; An inline constraint on an operand whose declared type has NO rete
;; comparator at all.
(:wat::core::defrecord :wat::rete::ConstraintTypeNotComparable
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   fact-type <- :wat::core::String
   head <- :wat::core::String
   operand <- :wat::core::String
   field-type <- :wat::core::String])

;; The fence-scoped twin of `ConstraintTypeMismatch` — a predicate inside a
;; `(:wat::rete::where …)` fence at a type mismatch. No `fact_type`: a
;; `where` fence sits at `:when` top level, bound to no fact.
(:wat::core::defrecord :wat::rete::FenceConstraintTypeMismatch
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   head <- :wat::core::String
   operand <- :wat::core::String
   op-type <- :wat::core::String
   operand-type <- :wat::core::String])

;; The fence-scoped twin of `ConstraintTypeNotComparable`.
(:wat::core::defrecord :wat::rete::FenceConstraintTypeNotComparable
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   head <- :wat::core::String
   operand <- :wat::core::String
   operand-type <- :wat::core::String])

;; A `?`-prefixed name used as the BINDER in a fence-local `let`/`match` —
;; would silently shadow the rule-wide rete binding.
(:wat::core::defrecord :wat::rete::FenceBinderShadowsReteVar
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   form <- :wat::core::String
   binder <- :wat::core::String])

;; A `(?v :- :field)` bind inside a `:not` whose variable is consumed NOWHERE.
(:wat::core::defrecord :wat::rete::UnconsumedWrapperBind
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   var <- :wat::core::String
   fact-type <- :wat::core::String])

;; A `(?v :- :field)` bind inside a `:not` whose variable is referenced
;; OUTSIDE the negation, where it provably cannot have a value.
(:wat::core::defrecord :wat::rete::EscapedWrapperBind
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   rule <- :wat::core::String
   var <- :wat::core::String
   fact-type <- :wat::core::String])
