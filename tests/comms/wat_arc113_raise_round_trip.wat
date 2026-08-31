;; tests/comms/wat_arc113_raise_round_trip.wat — co-located fixture for the raise round-trip probe,
;; slurped via startup_beside(file!()). No placeholder main — startup_beside loads defns only.
;;
;; Arc 296 re-gate + arc 278 the string-wrap annihilation: raise! requires
;; :wat::core::Error, and the round-trip is now STRUCTURAL WITH NO STRING WRAP —
;; the error survives the panic boundary as a RECORD carried in Failure's `error`:
;;   (raise! (Fault/of "arc113-raise-data"))
;;     → (:wat::kernel::Failure/error f) yields the :wat::core::Fault RECORD DIRECTLY
;;       (no edn::write into a String, no edn::read back out)
;;     → (Fault/message …) reads the message field off it.
;;
;; Arc 278 IPC de-prime (MAP unit): the driver migrated off the retired non-prime
;; `:wat::test::run-thread` (fork/spawn + RunResult scrape) onto the PRIMED peer wire.
;; A thread peer (`spawn-program' (thread)`) runs the raise!; the child crashes
;; BEFORE it can send, so `recv'` faces `RecvOutcome::Lost cause` where `cause` is a
;; `:wat::kernel::LociDiedError`. A raise! is an AssertionPayload panic → the
;; `LociDiedError::Panic` variant, whose `failure` field is `Some(Failure)` carrying
;; the raised Fault STRUCTURALLY in its `error` slot (same read proven in
;; tests/comms/probe_arc278_failure_carries_structured_error.wat). The child body is
;; UNCHANGED; only the driver flipped. The death is never swallowed.
;;
;; Returns (Option :- [String]) = the raised Fault's message field, read structurally,
;; proving the error rode the boundary as a record — not a stringified blob.

(:wat::core::defn :my::compute [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::let
    [p
      (:wat::test::spawn-peer (:wat::spawn::thread)
        (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
          (:wat::kernel::raise!
            (:wat::core::Fault/of "arc113-raise-data"))))]
    (:wat::core::match (:wat::kernel::recv p)
      ;; A clean send would mean the raise! never fired — surface :None (the test asserts Some).
      ((:wat::kernel::RecvOutcome::Message _m) :wat::core::None)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::core::match cause
          ((:wat::kernel::LociDiedError::Panic _message failure)
            (:wat::core::match failure
              ;; STRUCTURAL read: Failure/error yields the raised :wat::core::Fault RECORD
              ;; directly (it rode the panic boundary as data); Fault/message reads the
              ;; field off it — no edn::write, no edn::read, no string round-trip.
              ((:wat::core::Some f)
               (:wat::core::Some
                 (:wat::core::Fault/message (:wat::kernel::Failure/error f))))
              (:wat::core::None :wat::core::None)))
          ;; LociDiedError is the no-hidden-failures enum — every OTHER death is named
          ;; EXPLICITLY (no `_` lump; verbosity is the shield). Each surfaces a distinct
          ;; WRONG:<variant> so a RED names exactly which non-Panic death arrived.
          ((:wat::kernel::LociDiedError::RuntimeError _m) (:wat::core::Some "WRONG:RuntimeError"))
          (:wat::kernel::LociDiedError::Disconnected (:wat::core::Some "WRONG:Disconnected"))
          (:wat::kernel::LociDiedError::Stopped (:wat::core::Some "WRONG:Stopped"))
          (:wat::kernel::LociDiedError::Severed (:wat::core::Some "WRONG:Severed"))
          ((:wat::kernel::LociDiedError::StartupError _m) (:wat::core::Some "WRONG:StartupError"))
          ((:wat::kernel::LociDiedError::EntryFormFailure _m) (:wat::core::Some "WRONG:EntryFormFailure"))
          ((:wat::kernel::LociDiedError::MainSignature _m) (:wat::core::Some "WRONG:MainSignature"))
          ((:wat::kernel::LociDiedError::BadReturn _m) (:wat::core::Some "WRONG:BadReturn"))))
      (:wat::kernel::RecvOutcome::Stopped (:wat::core::Some "UNEXPECTED-STOPPED"))
      (:wat::kernel::RecvOutcome::Closed :wat::core::None))))
