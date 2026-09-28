;; wat/kernel/diagnostics.wat — arc 296: the kernel diagnostics aggregates,
;; declared in wat (wat is the source of truth).
;;
;; Seven Rust-registered aggregates (`register_builtin_types`, src/types.rs)
;; whose file placement is NOT the :wat::core::Error dependency edge that once
;; sent the narrower three-field "a location" record to wat/core.wat (retired
;; outright at excursus 003 D1 — :wat::core::Span, already resident there,
;; replaces it) — these seven are declared here instead, in dependency order
;; (each later form references an earlier one). Zero transcription: the field
;; names/types below came from the
;; registry describing itself (`:wat::runtime::field-names-of` /
;; `field-types-of`), not from anyone reading src/types.rs by eye.

;; ─── Excursus 003 D3: :wat::kernel::FrameKind — wat vs Rust frame origin ──────
;;
;; Every `:wat::kernel::Frame` names WHERE it came from: a wat call-stack entry
;; (`:Wat` — `CALL_STACK` / `MACRO_CALL_SITE`, `src/value/frame.rs`) or the ONE Rust
;; site that raised the error carrying it (`:Rust` — `#[track_caller]` at
;; `RuntimeError::new`, "like clojure has java in its traces"). A `:Rust` frame's
;; span always has `end` `None`: Rust knows only where the raising call began, never
;; where it ends (D1's own stated distinction, applied here).
(:wat::core::defenum :wat::kernel::FrameKind :wat::enum::Pure
;; A wat call-stack entry.
  :Wat
;; The Rust site (`#[track_caller]`) that constructed the error.
  :Rust)

;; ─── Arc 296 / Excursus 003 D3: :wat::kernel::Frame — moving the source of truth to wat ─────
;;
;; Mirrors the Rust registration in `register_builtin_types` (src/types.rs).
;; Arc 296 moves the source of truth for wat's own aggregate types from the
;; hand-written Rust literal to a wat declaration; the Rust side is meant to
;; become generated FROM this form rather than hand-maintained alongside it.
;;
;; One entry in a captured trace: a wat call-stack entry, captured by
;; `(:wat::kernel::call-site)` (from the runtime `FrameInfo` trampoline stack) or by
;; `(:wat::kernel::macro-call-site)` (from the expand-time macro-invocation stack) —
;; or the ONE Rust site that raised the error (excursus 003 D3). `symbol` is ALWAYS
;; KNOWN — a named fn's path, the `<anonymous>` marker for an anon fn, the macro name
;; for a macro-call-site, or the `<rust>` marker for a `:Rust` frame (arc 109 —
;; concrete, non-`Option` fields). `span` carries the location — for a `:Wat` frame,
;; the call site (a real `end`); for a `:Rust` frame, `#[track_caller]`'s point
;; location (`end` `None`). Excursus 003 D3 moved `file`/`line` off this record's own
;; fields and onto the shared `:wat::core::Span` (`Frame/span`), and added `kind` —
;; every prior `Frame/file` / `Frame/line` accessor use site now reads
;; `(:wat::core::Span/file (:wat::kernel::Frame/span f))` (a wat-fix codemod, not a
;; hand-edit — see `wat-scripts/fixes/nest-frame-file-line-in-span.wat`).
(:wat::core::defrecord :wat::kernel::Frame
  [symbol <- :wat::core::String
   span   <- :wat::core::Span
   kind   <- :wat::kernel::FrameKind])

;; ─── Excursus 003 step 3a: :wat::kernel::ClauseFailureReason / ClauseAttempt ──
;;
;; Mirror `crate::value::value::ClauseAttempt` / `ClauseFailureReason`
;; (`src/value/value.rs:500`/`:513`). Declared HERE, in `:wat::kernel::`, on the
;; builder's ruling (2026-09-26): they already ship on the wire as
;; `#wat.kernel/ClauseAttempt` / `#wat.kernel/<Reason>` (`src/edn/error.rs`'s
;; hand-written `clause_attempt_to_edn`/`clause_failure_reason_to_edn`), so they
;; keep that namespace rather than moving to `:wat::runtime::` alongside the
;; `RuntimeErrorKind` records that reference them (`NoMatchingClause`,
;; `wat/runtime-errors.wat`). Declaring them here only gains the enum-variant
;; tags their reason gets a dotted enum name (`#wat.kernel/ClauseFailureReason.
;; ArityMismatch` instead of the flat `#wat.kernel/ArityMismatch`) when
;; constructed through THIS declaration (`RuntimeError::to_record`) — the
;; existing hand-written writer above is untouched and keeps emitting the flat
;; form; the four `probe_arc237_stone4_*`/`probe_arc298_3_*` goldens that assert
;; it are unaffected (verified: they read through `clause_attempt_to_edn` and
;; `wat_edn::parse_owned`, neither of which this declaration touches).
;;
;; Per-clause failure reason for `defclause` dispatch — WHY a single clause was
;; skipped, not just THAT none matched (arc 233 errors-as-teaching-values).
(:wat::core::defenum :wat::kernel::ClauseFailureReason :wat::enum::Pure
;; The call's argument count did not match the clause's declared arity.
  :ArityMismatch    [expected <- :wat::core::i64
                     got      <- :wat::core::i64]
;; A specific argument position's runtime type did not match the clause's
;; declared type there.
  :ArgTypeMismatch  [position <- :wat::core::i64
                     expected <- :wat::core::String
                     got      <- :wat::core::String]
;; The clause's `:guard` expression evaluated to `false`.
  :GuardFalse)

;; One skipped `defclause` clause's diagnostic — index, declared shape, and why.
(:wat::core::defrecord :wat::kernel::ClauseAttempt
;; 0-based index of the clause in the defclause declaration.
  [clause-index       <- :wat::core::i64
;; Number of parameters the clause declares.
   declared-arity     <- :wat::core::i64
;; The clause's declared parameter types, formatted, in position order.
   declared-arg-types <- (:wat::core::Vector :- [:wat::core::String])
;; Why this clause was skipped.
   failure-reason     <- :wat::kernel::ClauseFailureReason])

;; ─── Arc 296: :wat::kernel::StartupError — RETIRED (excursus 003 strike A, F7) ───
;;
;; The `{message}` defstruct that used to live here (mirroring `register_builtin_types`,
;; src/types.rs:3071) was dead: constructed nowhere, read nowhere, in 0 goldens — dead
;; since step 3b gave `LociDiedError.StartupError` its own `Failure` payload, which is
;; what every real startup-failure producer builds instead. Not to be confused with
;; `:wat::kernel::LociDiedError::StartupError` (the enum variant below), which stays —
;; it is live and carries a real `Failure`.

;; ─── Arc 296: :wat::kernel::StopAccepted — moving the source of truth to wat ───
;;
;; Mirrors the Rust registration in `register_builtin_types` (src/types.rs).
;; Arc 296 moves the source of truth for wat's own aggregate types from the
;; hand-written Rust literal to a wat declaration.
;;
;; Arc 170 "stopping is a protocol" Phase 2. The shutdown worker's one
;; notice, emitted exactly once on STDOUT (via the primed StdOut service,
;; never a raw fd-1 write or eprintln — eprintln is wat's PANIC channel and
;; a graceful stop is not a death) BEFORE it asks any held service to stop.
;; `services` names exactly the process-lifetime services being asked (its
;; held stdio Handles that were still live at the moment of the ask — an
;; already-gone Handle is silently omitted, never listed).
(:wat::core::defrecord :wat::kernel::StopAccepted
  [services <- (:wat::core::Vector :- [:wat::core::String])])

;; ─── Arc 296: :wat::kernel::StopFailure — moving the source of truth to wat ───
;;
;; Mirrors the Rust registration in `register_builtin_types` (src/types.rs).
;; Arc 296 moves the source of truth for wat's own aggregate types from the
;; hand-written Rust literal to a wat declaration.
;;
;; One service's failed stop, inside a `StopFailed`. `cause` carries the
;; STRUCTURED `:wat::core::Error` the failure already is (see
;; `runtime.rs`'s `fault_from_runtime_error`, which builds it as a
;; `:wat::core::Fault` — the canonical minimal record that structurally
;; satisfies the `:wat::core::Error` surface, `wat/core.wat`) — never a
;; stringly message, never a bespoke `StopFailureCause` enum.
(:wat::core::defrecord :wat::kernel::StopFailure
  [service <- :wat::core::String
   cause   <- :wat::core::Error])

;; ─── Arc 296: :wat::kernel::StopFailed — moving the source of truth to wat ───
;;
;; Mirrors the Rust registration in `register_builtin_types` (src/types.rs).
;; Arc 296 moves the source of truth for wat's own aggregate types from the
;; hand-written Rust literal to a wat declaration.
;;
;; Arc 170 "stopping is a protocol", the builder's silent-drop-annihilation
;; ruling. The shutdown worker no longer discards an ask's (or the
;; `StopAccepted` announce's) error — every failure on the stop path is
;; collected into this record and, once `:user::main` returns, reported
;; LOUDLY: emitted as registered EDN on STDERR (the dying-declaration
;; channel — a graceful stop that failed is no longer graceful) immediately
;; before a non-zero exit. An empty collection means nothing changes — exit
;; as it always did.
(:wat::core::defrecord :wat::kernel::StopFailed
  [services <- (:wat::core::Vector :- [:wat::kernel::StopFailure])])

;; ─── Arc 296: :wat::kernel::Failure — moving the source of truth to wat ───
;;
;; Mirrors the Rust registration in `register_builtin_types` (src/types.rs).
;; Arc 296 moves the source of truth for wat's own aggregate types from the
;; hand-written Rust literal to a wat declaration.
;;
;; Structured panic / assertion payload populated when a sandboxed
;; `:user::main` fails. Arc 278 the string-wrap annihilation — Failure
;; carries the raised `:wat::core::Error` STRUCTURALLY in a mandatory
;; `error` field; `Failure/message` and `Failure/location` are DERIVED
;; accessors reading `error.message` / `error.location` (storing them
;; alongside `error` would duplicate data that can drift). `frames` is the
;; captured call stack. Excursus 003 step 3b — `frames-elided` joins the
;; floor: the wat call-stack cap (step 2's `capped_wat_frames`) is recorded
;; on the `Failure` that carries the frames it capped, not left stranded on
;; `RuntimeError` alone (the only producer that used to report it). 0 when
;; nothing was elided (the live stack fit under the cap, or the producer
;; never caps at all — an `AssertionPayload`'s frames are the full, uncapped
;; `snapshot_call_stack()` today; see BRIEF-envelope-step-3b).
;;
;; Excursus 003 strike A (F2) — `actual` / `expected` LEFT this record. They meant
;; something only for an assertion, and duplicated the assertion's own record
;; (`:wat::runtime::AssertionFailed`, `wat/runtime-errors.wat`, which already carries
;; them). An assertion's `error` field IS that record now, whichever path raised it
;; (an unhandled `assertion-failed!`, or `option`/`result::expect`); `Failure/actual`
;; and `Failure/expected` survive as DERIVED accessors (`src/intrinsic/kernel/error.rs`)
;; reading `error.actual` / `error.expected` when `error` is an `AssertionFailed`, and
;; `:wat::core::Option::None` otherwise — the same "no stored field" shape
;; `Failure/message` already uses for `error.message`.
(:wat::core::defrecord :wat::kernel::Failure
  [error         <- :wat::core::Error
   frames        <- (:wat::core::Vector :- [:wat::kernel::Frame])
   frames-elided <- :wat::core::i64])

;; ─── Arc 296 H-2c / Excursus 003 step 3b: :wat::kernel::LociDiedError — moving the source of truth to wat ───
;;
;; The ONE loci-agnostic death report (arc 278 DESIGN-loci-died-error.md). Variant
;; names *how* a peer died; the locus rides as data. Pure — a death report crosses
;; back to the owner as EDN. Field names/types transcribed from the Rust builtin
;; registration in `register_builtin_types` (src/types.rs) that this form replaces.
;;
;; Rust sources from this form: `wat_enum_from!` (a generated enum the consumers
;; ask) and `wat_enum_register_from!` (the TypeEnv row). There is no second list.
;;
;; Excursus 003 step 3b (DESIGN-the-error-envelope-and-its-frames.md D2) — every
;; failure-carrying variant now holds ONE `:wat::kernel::Failure` instead of a bare
;; `message <- String`: the envelope carries STRUCTURE, never a string. A Rust
;; constructor that used to put `to_wire_edn(e)` (the error's own serialized EDN)
;; into that string produced a double-quoted blob when the string was written out
;; again — this is the shape that stops it. `StartupError` joins the other failure-
;; carrying variants (was its own `error <- :wat::core::Error` field) so they all
;; share one failure shape. `LociDiedError/message` (`src/kernel/error.rs`) derives
;; its answer from `failure.error.message` for every one of them.
;;
;; Excursus 003 strike A (F7) — `EntryFormFailure` and `BadReturn` RETIRED.
;; `EntryFormFailure` had no producer anywhere in the tree (measured in step 3b,
;; re-confirmed here). `BadReturn`'s only producer is a runtime guard
;; (`src/process/verbs.rs`, `Ok(Ok(other))` when `:user::main` returns non-nil) that
;; the type checker refuses to ever reach — `:user::main`'s declared return type is
;; checked as `:wat::core::nil` at freeze, so a body returning anything else is a
;; check-time rejection, never a live runtime value (see
;; `tests/diagnostics/probe_excursus003_g3_one_shape_per_variant.rs` for the
;; measurement). The guard itself is KEPT (a guard that never fires from a legal wat
;; program is not dead code — reaching it would mean an internal invariant broke) but
;; now routes to `Panic` with a `Fault` naming the offending type, since that is a
;; panic's meaning, not a distinct death shape with no producer.
(:wat::core::defenum :wat::kernel::LociDiedError :wat::enum::Pure
;; Peer raised/panicked. Every panic — assertion-carrying or plain — now carries a
;; real `Failure`; a plain panic's `failure.error` is a `:wat::core::Fault`
;; synthesized from the panic message (and the panic's own location when the hook
;; captured one — measured: only an `AssertionPayload` panic's location is
;; captured; a bare String/&str panic has none, so its Fault's location falls back
;; to the Rust call site that built this value).
  :Panic            [failure <- :wat::kernel::Failure]
;; A type/arity/etc. error surfaced at run. `failure.error` is the error's own
;; declared `:wat::runtime::<Kind>` record (`RuntimeError::to_record`, step 3a).
  :RuntimeError     [failure <- :wat::kernel::Failure]
;; The wire dropped (was ChannelDisconnected).
  :Disconnected
;; A stop was requested mid-recv, any locus (arc 170: wat's word, not Rust's "shutdown").
  :Stopped
;; The locus didn't come up. `failure.error` is the structured cause the startup
;; pipeline raised (a `CheckErrors`, a `MacroError`, a `ReteCheckErrors`, a
;; `:wat::core::Fault`, …) — whatever concrete record already satisfies the
;; `:wat::core::Error` surface.
  :StartupError     [failure <- :wat::kernel::Failure]
;; The peer's :user::main had a bad signature.
  :MainSignature    [failure <- :wat::kernel::Failure])

;; ─── :wat::kernel::AssertionFailure — RETIRED (excursus 003 strike A, F1) ───
;;
;; The record that used to live here mirrored the panic-hook's hand-built
;; `#wat.kernel/AssertionFailure {…}` envelope (`src/panic_hook.rs`, arc 278
;; DESIGN-loci-died-error.md) — a SECOND top-level death shape for the exact same
;; event (a peer dying from an unhandled assertion) that `LociDiedError.Panic`
;; already reports for every other death. AUDIT-the-shape-of-an-error.md F1: 0 wat
;; readers, and the hand-built writer emitted `nil` for every `Option` field instead
;; of the declared record's own tagged-`None` convention — a declaration its own
;; writer disagreed with. An unhandled assertion is now reported exactly like any
;; other panic: `[#wat.kernel/LociDiedError.Panic {:failure #wat.kernel/Failure
;; {:error #wat.runtime/AssertionFailed {…}, ...}}]`. `thread` and `upstream-chain`
;; (the fields this record alone carried) retire with it — removal-biased, per the
;; audit's RULING: 0 wat readers, add back only if a real consumer asks.
