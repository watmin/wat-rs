;; Fixture: spawn-program' :thread against Thread'<i64,i64> annotation type-checks.
(:wat::core::defn :user::mk-echo-peer [] -> (:wat::kernel::Thread :- [:wat::core::i64 :wat::core::i64])
  (:wat::test::spawn-peer (:wat::spawn::thread)
    (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
      ;; arc 278 recv'-outcome wall — recv' returns a matchable RecvOutcome<i64>; the
      ;; echo consumer (send' self …) still pins O through the ::Message binding m.
      (:wat::core::let [r (:wat::kernel::recv self)]
        (:wat::core::match
          (:wat::kernel::send self
            (:wat::core::match r
              ((:wat::kernel::RecvOutcome::Message m) m)
              ((:wat::kernel::RecvOutcome::Lost cause)
                (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
              (:wat::kernel::RecvOutcome::Stopped
                (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; self was ALIVE and the channel open" :wat::core::None :wat::core::None))
              (:wat::kernel::RecvOutcome::Closed
                (:wat::kernel::assertion-failed! "recv': self closed before echo" :wat::core::None :wat::core::None))))
          (:wat::kernel::SendOutcome::Sent nil)
          (:wat::kernel::SendOutcome::Closed nil)
          (:wat::kernel::SendOutcome::Stopped nil)
          ((:wat::kernel::SendOutcome::Lost _c) nil))))))
