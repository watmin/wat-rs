;; wat/kernel/diagnostics.wat — arc 296: the kernel diagnostics aggregates,
;; declared in wat (wat is the source of truth).
;;
;; Seven Rust-registered aggregates (`register_builtin_types`, src/types.rs)
;; whose file placement is NOT the :wat::core::Error dependency edge that
;; sent :wat::kernel::Location to wat/core.wat — these seven are declared
;; here instead, in dependency order (each later form references an earlier
;; one). Zero transcription: the field names/types below came from the
;; registry describing itself (`:wat::runtime::field-names-of` /
;; `field-types-of`), not from anyone reading src/types.rs by eye.

;; ─── Arc 296: :wat::kernel::Frame — moving the source of truth to wat ─────
;;
;; Mirrors the Rust registration in `register_builtin_types` (src/types.rs).
;; Arc 296 moves the source of truth for wat's own aggregate types from the
;; hand-written Rust literal to a wat declaration; the Rust side is meant to
;; become generated FROM this form rather than hand-maintained alongside it.
;;
;; One entry on the wat call stack, captured by `(:wat::kernel::call-site)`
;; (from the runtime `FrameInfo` trampoline stack) or by
;; `(:wat::kernel::macro-call-site)` (from the expand-time macro-invocation
;; stack). Every field is ALWAYS KNOWN — a named fn's path, the
;; `<anonymous>` marker for an anon fn, or the macro name for a
;; macro-call-site (arc 109 — concrete, non-`Option` fields).
(:wat::core::defrecord :wat::kernel::Frame
  [file   <- :wat::core::String
   line   <- :wat::core::i64
   symbol <- :wat::core::String])

;; ─── Arc 296: :wat::kernel::StartupError — moving the source of truth to wat ───
;;
;; Mirrors the Rust registration in `register_builtin_types` (src/types.rs).
;; Arc 296 moves the source of truth for wat's own aggregate types from the
;; hand-written Rust literal to a wat declaration.
;;
;; Error variant of the Result returned by `:wat::kernel::spawn-program` /
;; `-ast` (arc 105a). Captured when freeze (parse + type-check + config +
;; macro) or `:user::main` signature validation fails. Single field for now
;; (the diagnostic message); extensible to kind / location if a real
;; consumer surfaces.
(:wat::core::defstruct :wat::kernel::StartupError
  [message <- :wat::core::String])

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
;; captured call stack; `actual` / `expected` are populated when the panic
;; payload carries an AssertionPayload.
(:wat::core::defrecord :wat::kernel::Failure
  [error    <- :wat::core::Error
   frames   <- (:wat::core::Vector :- [:wat::kernel::Frame])
   actual   <- (:wat::core::Option :- [:wat::core::String])
   expected <- (:wat::core::Option :- [:wat::core::String])])

;; ─── Arc 296: :wat::kernel::AssertionFailure — moving the source of truth to wat ───
;;
;; Mirrors the Rust registration in `register_builtin_types` (src/types.rs).
;; Arc 296 moves the source of truth for wat's own aggregate types from the
;; hand-written Rust literal to a wat declaration.
;;
;; Arc 278 (DESIGN-loci-died-error.md): the registered record that the
;; panic-hook `#wat.kernel/AssertionFailure {…}` envelope writer routes
;; through (via the derived `ToEdn`), replacing a hand-built Map with the
;; wrong field shapes. `frames` is a `(Vector :- [Frame])` (was an ad-hoc
;; `{:callee,:at}` map); `location` is an `(Option :- [Location])` (was a bare
;; `Span`); `upstream-chain` is a `(Vector :- [LociDiedError])` (was heterogeneous
;; Thread|Process) — the record is EDN all the way down.
(:wat::core::defrecord :wat::kernel::AssertionFailure
  [thread         <- :wat::core::String
   message        <- :wat::core::String
   location       <- (:wat::core::Option :- [:wat::kernel::Location])
   actual         <- (:wat::core::Option :- [:wat::core::String])
   expected       <- (:wat::core::Option :- [:wat::core::String])
   frames         <- (:wat::core::Vector :- [:wat::kernel::Frame])
   upstream-chain <- (:wat::core::Vector :- [:wat::kernel::LociDiedError])])
