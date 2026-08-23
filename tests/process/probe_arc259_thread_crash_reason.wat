;; tests/process/probe_arc259_thread_crash_reason.wat
;; co-located fixture for probe_arc259_thread_crash_reason.rs
;; startup_beside(file!()) world — thread-peer crash-reason IPC (Arc 259 S3.5a-0).
;;
;; :user::compute spawns a thread peer whose body calls assertion-failed! with a known
;; sentinel. Arc 278 recv'-wall: recv' returns a matchable RecvOutcome VALUE (never a raise). We
;; MATCH the outcome and RETURN the Lost cause's `Failure/message` — which carries the crash reason
;; that travelled over the pipe — as a VALUE the .rs asserts.

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           (:wat::kernel::assertion-failed! "BOOM-SENTINEL-9173" :wat::core::None :wat::core::None)))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::LociDiedError/message cause))
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED"))))
