;; wat/check-errors.wat — excursus 003 sweep S1: every `CheckErrorKind` variant
;; (`src/check/error.rs:87`, 34 measured) and the `CheckErrors` aggregate
;; (`src/check/error.rs:906`) become declared wat records.
;;
;; PURE DECLARATION (the sweep's one invariant): each record below mirrors
;; EXACTLY what `CheckError`/`CheckErrors`'s `WatError::error_edn()`
;; (`src/check/error_edn.rs`) emits on the wire TODAY — same tag, same keys
;; (the floor `message location causes`, then the kind's own fields, in their
;; `#[derive(ToEdn)]`-produced order: literal-prepended fields, then plain
;; field pairs in Rust declaration order, then a variant-level computed `via`
;; field appended last), same value shapes. No golden may move; no writer
;; changes. `causes` is always `[]` for every `CheckError` (`CheckError::
;; causes()` is unconditional, `src/check/error_edn.rs:58`) — no CheckErrorKind
;; variant nests a further typed error the way some `RuntimeErrorKind`
;; variants do, so unlike `wat/runtime-errors.wat` nothing here moves a field
;; into `causes`.
;;
;; Tag namespace: `crate::error_ns::CHECK` = `"wat.check"` (`src/error_ns.rs:9`),
;; so `:wat::check::<Name>` is exactly `#wat.check/<Name>` on the wire — the
;; SAME flat, one-tag-per-variant shape `wat/runtime-errors.wat` used for
;; `RuntimeErrorKind`'s 40 variants (no `defenum` wrapper for the top-level
;; taxonomy; each variant IS its own record).
;;
;; Loads after `wat/core.wat` (`:wat::core::Error`/`Span`/`String`/`i64`/
;; `keyword`/`Option`/`Vector`) and after `wat/kernel/diagnostics.wat` (needs
;; `:wat::kernel::Remedy`, declared there per this sweep — see that file's
;; header for why). See `src/load/stdlib.rs`.
;;
;; ── STOP: one untagged record-shaped map on today's wire, NOT declared ──────
;;
;; `NoMatchingClauseAtCallSite.attempted-clauses` (`src/check.rs`'s
;; `clause_attempts_to_edn`, routed via `#[to_edn(via = ...)]`) renders each
;; attempt as a BARE, UNTAGGED `{:arity N :param-types [...]}` map inside a
;; Vector — no `Tag` at all. The builder has ruled record-shaped values are
;; always tagged; this one, today, is not. Per the brief: do not declare
;; around it (tagging it would change the wire, strike B's business) — the
;; wat type universe also has no honest type for it: `:Any` is banned
;; (`src/types.rs:86`, "the type universe is closed"), and the map's two keys
;; hold different-shaped values (`i64` vs `Vector<String>`), so no homogeneous
;; `HashMap<K,V>` fits either. `NoMatchingClauseAtCallSite` is therefore the
;; ONE CheckErrorKind variant with NO record declared below (33 of 34
;; declared) — G-list's ruled exception, named by (variant, field) the same
;; way step 3a's G2 named its exceptions.
;;
;; ── The nested `EnsureFnInvalidReason` payload ──────────────────────────────
;;
;; `EnsureFnInvalid.reason` is typed `:wat::core::Value` below, not a nominal
;; record or a `defenum`. See `wat/kernel/diagnostics.wat`'s header for why:
;; the type universe has no union/sum type, `EnsureFnInvalidReason`'s five
;; variants share no common shape (no defsurface fits), and each variant tags
;; FLAT (`#wat.kernel/<Variant>`, the derive's default namespace with no dotted
;; enum-name prefix) — so a `defenum` here would register under a path the
;; general decoder's undotted-tag lookup never consults. `:wat::core::Value`
;; is the sanctioned universal-top slot for a genuinely polymorphic field
;; (unlike the banned `:Any`); decode is tag-driven regardless of the declared
;; field type (`reconstruct_struct`, `src/edn/render.rs`, consults `fty` only
;; for Option-rewrapping), so this is an honest declaration, not a workaround.

;; ─── The `CheckErrors` aggregate (`#wat.check/CheckErrors {…}`) ──────────────
;;
;; `check_program`'s batch of findings. Carries NO variant-specific fields
;; beyond the floor (`CheckErrors::variant()` returns an empty map,
;; `src/check/error_edn.rs:117`) — every finding lives in `:causes`, each one
;; a fully-floored `#wat.check/<Variant> {…}` record in its own right (never
;; the retired bespoke `:errors` key).
(:wat::core::defrecord :wat::check::CheckErrors
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])])

;; ─── The 33 declared `CheckErrorKind` records (34 minus the STOP above) ──────

;; Arc 138 slice 1 — a call site passed the wrong number of arguments.
(:wat::core::defrecord :wat::check::ArityMismatch
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   callee <- :wat::core::String
   expected <- :wat::core::i64
   got <- :wat::core::i64])

