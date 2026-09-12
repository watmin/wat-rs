;; SendOutcome / TrySendOutcome after a THREAD child exits cleanly.
;; Process-tier send-after-exit was Lost; thread-tier may be Closed.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-send-closed-thread.wat

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
          (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
            nil))
     recv1 (:wat::core::match (:wat::kernel::recv p)
             ((:wat::kernel::RecvOutcome::Message _) "Message")
             (:wat::kernel::RecvOutcome::Closed "Closed")
             (:wat::kernel::RecvOutcome::Stopped "Stopped")
             (:wat::kernel::RecvOutcome::TimedOut "TimedOut")
             ((:wat::kernel::RecvOutcome::Lost _) "Lost")
             ((:wat::kernel::RecvOutcome::Malformed _) "Malformed"))
     send1 (:wat::core::match (:wat::kernel::send p 1)
             (:wat::kernel::SendOutcome::Sent "Sent")
             (:wat::kernel::SendOutcome::Closed "Closed")
             (:wat::kernel::SendOutcome::Stopped "Stopped")
             ((:wat::kernel::SendOutcome::Lost _) "Lost"))]
    (:wat::kernel::println
      (:wat::core::format "thread-recv-after-exit={r};thread-send-after-exit={s}"
        :r recv1 :s send1))))
