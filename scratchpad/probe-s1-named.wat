(:wat::core::defn :my::adder [n <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ n 5))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [wf (:wat::kernel::fn-forms :my::adder :probe::work)
     w  (:wat::kernel::spawn-program' (:wat::spawn::process)
          (:wat::core::concat wf
            (:wat::core::forms
              (:wat::core::defn :probe::runner [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                (:wat::core::let [i (:wat::kernel::recv' self) _ (:wat::kernel::send' self (:probe::work i))] (:probe::runner self)))
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:probe::runner (:wat::program::self-peer :wat::core::i64 :wat::core::i64))))))
     _ (:wat::kernel::send' w 1) _ (:wat::kernel::send' w 2)
     a (:wat::kernel::recv' w) b (:wat::kernel::recv' w)]
    (:wat::kernel::println (:wat::core::string::concat (:wat::core::i64::to-string a) (:wat::core::string::concat " " (:wat::core::i64::to-string b))))))
