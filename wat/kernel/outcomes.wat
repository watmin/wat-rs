;; wat/kernel/outcomes.wat — arc 296 J: kernel outcome enums, declared in wat.
;;
;; The peer-lifecycle / I/O / run-result sum types formerly registered as
;; hand-written `EnumDef` literals in `src/types.rs`. wat is the source of
;; truth; Rust consumes them via `wat_enum_register_from!`.
;;
;; Load order: after `wat/kernel/diagnostics.wat` (`Failure`, `LociDiedError`)
;; and `wat/core.wat` (`Error`, `Option`). `Peer` is a Rust builtin.

;; Arc 170 — :wat::kernel::ReadFrameOutcome — what `:wat::kernel::read-frame` returns.
;;
;; A FRAME, not a line, and the name is load-bearing: `read_framed_edn`
;; (`src/edn/render.rs`) accumulates physical lines until the buffer forms a complete
;; EDN value. `:wat::io::IOReader/read-line` (`src/io.rs`) is the genuinely-one-line
;; verb; that name is already spent with that meaning, so reusing it here would both
;; lie and collide.
;;
;; MEASURED, because the obvious inference from "it accumulates" is WRONG for wat
;; source and I shipped that inference before running it: `next_complete_frame`
;; continues ONLY on `EdnFrameStatus::Incomplete`, and terminates the frame on
;; `Complete` *or* `Malformed` (`src/edn/render.rs`). Wat source is never valid EDN —
;; `:wat::core::defn` is precisely the "keyword begins with ::" that fails — so a
;; partial wat form scans as Malformed, not Incomplete, and the frame ENDS at the
;; first newline. Consequence: for wat input a frame is exactly one physical line, and
;; multi-line forms are NOT supported. Accumulation is real, but only for input that
;; is well-formed-EDN-so-far (an unclosed `{`).
;;
;; Three variants:
;;   :Frame [text] — the raw frame text, UNDECODED. `readln` reads the same bytes and
;;                   then EDN-decodes them, which is right for a wire and wrong for a
;;                   human: a REPL user types `(:wat::core::+ 1 1)`, which is wat
;;                   source, not an EDN literal, and decoding it fails on the `::`.
;;   :Eof []       — the clean stop, as a VALUE. The StdIn service has always returned
;;                   a matchable `::Eof` (`stdio.wat`, "NOT a panic that kills
;;                   the serve loop"); `stdio-read` then raised on it to preserve the
;;                   old fd-0 behavior for the 72 `readln` callers. That bank is what
;;                   made a REPL loop unable to stop cleanly. This verb spends it.
;;   :Stopped []   — a process-wide stop was requested while `stdio-read-frame`
;;                   (`stdio.wat`) was blocked waiting on the StdIn service.
;;                   NOT an `Eof` (the peer didn't close) and NOT an error — its own
;;                   outcome, matching `StdIn::ReadFrameResponse::Stopped` one layer
;;                   below. Named `Stopped`, not `Shutdown`: wat already has a word
;;                   for this fact — `(:wat::kernel::stopped?)` — and nothing is
;;                   shutting down here, a stop was merely requested. Ruled by an
;;                   intueri cast (2026-07-28, the "Stopped, not Shutdown" brief).
;;
;; It does NOT return the service's own `StdIn::ReadFrameResponse`. That enum carries
;; `RequestTooLarge`/`RequestMalformed` because `defservice` MANDATES those on every
;; serviceable op-Response — but this op's request is one i64 the kernel itself
;; builds, so neither can fire. Handing a caller an exhaustive match with two dead
;; arms is the same defect as any unreachable arm, and it would couple `:user::` code
;; to the stdin service's wire contract. A `:TooLong` arm is deliberately absent for
;; the same reason: it is only earned once `FramedRead::TooLarge` becomes a reply
;; instead of a raise inside `IOReader/read-frame`.
;;
;; Impure — it is I/O — and a sibling of the caller-facing `*Outcome` family
;; (RecvOutcome / SendOutcome / ConnectOutcome), so a reader already knows the shape.
;; Named by an intueri cast (2026-07-28), which also caught the frame-vs-line lie.
;; PURITY Impure: an I/O outcome
(:wat::core::defenum :wat::kernel::ReadFrameOutcome :wat::enum::Impure
  :Frame [text <- :wat::core::String]
  :Eof
;; Arc 170 stdin-joins-the-lock-step — a process-wide stop was requested
;; while `stdio-read-frame` (`stdio.wat`) was blocked waiting on
;; the StdIn service. NOT an `Eof` (the peer didn't close) and NOT an
;; error — its own outcome, matching `StdIn::ReadFrameResponse::Stopped`
;; one layer below. Named by the arc-170 intueri cast, 2026-07-28: wat
;; already has `(:wat::kernel::stopped?)` for this fact, so `Shutdown`
;; (a second word for the same thing) was the synonym anti-pattern.
  :Stopped)

;; Arc 170 closure #24 — (:wat::kernel::ReadlnOutcome :- [T]) — what `readln` returns.
;;
;; The THIRD outcome at this seam, and deliberately not either of the other two.
;; `IOReader::ReadFrameOutcome` and `kernel::ReadFrameOutcome` both carry RAW TEXT;
;; `readln` sits one level above them and hands back a DECODED value whose type flows
;; from the consumer (the arc-258/R54 `-> :T` annihilation: "readln reads what the
;; self-describing EDN wire says; the decoded value's type flows from the consumer").
;; So its payload cannot be `String` — it is the caller's `T`.
;;
;; WHY IT EXISTS. `readln` was the last IPC verb still RAISING. Every other one got its
;; outcome wall this arc (recv'/send'/close'/accept'/connect'), because a raise in a
;; language with no try/catch UNWINDS PAST THE READER — R53's `VERBO MEO CAPTVS`. The
;; wat `stdio-read` collapsed `Eof` and `Stopped` into `assertion-failed!` and said so
;; in its own comment: "the matchable ::Eof variant is BANKED, not yet exposed to the
;; 72 readln callers … there is no caller-facing value form for 'raise' to hand a stop
;; through". This is that value form; the bank is spent.
;;
;; `T` is generic exactly as `(RecvOutcome :- [O])`'s `O` is, and for the same reason —
;; the payload's type is the consumer's, not ours. That precedent is what makes this
;; mechanism already proven rather than newly invented.
;;
;; Impure for `(RecvOutcome :- [O])`'s reason exactly: `T` may itself be a live resource.
;;
;; ⚠ `Datum` is PROVISIONAL — arc 170 closure #26 casts intueri over this whole
;; surface (`StdIn::ReadFrameResponse` / `read-frame` / `:Frame`) and this variant name
;; rides with it. Named `Datum` and not `Value` to avoid colliding with
;; `:wat::core::Value`, the universal top; not `Line`, which is taken one layer down
;; for the raw text and would re-tell the frame-vs-line lie the 2026-07-28 cast caught.
(:wat::core::defenum :wat::kernel::ReadlnOutcome :- [T] :wat::enum::Impure
  :Datum [v <- :T]
  :Eof
  :Stopped)

;; :wat::kernel::LociDiedError — the ONE loci-agnostic death report
;; (arc 278 the IPC de-prime, DESIGN-loci-died-error.md). Annihilates the
;; two near-twin `ThreadDiedError` / `ProcessDiedError` enums: a service /
;; bracket-worker never knows its own locus (thread · process · uds ·
;; localhost tcp · remote mTLS · whatever comes), so its death is measured
;; as ONE enum every peer exhaustively handles (explicit-exception-paths,
;; verbosity-is-the-shield). The variant set is the UNION of the two dead
;; enums, generalized loci-agnostic — the variant names *how* a peer died;
;; the locus rides as data:
;;
;;   Panic(message, failure)  — peer raised/panicked; catch_unwind captured
;;                              the payload as `message`, `failure` is
;;                              `:Some(...)` when the panic carried an
;;                              arc-016/064 AssertionPayload, `:None`
;;                              for a plain `panic!()`.
;;   RuntimeError(message)    — a type/arity/etc. error surfaced at run.
;;   Disconnected             — the wire dropped (was ChannelDisconnected).
;;   Stopped                  — a stop was requested mid-recv, any locus (arc 170 intueri
;;                              cast: wat's word for this fact, not Rust's "shutdown").
;;   StartupError(message)    — the locus didn't come up (fork/exec fail,
;;                              or a remote ECONNREFUSED).
;;   EntryFormFailure(message)— the peer program's entry form was malformed.
;;   MainSignature(message)   — the peer's :user::main had a bad signature.
;;   BadReturn(message)       — the peer returned a value that won't cross
;;                              the wire.
;;
;; Purity::Pure — a death report crosses back to the owner as EDN data; its
;; payload is String / (Option :- [Failure]) (no live resource), unlike
;; (RecvOutcome :- [O]) which is Impure only because O may be live.
;;
;; ⛔ ARC 296 H-2c — GENERATED FROM WAT. The hand-written `EnumDef` literal that
;; stood here is DELETED; this row is now emitted from
;; `(:wat::core::defenum :wat::kernel::LociDiedError …)` in
;; `wat/kernel/diagnostics.wat`. wat is the source of truth; Rust consumes it.
;; The three EDN-tag string compares that used to key on `"wat.kernel.LociDiedError"`
;; source from the generated enum (`wat_enum_from!` in `src/kernel/error.rs`), so
;; a next wire change cannot unhook them silently.
;;
;; :wat::kernel::Location — a point in a source file. Populated by
;; `:wat::kernel::run-sandboxed` when a panic carries a PanicInfo
;; location, and by future assertion primitives whose failure-payload
;; needs to cite file:line:col.
;; ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
;; is DELETED; this row is now emitted from `(:wat::core::defrecord :wat::kernel::Location …)`
;; in `wat/core.wat`, read at BUILD time by `wat-source-derive`. wat is the source of truth;
;; Rust consumes it. Change the field list in the `.wat` and this registration follows —
;; there is no second copy to drift, and `include_str!` makes rustc rebuild when it moves.
;;
;; This is the first row converted, and it is the PROOF for the other twelve: if the emitted
;; registration were not identical to the literal it replaced, the corpus's own re-declaration
;; would stop hitting arc 054's `Existing::Equivalent` arm and the stdlib would fail to load.
;;
;; :wat::kernel::Frame — one entry on the wat call stack, captured by
;; `(:wat::kernel::call-site)` (from the runtime `FrameInfo` trampoline
;; stack) or by `(:wat::kernel::macro-call-site)` (from the expand-time
;; macro-invocation stack). Every field is ALWAYS KNOWN — the older
;; all-`Option` shape (justified by a never-built Rust-backtrace→Frame
;; path where symbol resolution could fail per-frame) was a lie: every
;; LIVE construction has a real file/line span and a real symbol (a named
;; fn's path, the `<anonymous>` marker for an anon fn, or the macro name
;; for a macro-call-site). Arc 109 — concrete, non-`Option` fields.
;; ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
;; is DELETED; this row is now emitted from `(:wat::core::defrecord :wat::kernel::Frame …)`
;; in `wat/kernel/diagnostics.wat`, read at BUILD time by `wat-source-derive`. wat is the
;; source of truth; Rust consumes it.
;;
;; :wat::core::Span — the leaf source location an error's `:location` floor
;; key carries (arc 278 "errors first-class EDN"). `Span` write-side is the
;; `#[derive(ToEdn)]` in `wat-reader` (`#wat.core/Span {:file :line :col :end}`)
;; but that derive is WRITE-ONLY (no `EdnSchema` submit) — so `edn_to_value`
;; STRICT could not reconstruct a `:location` back to a typed record; it hit
;; `UnknownTag`. Hand-register the decode schema here (the `:wat::kernel::Location`
;; exemplar above), so a `:wat::core::Error` floor record round-trips fully:
;; `:message` (String), `:location` (this Span), `:causes` ((Vector :- [Error])).
;; `:end` is `(Option :- [:wat::core::Pos])` (Pos is registered via the EdnSchema
;; drain below); `None` for the `rust_caller_span!()` point-spans.
;; ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
;; is DELETED; this row is now emitted from `(:wat::core::defrecord :wat::core::Span …)`
;; in `wat/core.wat`, read at BUILD time by `wat-source-derive`. wat is the source of truth;
;; Rust consumes it.
;;
;; :wat::kernel::Failure — structured panic / assertion payload
;; populated when a sandboxed `:user::main` fails. Slice 2b fills
;; the carried error / frames from `catch_unwind`; slice 3's
;; `:wat::test::assert-*` primitives additionally populate actual /
;; expected when the panic payload carries an AssertionPayload.
;; Arc 293.W.2b — Failure is pure EDN data (all fields are pure scalars/records); flipped
;; Struct → Record. Location and Frame also flipped to Record (pure data, no live resources).
;; This is the 2616-cascade root: ThreadDiedError/ProcessDiedError (Pure enums) carry
;; `failure: (Option :- [Failure])` — containment passes once Failure is a Record.
;;
;; Arc 278 the string-wrap annihilation — Failure carries the raised
;; `:wat::core::Error` STRUCTURALLY in a MANDATORY `error` field (four-questions Fork B).
;; The old stored `message` / `location` fields are REMOVED: `Failure/message` and
;; `Failure/location` are now DERIVED accessors reading `error.message` / `error.location`
;; (storing them alongside `error` fails Simple+Honest — duplication that can drift). The
;; `error` field is pure: `:wat::core::Error` is a `:nature :wat::core::Record` surface
;; (core.wat), and `is_pure_type` reads a surface's declared nature — post-load containment
;; (`validate_aggregate_containment`, freeze/env.rs) sees Error registered and passes.
;; ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
;; is DELETED; this row is now emitted from `(:wat::core::defrecord :wat::kernel::Failure …)`
;; in `wat/kernel/diagnostics.wat`, read at BUILD time by `wat-source-derive`. wat is the
;; source of truth; Rust consumes it.
;;
;; :wat::kernel::AssertionFailure — arc 278 (DESIGN-loci-died-error.md): the
;; registered record that the panic-hook `#wat.kernel/AssertionFailure {…}`
;; envelope writer now routes through (via the derived `ToEdn`), replacing
;; the hand-built Map with the wrong field shapes. `:frames` is a
;; `(Vector :- [Frame])` (was the ad-hoc `{:callee,:at}` map); `:location` is an
;; `(Option :- [Location])` (was a bare `Span`); `:upstream-chain` is a
;; `(Vector :- [LociDiedError])` (was heterogeneous Thread|Process). Every field
;; type (Frame, Location, Failure, LociDiedError) is registered above/below
;; — the record is EDN all the way down.
;; ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
;; is DELETED; this row is now emitted from `(:wat::core::defrecord :wat::kernel::AssertionFailure …)`
;; in `wat/kernel/diagnostics.wat`, read at BUILD time by `wat-source-derive`. wat is the
;; source of truth; Rust consumes it.
;;
;; :wat::kernel::StopAccepted — arc 170 "stopping is a protocol" Phase 2. The shutdown worker's
;; one notice, emitted exactly once on STDOUT (via the primed StdOut service, never a raw fd-1
;; write or eprintln — eprintln is wat's PANIC channel and a graceful stop is not a death) BEFORE
;; it asks any held service to stop. `services` names exactly the process-lifetime services being
;; asked (its held stdio Handles that were still live at the moment of the ask — an already-gone
;; Handle is silently omitted, never listed). Pure — crosses no live resource, pure EDN data,
;; rendering as `#wat.kernel/StopAccepted {:services [...]}`.
;; ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
;; is DELETED; this row is now emitted from `(:wat::core::defrecord :wat::kernel::StopAccepted …)`
;; in `wat/kernel/diagnostics.wat`, read at BUILD time by `wat-source-derive`. wat is the
;; source of truth; Rust consumes it.
;;
;; :wat::kernel::StopFailure — one service's failed stop, inside a `StopFailed`. `cause` carries
;; the STRUCTURED `:wat::core::Error` the failure already is (see `runtime.rs`'s
;; `fault_from_runtime_error`, which builds it as a `:wat::core::Fault` — the canonical minimal
;; record that structurally satisfies the `:wat::core::Error` surface, `wat/core.wat`) — never a
;; stringly message, never a bespoke `StopFailureCause` enum. Registered BEFORE `StopFailed` (which
;; holds `(Vector :- [StopFailure])`), matching the Frame/Location-before-AssertionFailure ordering above.
;; ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
;; is DELETED; this row is now emitted from `(:wat::core::defrecord :wat::kernel::StopFailure …)`
;; in `wat/kernel/diagnostics.wat`, read at BUILD time by `wat-source-derive`. wat is the
;; source of truth; Rust consumes it.
;;
;; :wat::kernel::StopFailed — arc 170 "stopping is a protocol", the builder's silent-drop-annihilation
;; ruling. The shutdown worker no longer discards an ask's (or the `StopAccepted` announce's) error —
;; every failure on the stop path is collected into this record and, once `:user::main` returns,
;; reported LOUDLY: emitted as registered EDN on STDERR (the dying-declaration channel — a graceful
;; stop that failed is no longer graceful) immediately before a non-zero exit
;; (`src/distribution/mod.rs`, beside the existing `emit_structured_exit` call). An empty collection
;; means nothing changes — exit as it always did.
;; ⛔ ARC 296 — GENERATED FROM WAT. The hand-written `AggregateDef` literal that stood here
;; is DELETED; this row is now emitted from `(:wat::core::defrecord :wat::kernel::StopFailed …)`
;; in `wat/kernel/diagnostics.wat`, read at BUILD time by `wat-source-derive`. wat is the
;; source of truth; Rust consumes it.
;;
;; (:wat::kernel::RecvOutcome :- [O]) — the matchable outcome of a point-to-point
;; peer read (`recv'`). Arc 278 the recv'-outcome wall (DESIGN-recv-outcome-wall.md):
;; recv' RETURNED O and RAISED on close/crash — a raise unwinds past the reader
;; (mute). This makes a reason-free failure UNREPRESENTABLE — a peer read yields a
;; matchable enum with exactly three shapes, mirroring the reason-bearing
;; `:wat::spawn::ServiceEvent` that select'/poll' already return:
;;   :Message [msg <- O]        — a real message (the happy path).
;;   :Closed  []                — a GENUINE clean EOF; the ONLY reason-free terminal.
;;   :Lost    [cause <- Failure] — abnormal loss; UNCONSTRUCTIBLE without a structured
;;                                cause. The cause is the first-class `:wat::kernel::Failure`
;;                                carrier (never a flat String — builder-ruled: wat is EDN
;;                                everywhere), the SAME structured carrier ServiceEvent::Lost
;;                                / Reply::Failed use (built via `message_only_failure`).
;; Impure like ServiceEvent (an I/O outcome). Registered as a builtin (peer with Failure /
;; (WalkStep :- [A])) so the checker knows it from type-env init — recv' is used INSIDE the stdlib
;; (spawn.wat) before any wat defenum would load; a builtin is load-order-robust and, per the
;; design's own note, Impure is the honest fixed purity (a Pure marking would lie the moment O
;; is a live resource). O carries the peer's output element type ((WalkStep :- [A]) is the parametric
;; precedent).
(:wat::core::defenum :wat::kernel::RecvOutcome :- [O] :wat::enum::Impure
  :Message [msg <- :O]
  :Closed
;; Arc 278 #73 — a stop was requested while this read was parked. NOTHING
;; DIED and NOTHING CLOSED: the peer is ALIVE and the channel is OPEN.
;;
;; Before this variant the fact had no honest home. It was produced (the
;; substrate has always known), then reported as `Lost[LociDiedError::Stopped]`
;; — a carrier whose very type name says "died" — so a caller matched a death
;; and had to open the death report to learn nothing had died. `Closed` was
;; the other candidate and is worse: it asserts a clean EOF that did not
;; happen (the false "peer closed" a months-long sigterm flake was made of).
;;
;; UNIT, carrying no cause: four precedents (`types.rs` Stopped variants) and
;; there is nothing to report. The substrate was asked to stop. That is the
;; whole fact — a cause here would be inventing a reason for "you asked me to".
  :Stopped
  :Lost [cause <- :wat::kernel::LociDiedError])

;; :wat::kernel::SendOutcome — Arc 278 the send'-outcome wall (Phase 1,
;; DESIGN-send-outcome-wall.md): the send-side twin of (RecvOutcome :- [O]) above.
;; send' RAISED reason-free MalformedForms on a gone peer ("peer already
;; closed" / "channel disconnected") — the last raise-that-masks. This makes
;; a send failure a matchable value instead, mirroring RecvOutcome exactly
;; except NON-parametric — send' carries no received payload, so no <O>:
;;   :Sent   []                — delivered (the happy path).
;;   :Closed []                — peer already cleanly closed (use-after-close;
;;                                was the "peer already closed" raise).
;;   :Lost   [cause <- LociDiedError] — disconnected mid-send; UNCONSTRUCTIBLE without
;;                                a structured cause. Arc 278 BRIEF-send-carries-its-cause
;;                                (#70): widened from the flat `Failure` to the SAME
;;                                loci-agnostic `LociDiedError` recv' already carries —
;;                                send' CAN distinguish a stop-woke-a-blocked-write
;;                                (`Stopped`) from a genuine peer loss (`Disconnected`);
;;                                it was simply discarding the distinction. Was the
;;                                "channel disconnected" raise.
;; PURE — unlike RecvOutcome. (RecvOutcome :- [O]) is Impure ONLY because of its payload
;; `O` (the received message may be a live resource — a socket/file handle). SendOutcome
;; is NON-parametric and holds only pure data: two nullary variants + `Lost[cause <-
;; LociDiedError]`, and LociDiedError is Purity::Pure (a death report — crosses back to
;; the owner as EDN data). A SendOutcome is fully EDN-reconstructable / wire-crossable;
;; marking it Impure would LIE (claim its values are locus-bound when they are not).
;; Registered as a builtin for the same load-order reason as RecvOutcome — send' is used
;; inside the stdlib before any wat defenum would load.
(:wat::core::defenum :wat::kernel::SendOutcome :wat::enum::Pure
  :Sent
  :Closed
;; Arc 278 #73 — the send-side twin of `RecvOutcome::Stopped` (see above for
;; the full argument). Landed in the SAME pass, deliberately: a half-fixed
;; pair is precisely how this arc got here — recv' was walled at R53 and the
;; send side went unwalled for months (R57 `IGNORANTIAM DELEMVS`).
;;
;; `send'` has always been able to tell a stop from a peer loss —
;; `SendError::Shutdown` is a distinct variant (`comms/mod.rs:919`, built to
;; mirror `RecvError::Shutdown`) — and folded it into `Lost` anyway.
  :Stopped
  :Lost [cause <- :wat::kernel::LociDiedError])

;; :wat::kernel::TrySendOutcome — Arc 278 the send'-outcome wall Phase 3a
;; (BRIEF-send-wall-3a-try-send-outcome.md): `try-send'`'s OWN outcome type,
;; sibling to SendOutcome, NOT a reuse. `try-send'` is NON-BLOCKING, so it has
;; an outcome `send'` structurally cannot: WouldBlock (a live peer just not
;; draining — the channel-full / deadlock-guard case, `service.wat:1163`).
;; Four-questions ruled: adding WouldBlock to SendOutcome FAILS (`send'`
;; never returns it — Obvious/Simple/Honest all fail); mapping WouldBlock to
;; Lost FAILS Honest ("alive but not draining" is not "gone"). So try-send'
;; gets its own type:
;;   :Sent       []                — delivered (the happy path).
;;   :WouldBlock []                — channel full / receiver not draining
;;                                    (crossbeam TrySendError::Full /
;;                                    process-tier EWOULDBLOCK) — try-send' ONLY.
;;   :Closed     []                — peer already cleanly closed (cell None).
;;   :Lost       [cause <- LociDiedError] — receiver dropped mid-send (crossbeam
;;                                    TrySendError::Disconnected / a genuine
;;                                    process-tier write failure). Arc 278
;;                                    BRIEF-send-carries-its-cause (#70): widened
;;                                    symmetric with SendOutcome::Lost above.
;; PURE for the same reason SendOutcome is (see above) — non-parametric, only
;; pure data (three nullary variants + a pure `LociDiedError` record).
(:wat::core::defenum :wat::kernel::TrySendOutcome :wat::enum::Pure
  :Sent
  :WouldBlock
  :Closed
  :Lost [cause <- :wat::kernel::LociDiedError])

;; :wat::kernel::CloseOutcome — Arc 278 peer-lifecycle Strike 2 (the close'
;; OUTCOME WALL, BRIEF-close-outcome-wall.md). `close'` (:wat::kernel::-restricted
;; teardown intrinsic) used to RAISE on its *handleable* failures (thread-join-
;; panic, process-signaled, process-wait-fail, process-stopped); per the
;; peer-lifecycle LAW those become a matchable outcome, only the must-never-happen
;; raises (double-close, close'-on-a-timer, arity/type) stay raises. Shape B (RULED):
;;   :Closed   [exit <- (Option :- [i64])] — clean close. None = thread (no OS exit code);
;;                                     Some(code) = process exit status. Loci-agnostic
;;                                     (R32): the exit rides in an Option, not two variants.
;;   :Signaled [signal <- i64]       — process TERMINATED by a signal (was the
;;                                     "killed by signal N" raise). Signaled means
;;                                     *terminated*, never merely stopped.
;;   :Failed   [cause <- Failure]    — join-panic / wait-fail / stopped-not-terminated;
;;                                     the abnormal-close carrier (structured Failure).
;; PURE — like SendOutcome, unlike (RecvOutcome :- [O]). Non-parametric; the peer is
;; CONSUMED (close' takes the Option, leaving None), so no value here holds a live
;; resource. It carries only pure data: an (Option :- [i64]), an i64, and a Nature::Record
;; Failure — fully EDN-reconstructable / wire-crossable. Marking it Impure would LIE.
;; Registered as a builtin for the same load-order reason as SendOutcome — close' is a
;; kernel intrinsic used before any wat defenum would load.
(:wat::core::defenum :wat::kernel::CloseOutcome :wat::enum::Pure
  :Closed [exit <- (:wat::core::Option :- [:wat::core::i64])]
  :Signaled [signal <- :wat::core::i64]
  :Failed [cause <- :wat::kernel::Failure])

;; :wat::kernel::Signal — Arc 278 process-signal-owner-to-child stone
;; (DESIGN-STONE-process-signal-owner-to-child.md § "The shape";
;; BRIEF-process-signal-p2-mint.md). A CLOSED SET (R27: a closed set is an
;; enum; the name holds the value) — a bare i64 signal number would be the
;; string-key mistake with a different hat. Six variants, three tiers,
;; deliberately NOT uniform in what they cause:
;;
;;   tier   variant     POSIX     who observes, and how
;;   flag   User1       SIGUSR1   the CHILD, and it keeps running — (sigusr1?) reads true
;;   flag   User2       SIGUSR2   the CHILD, and it keeps running — (sigusr2?) reads true
;;   flag   Hangup      SIGHUP    the CHILD, and it keeps running — (sighup?) reads true
;;   stop   Interrupt   SIGINT    the CHILD, and it chooses when to stop — (stopped?) reads true
;;   stop   Terminate   SIGTERM   the CHILD, and it chooses when to stop — (stopped?) reads true
;;   kill   Kill        SIGKILL   the OWNER — the child observes nothing and stops mid-instruction
;;
;; THIS TABLE IS THE ENUM'S DOC COMMENT, not commentary alongside it — it is
;; the only honest home for two facts about the SET that no single variant
;; name can carry:
;;   1. `Interrupt` and `Terminate` share ONE landing (both reach
;;      `substrate_on_stop_signal`; the child cannot tell them apart). Named
;;      independently anyway (RULED 2026-08-03): the shared landing is a
;;      HANDLER decision, not an identity claim about the signals — a
;;      non-wat observer (`strace`, `ps`) still sees the difference even
;;      though wat's own handler does not, and collapsing them to one `Stop`
;;      variant would forfeit the ability to say which one went out.
;;   2. `Kill` has NO child-side observable at all — SIGKILL is uncatchable
;;      (a POSIX guarantee, not a substrate choice; `handle.rs`: "SIGKILL is
;;      unignorable"). The round trip still closes, on the OWNER side, via
;;      `CloseOutcome::Signaled[signal <- i64]`.
;;
;; WHY the send side (this enum) is closed while the receive side
;; (`CloseOutcome::Signaled`'s bare i64) is open: we choose what is
;; SENDABLE — a closed, finite set we author — but we do not control what
;; KILLS you — any process on the box can send any signal. One concept,
;; two honest shapes for two different directions of control, not an
;; inconsistency to unify.
(:wat::core::defenum :wat::kernel::Signal :wat::enum::Pure
  :User1
  :User2
  :Hangup
  :Interrupt
  :Terminate
  :Kill)

;; :wat::kernel::SignalOutcome — the matchable outcome of
;; `(:wat::kernel::signal proc sig)` (BRIEF-process-signal-p2-mint.md).
;; Non-parametric — the peer is BORROWED, not consumed (unlike close', a
;; process may be signalled any number of times before it is closed), and
;; no variant holds a live resource. Same MUST_USE_TYPES slot as
;; CloseOutcome/SendOutcome (see check.rs `MUST_USE_TYPES`): a dropped
;; outcome is a compile error, closing both discard doors.
;;
;; `Delivered`, not `Sent` — `Sent` names the OWNER's action and is silent
;; on arrival, which is the entire reason this type exists.
;;
;; ⚠ STOP-2 — `Gone` (ESRCH) was NOT minted. Measured by a dedicated probe
;; (own run, 2026-08-03, `Pidfd::send_signal` against a child that had
;; exited but was deliberately left un-reaped): sending a signal to that
;; pidfd returns `Ok(())`, not ESRCH — delivery to a zombie is a silent
;; no-op, not an error. ESRCH appeared ONLY after the pidfd had already
;; been reaped — and in this substrate nothing reaps a `Process` peer's
;; pidfd except `close` (`eval_peer_close_prime`, `src/kernel/resource.rs`), which CONSUMES the peer
;; (`Option::take`). So the only way to reach an already-reaped pidfd
;; through this verb is to call it on an already-closed peer, and that path
;; is intercepted before the syscall (the same "peer already closed" guard
;; close' itself uses) — a live `signal` call can never observe ESRCH. Two
;; arms and a raise, per the stone's own named fallback for this outcome.
(:wat::core::defenum :wat::kernel::SignalOutcome :wat::enum::Pure
  :Delivered
  :Failed [cause <- :wat::kernel::Failure])

;; (:wat::kernel::AcceptOutcome :- [R S]) — Arc 278 peer-lifecycle Strike 3 (the accept'
;; OUTCOME WALL, BRIEF-accept-outcome-wall.md). `accept'` used to RETURN a bare
;; `(Peer' :- [R S])` and RAISE on its *handleable* failures (rendezvous dropped/shutdown,
;; decode error, `select` error, `peer_cred` read fail). Per the peer-lifecycle LAW
;; (2026-07-23) — "we deliver an enum for code to handle exceptions with; raise is
;; uncatchable on purpose, a thing that must never happen" — those become a matchable
;; outcome; only the must-never-happen raises (arity, listener-type-mismatch, and the
;; in-process malformed-connect-request substrate bug) stay raises. Shape (RULED):
;;   :Accepted [peer <- (Peer' :- [R S])]  — an AUTHORIZED peer connected (the happy path).
;;   :Closed   []                    — the listener's rendezvous shut down / address
;;                                     dropped (clean; no peer). The reason-free terminal.
;;   :Failed   [cause <- Failure]    — a decode / select / peer_cred / socket-wrap io
;;                                     error; the structured-cause carrier (never a flat
;;                                     String — built via `message_only_failure`).
;; `Rejected` is CUT: the security gate BOUNCES a stranger INTERNALLY (process tier:
;; drop + re-poll; thread tier: no gate — the crossbeam handle IS the grant), so no
;; tier returns a security-reject to the caller — a `Rejected` variant would never be
;; constructed (fails Honest).
;; Impure + PARAMETRIC, mirroring (RecvOutcome :- [O]): `Accepted` holds a live `Peer'` (a
;; socket/channel handle), so a Pure marking would lie the moment the peer is a live
;; resource. R,S carry the peer's wire element types (the parametric precedent).
;; Registered as a builtin for the same load-order reason as RecvOutcome — accept' is a
;; kernel verb usable inside the stdlib before any wat defenum would load.
(:wat::core::defenum :wat::kernel::AcceptOutcome :- [R S] :wat::enum::Impure
  :Accepted [peer <- (:wat::kernel::Peer :- [:R :S])]
  :Closed
  :Failed [cause <- :wat::kernel::Failure])

;; (:wat::kernel::ConnectOutcome :- [S R]) — Arc 278 peer-lifecycle Strike 4 (the connect'
;; OUTCOME WALL, BRIEF-connect-outcome-wall.md — the LAST peer-lifecycle wall). The
;; exact TWIN of `(AcceptOutcome :- [R S])` above. `connect'` used to RETURN a bare
;; `(Peer' :- [S R])` and RAISE on its *handleable* failures (ECONNREFUSED / no listener /
;; rendezvous gone, the `OnlyThisPeer` identity reject, `peer_cred` read fail,
;; socket-wrap io error). Per the peer-lifecycle LAW (2026-07-23) — "we deliver an enum
;; for code to handle exceptions with; raise is uncatchable on purpose, a thing that
;; must never happen" — those become a matchable outcome; only the must-never-happen
;; raises (arity, address-type-mismatch, and the in-process malformed-address substrate
;; bug — see below) stay raises. Shape (RULED):
;;   :Connected [peer <- (Peer' :- [S R])]  — dialed + admitted (the happy path).
;;   :Refused   [cause <- Failure]    — ECONNREFUSED / no listener / rendezvous gone;
;;                                      RETRYABLE transport (the server may come up).
;;   :Rejected  [cause <- Failure]    — the `OnlyThisPeer` identity check failed (the
;;                                      answerer's pid/euid != the address minter's);
;;                                      NOT retryable (wrong process, not a transport
;;                                      blip). FIRES here (unlike accept', where the
;;                                      gate bounces internally) — the client dials once
;;                                      and a server-identity mismatch is caller-visible.
;;   :Failed    [cause <- Failure]    — a `peer_cred` read / socket-wrap io error; the
;;                                      structured-cause carrier (never a flat String —
;;                                      built via `message_only_failure`).
;; Note the arg order `<S,R>` — connect's return is `Peer'<S,R>` (send-type first), the
;; MIRROR of accept's `Peer'<R,S>`. The must-never-happen raises stay raises: arity,
;; address-type-mismatch, and the in-process malformed-abstract-name (`from_abstract_name`
;; on `SocketAddress::name`) — the name is either kernel-minted (autobind, 5 random
;; bytes) or a wire-received `SocketAddressWire` already fully validated at decode
;; (non-empty, <=107-byte abstract-UDS limit, bytes 0..=255 — `capability::registry`),
;; so a malformed name at connect time is an in-process substrate bug, not adversarial
;; wire data (STOP-3, grounded).
;; Impure + PARAMETRIC, mirroring (AcceptOutcome :- [R S])/(RecvOutcome :- [O]): `Connected` holds a
;; live `Peer'` (a socket/channel handle), so a Pure marking would lie the moment the
;; peer is a live resource. S,R carry the peer's wire element types. Registered as a
;; builtin for the same load-order reason as AcceptOutcome — connect' is a kernel verb
;; usable inside the stdlib before any wat defenum would load.
(:wat::core::defenum :wat::kernel::ConnectOutcome :- [S R] :wat::enum::Impure
  :Connected [peer <- (:wat::kernel::Peer :- [:S :R])]
  :Refused [cause <- :wat::kernel::Failure]
  :Rejected [cause <- :wat::kernel::Failure]
  :Failed [cause <- :wat::kernel::Failure])

;; :wat::kernel::RunResult — the matchable outcome of running a program:
;; `:wat::kernel::run-sandboxed`, `:wat::test::run-thread` /
;; `run-hermetic'`, and (via the `:wat::test::TestResult` alias) every
;; `deftest`. Arc 278 the vacuous-gate wall (BRIEF-vacuous-deftest-gate-wall.md),
;; the third of the outcome walls after RecvOutcome (R53) and SendOutcome (R57).
;;
;; It WAS a Nature::Struct with one field, `failure <- (Option :- [Failure])`
;; (arc 278 wave 2d dropped the stdout/stderr capture buffers, leaving that
;; single slot). That shape is what let a caller look away: a pass and a
;; failure wore the SAME type, distinguished only by an `Option` slot nobody
;; was forced to read. The Rust gate idiom `call_beside_value(..).is_ok()` therefore
;; certified a fired assertion as a pass — proven by mutating a live gate's
;; `(assert-eq n 1)` to `n 4242` and watching the test still PASS.
;;
;; As an enum, a reason-free failure is UNREPRESENTABLE — exactly two shapes,
;; and `match` forces the reader to face both:
;;   :Passed []                    — the run completed with no failure.
;;   :Failed [failure <- Failure]  — UNCONSTRUCTIBLE without a structured cause;
;;                                    the first-class `:wat::kernel::Failure`
;;                                    carrier (never a flat String — wat is EDN
;;                                    everywhere), the same carrier RecvOutcome::Lost
;;                                    / SendOutcome::Lost / Reply::Failed use.
;;
;; PURE, for the same reason SendOutcome is: non-parametric, holding only pure
;; data (a nullary variant + a `Failure`, which is Nature::Record / pure EDN,
;; arc 293.W.2b). A RunResult is fully EDN-reconstructable and wire-crossable;
;; marking it Impure would lie. Registered as a builtin (like its two sibling
;; outcome walls) because `run-thread'` constructs it inside the stdlib, before
;; any wat `defenum` would load.
(:wat::core::defenum :wat::kernel::RunResult :wat::enum::Pure
  :Passed
  :Failed [failure <- :wat::kernel::Failure])
