;; tests/comms/probe_arc209_structured_peer_death.wat — co-located fixture slurped via startup_beside(file!()).
;; Arc 209 C0b PREREQUISITE — structured peer death: AssertionPayload (actual+expected) must survive.
;; The thread peer crashes via assertion-failed! carrying known message+actual+expected strings.
;;
;; Arc 278 recv'-wall: recv' returns a matchable RecvOutcome VALUE, never a raise (a raise unwinds
;; PAST the reader — the mask this arc kills). We MATCH the outcome and RETURN the surviving fields.
;;
;; Arc 278 no-hidden-failures (the string-wrap annihilation, deeper): the thread crash channel now
;; carries a STRUCTURED `(Vector :- [LociDiedError])` (parity with the process tier), NOT the flattened
;; `#wat.kernel/AssertionFailure {…}` envelope String. So `message`, `actual`, and `expected` ride in
;; the `Failure` RECORD's own fields — read STRUCTURALLY off `Failure/message` / `Failure/actual` /
;; `Failure/expected`, not scraped out of a stringified blob. We join the three with "|" so the .rs
;; asserts the EXACT structured value (no loose contains, no host path — these are the user's own
;; strings). The death is a Panic (assertion-failed! → AssertionPayload); every OTHER LociDiedError
;; variant is named EXPLICITLY (no `_` lump; verbosity is the shield) and returns a distinct sentinel.

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           (:wat::kernel::assertion-failed! "structured-death-marker"
              (:wat::core::Some "ACTUAL-42173")
              (:wat::core::Some "EXPECTED-99731"))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::core::match cause
          ((:wat::kernel::LociDiedError::Panic _message failure)
            (:wat::core::match failure
              ;; STRUCTURAL read: the Failure record carries message/actual/expected in its
              ;; own fields — the AssertionPayload rode the crash boundary as DATA.
              ((:wat::core::Some f)
               (:wat::core::let
                 [msg (:wat::kernel::Failure/message f)
                  a (:wat::core::match (:wat::kernel::Failure/actual f)
                      ((:wat::core::Some av) av)
                      (:wat::core::None "NO-ACTUAL"))
                  e (:wat::core::match (:wat::kernel::Failure/expected f)
                      ((:wat::core::Some ev) ev)
                      (:wat::core::None "NO-EXPECTED"))]
                 (:wat::string::concat msg
                   (:wat::string::concat "|"
                     (:wat::string::concat a
                       (:wat::string::concat "|" e))))))
              (:wat::core::None "NO-FAILURE")))
          ((:wat::kernel::LociDiedError::RuntimeError _m) "WRONG:RuntimeError")
          (:wat::kernel::LociDiedError::Disconnected "WRONG:Disconnected")
          (:wat::kernel::LociDiedError::Stopped "WRONG:Stopped")
          (:wat::kernel::LociDiedError::Severed "WRONG:Severed")
          ((:wat::kernel::LociDiedError::StartupError _m) "WRONG:StartupError")
          ((:wat::kernel::LociDiedError::EntryFormFailure _m) "WRONG:EntryFormFailure")
          ((:wat::kernel::LociDiedError::MainSignature _m) "WRONG:MainSignature")
          ((:wat::kernel::LociDiedError::BadReturn _m) "WRONG:BadReturn")))
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED") (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