;; Arc 138 slice 1 — a call-site parameter's runtime type did not match. `remedies`
;; is computed at serialize time from `callee` (`type_error_remedies_via`),
;; always emitted (possibly empty).
(:wat::core::defrecord :wat::check::TypeMismatch
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   callee <- :wat::core::String
   param <- :wat::core::String
   expected <- :wat::core::String
   got <- :wat::core::String
   remedies <- (:wat::core::Vector :- [:wat::kernel::Remedy])])

;; Arc 138 slice 1 — a function body's type does not match its declared return
;; type. `remedies` is the stored typo candidates merged with `type_error_
;; remedies(function)` at serialize time (`return_type_remedies_via`), always
;; emitted (possibly empty); the raw `remedies` struct field is `#[to_edn(skip)]`.
(:wat::core::defrecord :wat::check::ReturnTypeMismatch
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   function <- :wat::core::String
   expected <- :wat::core::String
   got <- :wat::core::String
   remedies <- (:wat::core::Vector :- [:wat::kernel::Remedy])])

;; Arc 138 slice 1 — an unknown callee at a call site.
(:wat::core::defrecord :wat::check::UnknownCallee
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   callee <- :wat::core::String])

;; A built-in form is structurally malformed in a way the syntax grammar
;; doesn't catch. `remedies` is a stored field (not computed), always emitted
;; (possibly empty).
(:wat::core::defrecord :wat::check::MalformedForm
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   head <- :wat::core::String
   reason <- :wat::core::String
   remedies <- (:wat::core::Vector :- [:wat::kernel::Remedy])])

;; Arc 110 — a comm-call (`send`/`recv`) appeared outside every permitted
;; position.
(:wat::core::defrecord :wat::check::CommCallOutOfPosition
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   callee <- :wat::core::String])

;; Arc 170 — a Process output-channel accessor call conflicts with a
;; join-before-drain in the same `let` scope.
(:wat::core::defrecord :wat::check::ProcessJoinBeforeOutputDrain
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   process-identifier <- :wat::core::String
   output-accessor <- :wat::core::String
;; Source location of the conflicting output accessor call. Key `output-location`.
   output-location <- :wat::core::Span])

;; Arc 202 — a Process input-channel sender is held live across a join in the
;; same `let` scope.
(:wat::core::defrecord :wat::check::ProcessJoinHoldsStdinSender
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   process-identifier <- :wat::core::String
;; Source location where `<process-identifier>` was bound. Key `bind-location`.
   bind-location <- :wat::core::Span])

;; Arc 109 slice 1c — a bare (non-FQDN) primitive type name in user code.
(:wat::core::defrecord :wat::check::BareLegacyPrimitive
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   primitive <- :wat::core::String
   fqdn <- :wat::core::String])

;; Arc 109 slice 1d — the bare unit type annotation `:()` (retired; use
;; `:wat::core::nil`). Unit variant; `primitive`/`fqdn` are synthetic literals.
(:wat::core::defrecord :wat::check::BareLegacyUnitType
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   primitive <- :wat::core::String
   fqdn <- :wat::core::String])

;; Arc 179 — the bare `()` empty-list literal in VALUE position (retired; `nil`
;; is the sole unit value). Unit variant; `retired`/`fqdn` are synthetic literals.
(:wat::core::defrecord :wat::check::BareLegacyUnitValue
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   retired <- :wat::core::String
   fqdn <- :wat::core::String])

;; Arc 153 — `:wat::core::unit` (retired in favor of `:wat::core::nil`). Unit
;; variant; `retired`/`fqdn` are synthetic literals.
(:wat::core::defrecord :wat::check::BareLegacyUnitName
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   retired <- :wat::core::String
   fqdn <- :wat::core::String])

;; Arc 154 — `:wat::core::let*` (retired). Unit variant; `retired`/`fqdn` are
;; synthetic literals.
(:wat::core::defrecord :wat::check::BareLegacyLetStar
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   retired <- :wat::core::String
   fqdn <- :wat::core::String])

;; Arc 155 — `:wat::core::lambda` (retired). Unit variant; `retired`/`fqdn`
;; are synthetic literals.
(:wat::core::defrecord :wat::check::BareLegacyLambda
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   retired <- :wat::core::String
   fqdn <- :wat::core::String])

;; Arc 155 — the bare `:fn(...)->ret` type-position spelling (retired). Unit
;; variant; `retired`/`fqdn` are synthetic literals.
(:wat::core::defrecord :wat::check::BareLegacyLowercaseFn
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   retired <- :wat::core::String
   fqdn <- :wat::core::String])

;; Arc 109 slice 1e — a bare substrate-named parametric type head.
(:wat::core::defrecord :wat::check::BareLegacyContainerHead
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   head <- :wat::core::String
   fqdn <- :wat::core::String])

;; Arc 109 slice 9d — the legacy `:wat::std::stream::` prefix.
(:wat::core::defrecord :wat::check::BareLegacyStreamPath
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   old <- :wat::core::String
   new <- :wat::core::String])

;; Arc 109 slice K.lru — the legacy `:wat::lru::CacheService::` prefix.
(:wat::core::defrecord :wat::check::BareLegacyLruCacheServicePath
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   old <- :wat::core::String
   new <- :wat::core::String])

