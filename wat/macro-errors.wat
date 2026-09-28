;; wat/macro-errors.wat — excursus 003 sweep S3: every `MacroErrorKind` variant
;; (`src/macros/error.rs:42`, 16 measured) becomes a declared wat record.
;;
;; PURE DECLARATION (the sweep's one invariant): each record below mirrors
;; EXACTLY what `MacroError`'s `WatError::error_edn()` (`src/macros/error_edn.rs`)
;; emits on the wire TODAY — same tag, same keys (the floor `message location
;; causes`, then the kind's own fields, in the `#[derive(ToEdn)]`-produced
;; order: a tuple variant's single field under its `#[to_edn(key = "name")]`
;; rename, or plain field pairs in Rust declaration order), same value
;; shapes. No golden may move; no writer changes. `causes` is always `[]`
;; for every `MacroError` (`MacroError::causes()` is unconditional) — the two
;; nested-cause variants below (`ProgramBodyEvalFailed`, `MacroEvalRuntimeFailed`)
;; carry their cause as a NAMED FIELD (`cause <- :wat::core::Error`), not via
;; the floor `:causes`, exactly as `wat/eval.wat`'s `:CheckFailed [cause <-
;; :wat::core::Error]` and `wat/load-errors.wat`'s `Parse.cause` already do.
;;
;; Tag namespace: `crate::error_ns::MACRO` = `"wat.macro"` (`src/error_ns.rs`),
;; so `:wat::macro::<Name>` is exactly `#wat.macro/<Name>` on the wire.
;;
;; `ProgramBodyEvalFailed.cause` is `Box<MacroError>` — SELF-referential
;; (this very taxonomy). `:wat::core::Error` is the structural surface, not a
;; nominal `:wat::macro::MacroError` type (no such record exists; `MacroError`
;; is span + kind, never itself tagged as one wire value) — any one of THIS
;; file's own declared records satisfies it once declared, which is exactly
;; why the field types the structural surface rather than a name that could
;; not exist yet at declaration time.
;;
;; `MacroEvalRuntimeFailed.cause` is `Box<RuntimeError>` — `RuntimeErrorKind`
;; was fully declared in `wat/runtime-errors.wat` (step 3a, before this
;; excursus), so this field already decodes typed without any new work here.
;;
;; Loads after `wat/core.wat` (`:wat::core::Error`/`Span`/`String`/`i64`) and
;; after `wat/runtime-errors.wat` (`MacroEvalRuntimeFailed.cause`'s
;; `RuntimeErrorKind` family). See `src/load/stdlib.rs`.

;; ─── The 16 declared `MacroErrorKind` records ────────────────────────────────

;; Two `(:wat::core::defmacro ...)` forms registered the same name. Tuple
;; variant, `#[to_edn(key = "name")]`.
(:wat::core::defrecord :wat::macro::DuplicateMacro
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   name <- :wat::core::String])

;; A user macro declared under a reserved `:wat::...` prefix.
(:wat::core::defrecord :wat::macro::ReservedPrefix
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   name <- :wat::core::String])

;; A macro name reached the registration gate with no namespace.
(:wat::core::defrecord :wat::macro::UnnamespacedName
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   name <- :wat::core::String])

;; A macro name reached the registration gate with a `.` in its name segment
;; — reserved: a dot in a tag's NAME half means "this is an enum variant".
(:wat::core::defrecord :wat::macro::DottedName
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   name <- :wat::core::String])

;; A `defmacro` form was malformed.
(:wat::core::defrecord :wat::macro::MalformedDefmacro
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   reason <- :wat::core::String])

;; A macro call passed the wrong number of arguments (fixed-arity macro).
(:wat::core::defrecord :wat::macro::ArityMismatch
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   name <- :wat::core::String
   expected <- :wat::core::i64
   got <- :wat::core::i64])

;; A variadic macro call passed too few arguments (fewer than the required
;; fixed-param minimum).
(:wat::core::defrecord :wat::macro::ArityTooFew
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   name <- :wat::core::String
   minimum <- :wat::core::i64
   got <- :wat::core::i64])

;; An `unquote` reference named a parameter the macro didn't declare.
(:wat::core::defrecord :wat::macro::UnboundMacroParam
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   name <- :wat::core::String])

;; `unquote-splicing` was applied to a non-sequence argument. `got` is the
;; offending shape's name (`&'static str`, e.g. `"String"`), wire-typed
;; identically to an owned String.
(:wat::core::defrecord :wat::macro::SpliceNotSequence
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   name <- :wat::core::String
   got <- :wat::core::String])

;; Expansion depth exceeded a sanity limit — probably an infinite recursive
;; macro.
(:wat::core::defrecord :wat::macro::ExpansionDepthExceeded
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   limit <- :wat::core::i64])

;; Other malformation in a macro invocation or template.
(:wat::core::defrecord :wat::macro::MalformedTemplate
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   reason <- :wat::core::String])

;; A computed-unquote expression named a keyword head that is not on the
;; blessed pure-combinator allow-list.
(:wat::core::defrecord :wat::macro::RefusedInMacro
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   head <- :wat::core::String])

;; A call head whose registry entry declares `ExpandTime::ExpandOnly` was
;; found in PROGRAM code — the mirror of `RefusedInMacro`.
(:wat::core::defrecord :wat::macro::ExpandOnlyOutsideMacro
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   head <- :wat::core::String])

;; A program-body quasiquote template introduces a literal name in a binder
;; position, which could capture caller-site names silently.
(:wat::core::defrecord :wat::macro::ProgramBodyIntroducesName
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   macro-name <- :wat::core::String
   binder <- :wat::core::String])

;; Macro program-body evaluation failed with a nested `MacroError`. `cause`
;; carries the FULL recursive floor (`error_edn_of_boxed`), self-referential
;; to this taxonomy.
(:wat::core::defrecord :wat::macro::ProgramBodyEvalFailed
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   macro-name <- :wat::core::String
   cause <- :wat::core::Error])

;; Runtime `eval` in macro-eval context failed with a `RuntimeError`. `cause`
;; carries the FULL recursive floor (`error_edn_of_boxed`) — a
;; `RuntimeErrorKind` family record, declared in `wat/runtime-errors.wat`.
(:wat::core::defrecord :wat::macro::MacroEvalRuntimeFailed
  [message <- :wat::core::String
   location <- :wat::core::Span
   
   cause <- :wat::core::Error])
