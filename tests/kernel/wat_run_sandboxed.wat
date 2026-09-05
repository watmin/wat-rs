;; Co-located fixture for wat_run_sandboxed.rs — slurped via startup_beside(file!()).
;;
;; Arc 278 IPC de-prime (MAP unit). These tests historically drove hermetic-
;; execution semantics through the retired non-prime `:wat::test::run-hermetic`
;; capture model (fork + OS-pipe scrape → :wat::kernel::RunResult stdout/stderr/
;; failure). They are migrated onto the PRIMED peer wire — a direct
;; `(:wat::test::spawn-peer (:wat::spawn::process) (:wat::core::forms …))`
;; child + `(:wat::kernel::recv' p)` — and `RunResult` is GONE from this file.
;; Each hermetic semantic is re-expressed against the primed `RecvOutcome`:
;;
;;   - a child that prints a value      → RecvOutcome::Message[v]   — v is the
;;       DECODED native value (the old model captured the EDN text; the wire hands
;;       back the value itself: `(println "hello")` → Message["hello"], not
;;       Message["\"hello\""]).  (shape: tests/kernel/wat_hermetic_round_trip.wat)
;;   - a clean child that prints nothing → RecvOutcome::Closed
;;   - a child that DIES                → RecvOutcome::Lost[cause], cause a
;;       `:wat::kernel::LociDiedError` (matchable death report). raise!/panic →
;;       LociDiedError::Panic (message = the raised Fault's human message);
;;       a terminal `(eprintln v)` → LociDiedError::Panic (message = v's EDN;
;;       see tests/diagnostics/probe_arc278_eprintln_terminal.wat); a missing
;;       `:user::main` → LociDiedError::RuntimeError (UserMainMissing).
;;
;; `:wat::kernel::PipeWriter` is UNBUFFERED (src/io.rs — one write(2) per println,
;; no user-level buffer, flush is a no-op), so a println issued BEFORE a crash is
;; already in the kernel pipe and is received as a Message ahead of the crash
;; surfacing as Lost — that is what makes the "partial output then die" cases
;; (stdout-stderr, panic-partial) observe their pre-crash Messages deterministically.

;; ── noop — a clean child that prints nothing closes the wire ────────────────
;; No message, clean nil-return → recv' → Closed.
(:wat::core::defn :my::compute-noop [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil nil)))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) "message")
      ((:wat::kernel::RecvOutcome::Lost _cause) "lost")
      (:wat::kernel::RecvOutcome::Stopped "stopped")
      (:wat::kernel::RecvOutcome::Closed "closed") (:wat::kernel::RecvOutcome::TimedOut "lost"))))

;; ── single stdout write — the value crosses the wire DECODED ────────────────
;; `(println "hello")` → recv' → Message[m], m the native String "hello".
(:wat::core::defn :my::compute-single-line [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println "hello"))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m) m)
      ((:wat::kernel::RecvOutcome::Lost _cause) "UNEXPECTED-LOST")
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED") (:wat::kernel::RecvOutcome::TimedOut "UNEXPECTED-LOST"))))

