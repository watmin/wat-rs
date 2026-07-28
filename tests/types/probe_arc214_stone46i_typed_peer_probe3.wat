;; Fixture: spawn-program' :process against Process'<i64,i64> annotation type-checks.
(:wat::core::defn :user::mk-echo-proc [] -> :wat::kernel::Process<wat::core::i64,wat::core::i64>
  (:wat::test::spawn-peer (:wat::spawn::process)
    (:wat::core::forms
      (:wat::core::defn :user::main [] -> :wat::core::nil
        (:wat::core::let [n (:wat::kernel::readln )
                          _ (:wat::kernel::println n)]
          nil)))))
