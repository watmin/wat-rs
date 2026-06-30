;; tests/comms/probe_arc209_structured_peer_death.wat — co-located fixture slurped via startup_beside(file!()).
;; Arc 209 C0b PREREQUISITE — structured peer death: AssertionPayload (actual+expected) must survive.
;; The thread peer crashes via assertion-failed! carrying known actual+expected strings.
;; recv' must raise with the STRUCTURED fields, not just the message.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::kernel::spawn-program' (:wat::spawn::thread)
         (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
           (:wat::kernel::assertion-failed! "structured-death-marker"
              (:wat::core::Some "ACTUAL-42173")
              (:wat::core::Some "EXPECTED-99731"))))
     _ (:wat::kernel::recv' p)]
    0))

