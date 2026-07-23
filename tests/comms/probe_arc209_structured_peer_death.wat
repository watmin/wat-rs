;; tests/comms/probe_arc209_structured_peer_death.wat — co-located fixture slurped via startup_beside(file!()).
;; Arc 209 C0b PREREQUISITE — structured peer death: AssertionPayload (actual+expected) must survive.
;; The thread peer crashes via assertion-failed! carrying known actual+expected strings.
;;
;; Arc 278 recv'-wall: recv' returns a matchable RecvOutcome VALUE, never a raise (a raise unwinds
;; PAST the reader — the mask this arc kills). We MATCH the outcome and RETURN the crash reason as a
;; VALUE the .rs asserts. The thread crash channel carries the #wat.kernel/AssertionFailure envelope
;; (assertion_failure_envelope → payload_to_edn), so the Lost cause's `Failure/message` embeds BOTH
;; the message AND the structured actual/expected fields — the .rs asserts all three survive.

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::kernel::spawn-program' (:wat::spawn::thread)
         (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
           (:wat::kernel::assertion-failed! "structured-death-marker"
              (:wat::core::Some "ACTUAL-42173")
              (:wat::core::Some "EXPECTED-99731"))))]
    (:wat::core::match (:wat::kernel::recv' p)
      ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::Failure/message cause))
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED"))))
