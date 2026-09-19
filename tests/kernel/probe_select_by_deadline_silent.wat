;; Primitive: select-by-deadline on silent spawn-tier threads returns TimedOut.
;; Peers nap 2000ms and never send; deadline is 200ms. Unbounded select would hang.
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [t0 (:wat::time::epoch-nanos (:wat::time::now))
     a (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::nil :wat::core::nil])]
           -> :wat::core::nil
           (:wat::core::match
             (:wat::kernel::recv
               (:wat::kernel::after :wat::program::PeerKind::thread
                 (:wat::time::Milliseconds 2000) nil))
             ((:wat::kernel::RecvOutcome::Message _m) nil)
             ((:wat::kernel::RecvOutcome::Lost _c) nil)
             (:wat::kernel::RecvOutcome::Stopped nil)
             (:wat::kernel::RecvOutcome::Closed nil)
             (:wat::kernel::RecvOutcome::TimedOut nil)
             ((:wat::kernel::RecvOutcome::Malformed _c) nil))))
     b (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::nil :wat::core::nil])]
           -> :wat::core::nil
           (:wat::core::match
             (:wat::kernel::recv
               (:wat::kernel::after :wat::program::PeerKind::thread
                 (:wat::time::Milliseconds 2000) nil))
             ((:wat::kernel::RecvOutcome::Message _m) nil)
             ((:wat::kernel::RecvOutcome::Lost _c) nil)
             (:wat::kernel::RecvOutcome::Stopped nil)
             (:wat::kernel::RecvOutcome::Closed nil)
             (:wat::kernel::RecvOutcome::TimedOut nil)
             ((:wat::kernel::RecvOutcome::Malformed _c) nil))))
     ev (:wat::kernel::select-by-deadline [a b] 200)
     elapsed (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) t0) 1000000)
     name (:wat::core::match ev
            (:wat::spawn::SelectDeadline::TimedOut "TimedOut")
            ((:wat::spawn::SelectDeadline::Event e)
              (:wat::core::match e
                ((:wat::spawn::ServiceEvent::Message _i _m) "Message")
                ((:wat::spawn::ServiceEvent::Closed _i) "Closed")
                ((:wat::spawn::ServiceEvent::Lost _i _c) "Lost")
                ((:wat::spawn::ServiceEvent::Malformed _i _c) "Malformed")
                ((:wat::spawn::ServiceEvent::Rejected _i _c) "Rejected")
                (:wat::spawn::ServiceEvent::Shutdown "Shutdown")
                ((:wat::spawn::ServiceEvent::Connection _p) "Connection")
                ((:wat::spawn::ServiceEvent::Admin _m) "Admin"))))]
    (:wat::core::format "arm={a};elapsed-ms={e}"
      :a name :e elapsed)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))
