;; tests/process/probe_arc209_structured_peer_death_process.wat
;; co-located fixture for probe_arc209_structured_peer_death_process.rs
;; startup_beside(file!()) world — structured peer death, PROCESS tier (Arc 209 C0b).
;;
;; :user::compute spawns a :process peer via spawn-program', sends it 0 (prompting readln),
;; then recv' — the child calls assertion-failed! carrying actual + expected, which crashes it.
;;
;; Arc 278 recv'-wall: recv' returns a matchable RecvOutcome VALUE, never a raise (a raise unwinds
;; PAST the reader). We MATCH the outcome and RETURN the crash reason as a VALUE the .rs asserts. The
;; process crash channel (fd 2) carries the #wat.kernel/ProcessPanics envelope whose nested Failure
;; embeds the message AND the structured actual/expected, so the Lost cause's `Failure/message`
;; carries all three — the .rs asserts they survive.

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let [peer (:wat::kernel::spawn-program' (:wat::spawn::process)
                           (:wat::core::forms
                             (:wat::core::defn :user::main [] -> :wat::core::nil
                               (:wat::core::let [n (:wat::kernel::readln )
                                                  _ (:wat::kernel::assertion-failed! "proc-structured-marker"
                                                      (:wat::core::Some "PROC-ACTUAL-5521")
                                                      (:wat::core::Some "PROC-EXPECTED-8841"))]
                                 nil))))
                    _ (:wat::core::match (:wat::kernel::send' peer 0) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
    (:wat::core::match (:wat::kernel::recv' peer)
      ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::Failure/message cause))
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED"))))
