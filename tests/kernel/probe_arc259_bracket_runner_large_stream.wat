;; Co-located fixture for probe_arc259_bracket_runner.rs — runner_handles_a_large_stream.
;; 300-item stream: TCO proof (work-fn x*2; driver sends 1..=300; sum 2*(1+...+300)=90300).

(:wat::core::defn :user::drive
    [peer <- :wat::kernel::Thread'<wat::core::i64,wat::core::i64>
     n    <- :wat::core::i64
     acc  <- :wat::core::i64] -> :wat::core::i64
   (:wat::core::if (:wat::core::= n 0)
     acc
     (:wat::core::let [_   (:wat::kernel::send' peer n)
                       res (:wat::kernel::recv' peer)]
       (:user::drive peer (:wat::core::- n 1) (:wat::core::+ acc res)))))

(:wat::core::defn :user::compute [] -> :wat::core::i64
   (:wat::core::let
     [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
                     (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                       (:wat::bracket::runner-loop self
                         (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2)))))]
     (:user::drive peer 300 0)))