;; Arc 109 slice K.kernel-channel — the legacy `:wat::kernel::Queue*` names.
(:wat::core::defrecord :wat::check::BareLegacyKernelQueuePath
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   old <- :wat::core::String
   new <- :wat::core::String])

;; Arc 157 — a `:wat::core::def` redefinition of an already-bound name.
(:wat::core::defrecord :wat::check::DefRedefForbidden
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   name <- :wat::core::String
;; Source location of the prior (first) binding. Key `prior-loc`.
   prior-loc <- :wat::core::Span])

;; Arc 157 slice 1a-ii — a `:wat::core::def` redefinition that also changes type.
(:wat::core::defrecord :wat::check::DefRedefTypeChange
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   name <- :wat::core::String
   prior-type <- :wat::core::String
   new-type <- :wat::core::String
;; Source location of the prior (first) binding. Key `prior-loc`.
   prior-loc <- :wat::core::Span])

;; Arc 278 BRIEF-scalar-def-reaches-the-gate — a top-level `def` with no
;; namespace on its name.
(:wat::core::defrecord :wat::check::UnnamespacedName
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   name <- :wat::core::String])

;; Arc 278 BRIEF-scalar-def-reaches-the-gate — a top-level `def` using a
;; reserved namespace prefix.
(:wat::core::defrecord :wat::check::ReservedPrefix
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   name <- :wat::core::String])

;; Arc 296 stone H-1 — a registered name's last segment contains a `.`, the
;; wire discriminator for a tagged-enum variant.
(:wat::core::defrecord :wat::check::DottedName
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   name <- :wat::core::String])

;; Arc 296 stone I — a name landed on `CheckEnv`'s overlay scheme table
;; divergent from what is already registered there.
(:wat::core::defrecord :wat::check::DuplicateScheme
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   name <- :wat::core::String])

;; Arc 170 slice 1e — `:user::main` declared with a non-canonical signature.
;; Unit variant; every field below is a synthetic literal.
(:wat::core::defrecord :wat::check::BareLegacyMainSignature
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   canonical-signature <- :wat::core::String
   rationale <- :wat::core::String])

;; Arc 109 § kill-std — the retired `:wat::console::*` namespace. The first
;; four fields are synthetic literals; `offending-token` is the real field
;; (Rust name `path`).
(:wat::core::defrecord :wat::check::BareLegacyConsolePath
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   retired-namespace <- :wat::core::String
   canonical-stdout <- :wat::core::String
   canonical-stderr <- :wat::core::String
   canonical-stdin <- :wat::core::String
   offending-token <- :wat::core::String])

;; Stone 241.14 — a restricted-caller whitelist violation.
(:wat::core::defrecord :wat::check::DefRestrictedCallerNotAllowed
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   callee <- :wat::core::String
   enclosing-fn <- :wat::core::String
   prefixes <- (:wat::core::Vector :- [:wat::core::String])])

;; STOP — `:wat::check::NoMatchingClauseAtCallSite` (stone 237.2) is NOT
;; declared. Its `attempted-clauses` field renders each attempt as a bare,
;; UNTAGGED `{:arity N :param-types [...]}` map (`clause_attempts_to_edn`,
;; `src/check.rs`) — the exact "record-shaped values are always tagged"
;; violation the file header's STOP section names. 33 of 34 CheckErrorKind
;; kinds are declared here; this is the one ruled exception (G-list).

;; Arc <post-278> — an open-surface `defclause` dispatch matched multiple
;; narrowing clauses whose declared return types do not unify.
(:wat::core::defrecord :wat::check::AmbiguousClauseReturnAtCallSite
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   name <- :wat::core::String
   called-arg-types <- (:wat::core::Vector :- [:wat::core::String])
   candidate-returns <- (:wat::core::Vector :- [:wat::core::String])])

;; Stone 237.3 — a defclause's `:guard` expression is not boolean.
(:wat::core::defrecord :wat::check::GuardExprNotBoolean
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   defclause-name <- :wat::core::String
   clause-index <- :wat::core::i64
   got-type <- :wat::core::String])

;; Stone 237.3 — a defclause's `:ensure :fn` is structurally invalid. `reason`
;; is one of the five `EnsureFnInvalidReason` records
;; (`wat/kernel/diagnostics.wat`); typed `:wat::core::Value` here — see this
;; file's header.
(:wat::core::defrecord :wat::check::EnsureFnInvalid
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   defclause-name <- :wat::core::String
   clause-index <- :wat::core::i64
   reason <- :wat::core::Value])

;; Arc 291 — hygiene-scope divergence: a reference is unbound, but a binder of
;; the same name exists under a different hygiene scope.
(:wat::core::defrecord :wat::check::HygieneScopeDivergence
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   name <- :wat::core::String
   ref-key <- :wat::core::String
   binder-key <- :wat::core::String])

;; Arc 278 BRIEF-arming-is-internal-only — a `:wat::service::Alarm`'s `op` was
;; built from a PUBLIC (non-dash-prefixed) op ctor, which has no client to
;; reply to. NAME PLACEHOLDER (per the brief, not yet cast-ratified).
(:wat::core::defrecord :wat::check::PublicOpInAlarm
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
   variant <- :wat::core::String
   op-type <- :wat::core::String])
