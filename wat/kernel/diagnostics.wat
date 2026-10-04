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

;; ─── Excursus 003 strike D: :wat::kernel::Frame — a function and where it is ──
;;
;; Mirrors the Rust registration in `register_builtin_types` (src/types.rs).
;; Arc 296 moves the source of truth for wat's own aggregate types from the
;; hand-written Rust literal to a wat declaration; the Rust side is meant to
;; become generated FROM this form rather than hand-maintained alongside it.
;;
;; One entry in a captured trace, pairing (*function*, *where inside it
;; execution is*) — the Clojure/Java backtrace convention. `fn` is ALWAYS KNOWN
;; (arc 109 — concrete, non-`Option` fields; never a placeholder): a named fn's
;; path, the `<anonymous>` marker for an anon fn, the macro name for a
;; macro-call-site, or — for the one innermost Rust activation every
;; `RuntimeError` carries — the registered intrinsic/special-form name
;; currently dispatching, or the freeze phase currently running, named by
;; `crate::value::frame::current_activation()` (excursus 003 strike D item 4:
;; a frame's identity is a property of the STACK, not of the error's content,
;; so this is never read off `RuntimeErrorKind`). `at` is the location INSIDE
;; `fn` — the call it is about to make, or the Rust raise site. `tail-elided`
;; (excursus 003 strike D item 3, the RULING 2026-10-03) counts how many
;; tail-collapsed activations are missing BEYOND the one named here: `0` for
;; an ordinary frame, and also for a tail-collapsed frame whose own single
;; substitution lost nothing further — the value is honest either way.
;;
;; `:kind` (excursus 003 D3's wat-vs-Rust origin marker) is RETIRED this
;; strike: it was derivable from `span.end` (D1) and duplicated information
;; `at` already carries; the RULING struck it along with the fabricated
;; raise frame it used to distinguish.
(:wat::core::defrecord :wat::kernel::Frame
  [fn          <- :wat::core::String
   at          <- :wat::core::Span
   tail-elided <- :wat::core::i64])

;; ─── Excursus 003 step 3a: :wat::kernel::ClauseFailureReason / ClauseAttempt ──
;;
;; Mirror `crate::value::value::ClauseAttempt` / `ClauseFailureReason`
;; (`src/value/value.rs:500`/`:513`). Declared HERE, in `:wat::kernel::`, on the
;; builder's ruling (2026-09-26): they already ship on the wire as
;; `#wat.kernel/ClauseAttempt` / `#wat.kernel/ClauseFailureReason.<Variant>`
;; (`src/edn/error.rs`'s hand-written `clause_attempt_to_edn`/
;; `clause_failure_reason_to_edn`), so they keep that namespace rather than
;; moving to `:wat::runtime::` alongside the `RuntimeErrorKind` records that
;; reference them (`NoMatchingClause`, `wat/runtime-errors.wat`).
;;
;; ⛔ EXCURSUS 003 STRIKE B2, item 5 — CLOSED. This comment used to note a gap:
;; `RuntimeError::to_record` rendered `failure-reason` dotted (via this
;; declaration), while `error_edn()`'s hand-written `clause_failure_reason_to_edn`
;; kept emitting the flat `#wat.kernel/<Variant>` form — the two writers
;; disagreed. Both now call the SAME dot-join (`edn_tag_dotted` /
;; `wat_edn::Tag::enum_variant`, `src/edn/contract.rs` — the helper
;; `#[to_edn(qualified)]` also emits), so `error_edn()`'s wire and `to_record()`'s
;; render are byte-identical for this field; `probe_excursus003_step3a_wat_records
;; .rs`'s G2 gate no longer lists `("NoMatchingClause", "attempted-clauses")` as
;; an exception (the generic byte-equality check now covers it).
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

;; ─── Excursus 003 S1: :wat::kernel::Remedy — ranked remediation candidate ────
;;
;; Mirrors `crate::remedy::Remedy` (`src/remedy/mod.rs:79`) and its hand-written
;; `ToEdn` impl (`src/remedy/mod.rs:154`: `#wat.kernel/Remedy {:form :kind
;; :score :note}`). Declared HERE (kernel namespace, matching the wire tag)
;; rather than in `wat/check-errors.wat`, on the same placement rule step 3a
;; used for `ClauseAttempt`/`ClauseFailureReason` above: the type already
;; ships as `#wat.kernel/Remedy`, and `CheckErrorKind`'s `TypeMismatch`/
;; `ReturnTypeMismatch`/`MalformedForm` are only the FIRST taxonomy to embed
;; it, not its owner. `note` is `nil` unless the remedy carries a migration
;; caveat (a retired-form replacement needing more than a form-swap).
(:wat::core::defrecord :wat::kernel::Remedy
  [form  <- :wat::core::String
;; `:typo` (edit-distance candidate) or `:retirement` (explicit table hit).
   kind  <- :wat::core::keyword
;; Levenshtein distance for a typo; 0 for a retirement (exact table hit).
   score <- :wat::core::i64
   note  <- (:wat::core::Option :- [:wat::core::String])])

;; ─── Excursus 003 S1: EnsureFnInvalidReason — MOVED (strike B2, item 2) ─────
;;
;; `CheckErrorKind::EnsureFnInvalid.reason`'s sum type used to live here as
;; five independent flat records (S1: no `defenum` existed yet, and the
;; derive's own default namespace put every variant's flat tag under
;; `wat.kernel`). Strike B2 gives it a genuine `defenum`, dot-joined by
;; `#[to_edn(qualified)]`, and moves it to its OWN namespace —
;; `:wat::check::EnsureFnInvalidReason`, declared in `wat/check-errors.wat`
;; (it is a check diagnostic, not a kernel shared value) — right before
;; `:wat::check::EnsureFnInvalid`, the record whose `reason` field names it.

;; ─── Excursus 003 strike B2, item 2: :wat::kernel::LoadFetchError ───────────
;;
;; Mirrors `crate::load::loader::LoadFetchError` (`src/load/loader.rs`), the
;; payload of `LoadErrorKind::Fetch` (`wat/load-errors.wat`). S2 declared this
;; as three independent flat records (one `edn_tag` per variant, undotted) —
;; B2 closes it: ONE `defenum`, its hand-written `ToEdn` impl (kept hand-
;; written — the Rust variant `Other` renames its wire TAG to `LoadOther`,
;; which `#[derive(ToEdn)]` has no directive for) now calling `edn_tag_dotted`,
;; the SAME dot-join helper `#[to_edn(qualified)]` emits
;; (`#wat.kernel/LoadFetchError.<Variant>`).
;;
;; `LoadFetchError` is DATA (a reason code), not an Error: it carries no
;; `message`/`location` of its own (unlike `HashError` below, which strike B2
;; item 3 DOES promote to a floor-bearing record) — `LoadErrorKind::Fetch`
;; already supplies the floor at the outer level. `Fetch.cause` (`wat/load-
;; errors.wat`) is retyped from `:wat::core::Value` to the concrete enum,
;; `:wat::kernel::LoadFetchError`, per item 2's "holder field is typed as the
;; enum" instruction.
(:wat::core::defenum :wat::kernel::LoadFetchError :wat::enum::Pure
;; The loader's requested path does not exist under its domain.
  :NotFound   [path <- :wat::core::String]
;; Loader-specific I/O or resolution failure; `reason` is prose. Variant
;; keyword is `LoadOther` (the WIRE tag — the hand-written writer renames the
;; Rust variant `Other` to it).
  :LoadOther  [path   <- :wat::core::String
               reason <- :wat::core::String]
;; The requested path's canonical target escapes the loader's allowed scope
;; (e.g. `../../etc/passwd`, or a symlink pointing outside the scope).
  :OutOfScope [path  <- :wat::core::String
               scope <- :wat::core::String])

;; ─── Excursus 003 strike B2, item 3: :wat::kernel::HashErrorKind / HashError ──
;;
;; Mirrors `crate::hash::HashErrorKind` / `HashError` (`src/hash.rs`). S2
;; declared `HashError`'s eight variants as independent flat records (one
;; `edn_tag` per variant, undotted) — the same shape `LoadFetchError` above
;; USED to have. B2 closes it for HashError: `HashErrorKind` is now ONE
;; `defenum`, `#[derive(ToEdn)]`'s `qualified` directive dot-joining every
;; variant's wire tag (`#wat.kernel/HashErrorKind.<Variant>`), registered
;; with `wat_enum_register_from!` — no hand-rolled writer (unlike
;; `LoadFetchError` above, which keeps its hand-written writer for the
;; `Other`->`LoadOther` tag rename the derive cannot express). `usize` fields
;; (`expected`/`got`) type `:wat::core::i64`, matching every other `usize`
;; field in this sweep.
;;
;; `HashError` ALSO gains the `:wat::core::Error` floor (`message`/
;; `location`) around `kind` — it is a `defrecord` (an Aggregate), not the
;; `defenum` itself: `struct_satisfies_surface` (`src/types/surface.rs`) and
;; `conforms_to_surface` (`src/edn/render.rs`) both check ONLY Aggregate
;; fields for a Field-surface member, never a bare enum's variant fields, so
;; the union data had to move OFF the Error-typed slot and into this
;; wrapper's own `kind` field for `HashError` to structurally satisfy
;; `:wat::core::Error`. `location` is the verifying call's own span (threaded
;; in via `HashError::new` at its 8 construction sites — `src/runtime.rs`,
;; `src/freeze.rs`, `src/holon/coincident.rs`, `src/load/loader.rs`), the
;; SAME span value the OUTER `LoadError`/`RuntimeError` already raises with.
;;
;; `LoadErrorKind::VerificationFailed.cause` / `RuntimeErrorKind::
;; EvalVerificationFailed.cause` are now typed `:wat::core::Error` (not
;; `:wat::core::Value`) — `EvalVerificationFailed.cause` held B1's interim
;; `:wat::core::Fault` until this strike closed the gap that section's old
;; comment named; see `src/value/runtime_records.rs`'s `hash_error_value`.

