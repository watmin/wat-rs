;; tests/process/probe_arc209_structured_peer_death_process.wat
;; co-located fixture for probe_arc209_structured_peer_death_process.rs
;; startup_beside(file!()) world — structured peer death, PROCESS tier (Arc 209 C0b).
;;
;; :user::compute spawns a :process peer via spawn-program', sends it 0 (prompting readln),
;; then recv' — the child calls assertion-failed! carrying actual + expected, which crashes it.
;;
;; Arc 278 recv'-wall + the LociDiedError stone: recv' returns a matchable RecvOutcome VALUE, never a
;; raise. The Lost cause is a loci-agnostic :wat::kernel::LociDiedError; its Panic variant carries the
;; structured Failure (message + actual + expected) in the `failure` field. `LociDiedError/to-failure`
;; recovers that Failure; we `edn::write` it so all three structured fields ride the returned String —
;; the .rs asserts they survive recv'.

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let [peer (:wat::test::spawn-peer (:wat::spawn::process)
                           (:wat::core::forms
                             (:wat::core::defn :user::main [] -> :wat::core::nil
                               (:wat::core::let [n (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                                                  _ (:wat::kernel::assertion-failed! "proc-structured-marker"
                                                      (:wat::core::Some "PROC-ACTUAL-5521")
                                                      (:wat::core::Some "PROC-EXPECTED-8841"))]
                                 nil))))
                    ;; arc 278 #73 — uniform, precondition is the recv' right below: a stop
                    ;; that interrupted this write is still in force when the read parks, so
                    ;; the read returns Stopped and the caller is told once, by the arm below.
                    _ (:wat::core::match (:wat::kernel::send peer 0) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
    (:wat::core::match (:wat::kernel::recv peer)
      ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost cause) (:wat::edn::write (:wat::kernel::LociDiedError/to-failure cause)))
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED") (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