;; ── stdout + terminal stderr — partial Messages then Lost ───────────────────
;; The child prints "one"/"two" (two Messages on the wire) then `(eprintln "oops")`
;; — a DYING declaration (arc 278): it emits the value's EDN and crashes the child.
;; Unbuffered PipeWriter → "one"/"two" are received as Messages BEFORE the crash
;; surfaces as Lost. The terminal eprintln's value rides the crash cause:
;; LociDiedError::Panic.message = the value's EDN "\"oops\"".
;; Returns [msg1 msg2 death-message].
(:wat::core::defn :my::compute-stdout-stderr [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::do
               (:wat::kernel::println "one")
               (:wat::kernel::println "two")
               (:wat::kernel::eprintln "oops")))))
     r1 (:wat::core::match (:wat::kernel::recv p)
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost _cause) "UNEXPECTED-LOST-1")
          (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED-1")
          (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED-1") (:wat::kernel::RecvOutcome::TimedOut "UNEXPECTED-LOST-1"))
     r2 (:wat::core::match (:wat::kernel::recv p)
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost _cause) "UNEXPECTED-LOST-2")
          (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED-2")
          (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED-2") (:wat::kernel::RecvOutcome::TimedOut "UNEXPECTED-LOST-2"))
     r3 (:wat::core::match (:wat::kernel::recv p)
          ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE-3")
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::core::match cause
              ((:wat::kernel::LociDiedError::Panic message _failure) message)
              (_ "LOST-NON-PANIC-3")))
          (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED-3")
          (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED-3") (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))]
    [r1 r2 r3]))

;; ── body-raise failure ("parse-error" case) — Lost[Panic] ──────────────────
;; `(raise! (Fault/of "inner-failure"))` crashes the child; recv' → Lost[Panic].
;; Panic.message carries the raised Fault's human message verbatim (arc 278 the
;; string-wrap annihilation; the structured Fault also rides Panic.failure).
;;
;; NOTE (see the MAP report + wat_run_sandboxed.rs header): the legacy NAME is
;; "parse-error", but a genuine lexer/parse error is UNREACHABLE over
;; spawn-program' :process — the entry `(:wat::core::forms …)` is already-parsed
;; AST, never a source string re-lexed in the child. This preserves the current
;; body's raise!->Panic semantics (what the test has actually exercised since its
;; arc-170 rearchitecture).
(:wat::core::defn :my::compute-parse-error [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::raise! (:wat::core::Fault/of "inner-failure")))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::core::match cause
          ((:wat::kernel::LociDiedError::Panic message _failure) message)
          (_ "LOST-NON-PANIC")))
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED") (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; ── missing :user::main — Lost[RuntimeError] (UserMainMissing) ─────────────
;; The entry forms define NO :user::main. Startup + main-signature validation both
;; pass (a missing main is NOT a signature error), then invoke_user_main raises
;; UserMainMissing at RUNTIME → the child dies with a RuntimeError envelope → recv'
;; → Lost[LociDiedError::RuntimeError].
;;
;; NOTE (report): this maps to RuntimeError, NOT MainSignature — MainSignature
;; fires only for a PRESENT main with a bad signature. Grounded: freeze.rs
;; `invoke_main_missing_is_error` (missing main → UserMainMissing) +
;; finish_forked_child's Ok(Err runtime) arm → process_died_error_runtime_value.
;; Returns a tag naming the variant that actually surfaced (so a RED reveals it).
(:wat::core::defn :my::compute-missing-main [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :my::not-a-main [] -> :wat::core::nil nil)))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::core::match cause
          ((:wat::kernel::LociDiedError::RuntimeError _message) "runtime-error")
          ((:wat::kernel::LociDiedError::Panic _pm _pf) "panic")
          ((:wat::kernel::LociDiedError::StartupError _sm) "startup-error")
          ((:wat::kernel::LociDiedError::MainSignature _mm) "main-signature")
          ((:wat::kernel::LociDiedError::BadReturn _bm) "bad-return")
          (_ "other-lost")))
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED") (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; ── partial output then panic — partial Message then Lost[Panic] ───────────
;; The child prints "before panic" (one Message on the wire) then
;; `(raise! (Fault/of "boom"))` crashes it. Unbuffered PipeWriter → the "before
;; panic" Message is received BEFORE the crash surfaces as Lost[Panic].
;; Returns [partial-message panic-message].
(:wat::core::defn :my::compute-panic-partial [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::do
               (:wat::kernel::println "before panic")
               (:wat::kernel::raise! (:wat::core::Fault/of "boom"))))))
     r1 (:wat::core::match (:wat::kernel::recv p)
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost _cause) "UNEXPECTED-LOST-1")
          (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED-1")
          (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED-1") (:wat::kernel::RecvOutcome::TimedOut "UNEXPECTED-LOST-1"))
     r2 (:wat::core::match (:wat::kernel::recv p)
          ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE-2")
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::core::match cause
              ((:wat::kernel::LociDiedError::Panic message _failure) message)
              (_ "LOST-NON-PANIC-2")))
          (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED-2")
          (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED-2") (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))]
    [r1 r2]))

;; ── scope inside — empty child loader → Err arm → terminal eprintln → Lost ──
;; Under hermetic the child's InMemoryLoader has NO entries, so eval-file! always
;; takes the Err arm; `(eprintln "err")` is a DYING declaration → the child dies.
;; recv' → Lost[Panic] whose message is the eprintln value's EDN "\"err\"".
;; (The Ok arm — `(println "ok")` — never runs.)
(:wat::core::defn :my::compute-scope-inside [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::match
               (:wat::eval-file! "/nonexistent-in-child-loader.wat")
               ((:wat::core::Ok h) (:wat::kernel::println "ok"))
               ((:wat::core::Err _) (:wat::kernel::eprintln "err"))))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::core::match cause
          ((:wat::kernel::LociDiedError::Panic message _failure) message)
          (_ "LOST-NON-PANIC")))
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED") (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; ── scope outside — same empty-loader Err arm; the Ok "leaked" never runs ───
;; recv' → Lost[Panic] whose message is the eprintln value's EDN "\"blocked\"".
(:wat::core::defn :my::compute-scope-outside [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::match
               (:wat::eval-file! "/also-nonexistent-in-child-loader.wat")
               ((:wat::core::Ok _) (:wat::kernel::println "leaked"))
               ((:wat::core::Err _) (:wat::kernel::eprintln "blocked"))))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::core::match cause
          ((:wat::kernel::LociDiedError::Panic message _failure) message)
          (_ "LOST-NON-PANIC")))
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED") (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
