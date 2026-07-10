;; probe-m1-phantom-d.wat — does a PoolMsg<D,I> with D never constructed monomorphize?
;; A thread worker recv's PoolMsg<D,I>, only ever gets :Work. D is phantom.
;; EXPECT (green): "6" (work-fn doubles 3)

(:wat::core::defenum :probe::PoolMsg<D,I> :wat::enum::Pure
  :Setup [deps <- :D]
  :Work  [pair <- :(wat::core::i64,I)])

(:wat::core::defn :probe::serve
  [self <- :wat::kernel::ThreadSelfPeer'<(wat::core::i64,wat::core::i64),probe::PoolMsg<wat::core::nil,wat::core::i64>>]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::recv' self) -> :wat::core::nil
    ((:probe::PoolMsg::Work pair)
      (:wat::core::let
        [out (:wat::core::Tuple (:wat::core::first pair) (:wat::core::* (:wat::core::second pair) 2))
         _   (:wat::kernel::send' self out)]
        (:probe::serve self)))
    ((:probe::PoolMsg::Setup _deps)
      (:probe::serve self))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [w (:wat::kernel::spawn-program' (:wat::spawn::thread)
         (:wat::core::fn [sp <- :wat::kernel::ThreadSelfPeer'<(wat::core::i64,wat::core::i64),probe::PoolMsg<wat::core::nil,wat::core::i64>>] -> :wat::core::nil
           (:probe::serve sp)))
     _  (:wat::kernel::send' w (:probe::PoolMsg::Work (:wat::core::Tuple 0 3)))
     r  (:wat::kernel::recv' w)]
    (:wat::kernel::println (:wat::core::second r))))
