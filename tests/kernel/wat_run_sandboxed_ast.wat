;; Co-located fixture for wat_run_sandboxed_ast.rs — slurped via startup_beside(file!()).
;;
;; Arc 278 IPC de-prime: both drivers migrated off the non-prime runners
;; (`:wat::test::run-hermetic` / `:wat::test::run-thread`) onto the PRIMED
;; peer wire (`spawn-program'` + `recv'`). The child bodies are unchanged;
;; only the driver flips.
;;
;; - compute-prints-hello (process tier): the child `println`s "hello"; on
;;   the primed wire that value crosses to the parent as a DECODED message
;;   (native String "hello"), not a scraped EDN stdout line ("\"hello\"").
;; - compute-assertion-failure (thread tier): the child's failing
;;   `assert-eq` CRASHES the peer → `recv'` returns `Lost[cause]`; a detected
;;   failure (the point of this test) maps to 1, mirroring the old
;;   `RunResult/failure` Some→1 / None→0.

(:wat::core::defn :my::compute-prints-hello [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println "hello"))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m) m)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "compute-prints-hello: child closed before sending its value" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

(:wat::core::defn :my::compute-assertion-failure [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           ;; The child body (assert-eq 1 2) is unchanged; a failing assertion
           ;; crashes the peer BEFORE the completion-signal send' — the parent's
           ;; recv' then faces Lost[cause] (the LociDiedError carrying the
           ;; assertion failure). A passing body would reach send' 0 → Message.
           (:wat::core::do
             (:wat::test::assert-eq 1 2)
             (:wat::core::match (:wat::kernel::send self 0)
               (:wat::kernel::SendOutcome::Sent   nil)
               (:wat::kernel::SendOutcome::Closed nil)
               ((:wat::kernel::SendOutcome::Lost _c) nil)
               (:wat::kernel::SendOutcome::Stopped nil)))))] ;; arc 278 #73 — fire-and-forget completion signal; outcome ignored uniformly regardless of cause
    ;; Reproduce the old `RunResult/failure` assertion off the recv' outcome:
    ;; Lost = the child crashed = the failure was detected → 1 (the Some arm);
    ;; Message = clean completion = no failure → 0 (the None arm). Stopped is
    ;; neither: the substrate asked to stop, the child never crashed, so it
    ;; is NOT a detected assertion failure → 0, same as Closed/Message.
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) 0)
      ((:wat::kernel::RecvOutcome::Lost _cause) 1)
      (:wat::kernel::RecvOutcome::Stopped 0)
      (:wat::kernel::RecvOutcome::Closed 0) (:wat::kernel::RecvOutcome::TimedOut 1))))
