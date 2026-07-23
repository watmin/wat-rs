;; Proof 2: thread post-spawn hook fires owner-side with the empty ThreadLaunch.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair  (:wat::kernel::peer-pair' :wat::core::i64 :wat::core::i64)
     tx    (:wat::core::first pair)
     rx    (:wat::core::second pair)
     _thr  (:wat::kernel::spawn-program'
             (:wat::spawn::thread/post-spawn
               (:wat::core::fn [launch <- :wat::spawn::ThreadLaunch] -> :wat::core::nil
                 (:wat::core::let [_ (:wat::kernel::send' tx 777)] nil)))
             (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
               nil))
     sentinel (:wat::core::match (:wat::kernel::recv' rx)
                ((:wat::kernel::RecvOutcome::Message m) m)
                ((:wat::kernel::RecvOutcome::Lost cause)
                  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None))
                (:wat::kernel::RecvOutcome::Closed
                  (:wat::kernel::assertion-failed! "recv': rx closed before the post-spawn hook sent the sentinel" :wat::core::None :wat::core::None)))]
    sentinel))
