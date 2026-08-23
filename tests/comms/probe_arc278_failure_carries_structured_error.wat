;; tests/comms/probe_arc278_failure_carries_structured_error.wat — the RED gate for the
;; STRING-WRAP ANNIHILATION stone. Co-located fixture, slurped via startup_beside(file!()); defns only.
;;
;; THE SMELL BEING KILLED: `raise!(e: :wat::core::Error)` used to `edn::write` the Error into
;; a String field (`Failure.message`), and consumers had to `edn::read` it back out — EDN
;; wearing a string costume, double-encoded across a boundary that is already EDN.
;;
;; THE GATE (RED today, GREEN when the stone lands): a `:wat::kernel::Failure` carries the raised
;; `:wat::core::Error` STRUCTURALLY, in a mandatory `error` field. This probe reads it back with
;; NO `edn::read` and NO string re-parse — `(:wat::kernel::Failure/error f)` yields the Error
;; RECORD itself; `Fault/message` reads its field directly.
;;
;;   Today: RED on EXACTLY the gap — `:wat::kernel::Failure` has no `error` field, so the accessor
;;   `:wat::kernel::Failure/error` is unresolved (a check-time error). The string round-trip is the
;;   only path, and this probe refuses to take it.
;;   Post-stone: GREEN — the raised Fault survives as a structured record, read straight off the Failure.

(:wat::core::defn :my::failure-error-is-structured [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::let
    [p
      (:wat::test::spawn-peer (:wat::spawn::process)
        (:wat::core::forms
          (:wat::core::defn :user::main [] -> :wat::core::nil
            (:wat::kernel::raise! (:wat::core::Fault/of "structured-error-data")))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) :wat::core::None)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::core::match cause
          ((:wat::kernel::LociDiedError::Panic _message failure)
            (:wat::core::match failure
              ;; THE GATE: read the raised Error STRUCTURALLY off the Failure — no string re-parse.
              ((:wat::core::Some f)
               (:wat::core::Some
                 (:wat::core::Fault/message (:wat::kernel::Failure/error f))))
              (:wat::core::None :wat::core::None)))
          (_ :wat::core::None)))
      (:wat::kernel::RecvOutcome::Stopped :wat::core::None)
      (:wat::kernel::RecvOutcome::Closed :wat::core::None))))
