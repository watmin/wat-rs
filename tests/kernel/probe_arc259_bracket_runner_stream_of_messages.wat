;; Co-located fixture for probe_arc259_bracket_runner.rs — runner_serves_a_stream_of_messages.
;; 3-item stream: proves the peer serves MULTIPLE messages (1->2, 2->4, 3->6; sum 12).

(:wat::core::defn :user::compute [] -> :wat::core::i64
   (:wat::core::let
     [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
                     (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                       (:wat::bracket::runner-loop self
                         (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2)))))
      _a (:wat::kernel::send' peer 1) a (:wat::kernel::recv' peer)
      _b (:wat::kernel::send' peer 2) b (:wat::kernel::recv' peer)
      _c (:wat::kernel::send' peer 3) c (:wat::kernel::recv' peer)]
     (:wat::core::+ a (:wat::core::+ b c))))