;; The 8 structural failure modes for hash / signature verification.
(:wat::core::defenum :wat::kernel::HashErrorKind :wat::enum::Pure
;; The requested digest algorithm is not supported (this build supports sha256).
  :UnsupportedAlgorithm         [algo     <- :wat::core::String]
;; A computed digest did not match the expected one.
  :Mismatch                     [algo     <- :wat::core::String
                                  expected <- :wat::core::String
                                  actual   <- :wat::core::String]
;; The requested signature algorithm is not supported (this build supports ed25519).
  :UnsupportedSignatureAlgorithm [algo    <- :wat::core::String]
;; A base64-encoded field failed to decode.
  :InvalidBase64                [field  <- :wat::core::String
                                  reason <- :wat::core::String]
;; A decoded signature's byte length did not match the algorithm's expectation.
  :InvalidSignatureLength       [algo     <- :wat::core::String
                                  expected <- :wat::core::i64
                                  got      <- :wat::core::i64]
;; A decoded public key's byte length did not match the algorithm's expectation.
  :InvalidPubKeyLength          [algo     <- :wat::core::String
                                  expected <- :wat::core::i64
                                  got      <- :wat::core::i64]
;; A decoded public key's bytes do not form a valid key for the algorithm.
  :InvalidPubKey                [algo   <- :wat::core::String
                                  reason <- :wat::core::String]
;; A signature verification check failed (the bytes don't verify against the key).
  :SignatureMismatch            [algo <- :wat::core::String])

;; The `:wat::core::Error`-floored wrapper: `message`/`location` plus the
;; structural failure itself.
(:wat::core::defrecord :wat::kernel::HashError
  [message  <- :wat::core::String
   location <- :wat::core::Span
   kind     <- :wat::kernel::HashErrorKind])

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
