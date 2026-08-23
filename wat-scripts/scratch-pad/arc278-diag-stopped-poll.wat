;; Diagnostic: does (:wat::kernel::stopped?) observe SIGTERM at all once a thread peer
;; has been spawned? Decouples "does the signal handler fire" from "does Thread::recv
;; observe SHUTDOWN_RX".

(:wat::core::defn :diag::poll [n <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::if (:wat::kernel::stopped?)
    (:wat::kernel::println (:wat::core::string::concat "STOPPED_TRUE at n=" (:wat::core::i64::to-string n)))
    (:wat::core::if (:wat::core::i64::> n 200000000)
      (:wat::kernel::println "GAVE_UP_NEVER_STOPPED")
      (:diag::poll (:wat::core::i64::+ n 1)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [w (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           (:wat::core::match (:wat::kernel::recv self)
             ((:wat::kernel::RecvOutcome::Message _m) nil)
             ((:wat::kernel::RecvOutcome::Lost _c) nil)
             ;; the substrate stopping while parked on its own recv — same discard as
             ;; its siblings, this child's body never inspects the outcome either way.
             (:wat::kernel::RecvOutcome::Stopped nil)
             (:wat::kernel::RecvOutcome::Closed nil))))]
    (:wat::core::do
      (:wat::kernel::println "READY")
      (:diag::poll 0))))
