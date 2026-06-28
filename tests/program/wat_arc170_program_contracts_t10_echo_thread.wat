;; tests/program/wat_arc170_program_contracts_t10_echo_thread.wat — echo-thread fn for spawn-thread test.
(:wat::core::defn :my::echo-thread
  [rx <- :wat::kernel::Receiver<wat::core::i64>
   tx <- :wat::kernel::Sender<wat::core::i64>]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::recv rx) -> :wat::core::nil
    ((:wat::core::Ok (:wat::core::Some n))
      (:wat::core::match (:wat::kernel::send tx (:wat::core::i64::* n 2)) -> :wat::core::nil
        ((:wat::core::Ok _) nil)
        ((:wat::core::Err _) nil)))
    ((:wat::core::Ok :wat::core::None) nil)
    ((:wat::core::Err _died) nil)))
