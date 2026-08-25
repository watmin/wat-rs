;; localize pure?/det? : test a bare call form vs a fn form
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [call-form (:wat::core::quote (:wat::core::> n 3))
     fn-form   (:wat::core::quote
                  (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::bool
                    (:wat::core::> n 3)))]
    (:wat::kernel::println (:wat::string::concat "call pure=" (:wat::core::str (:wat::rete::pure? call-form))))
    (:wat::kernel::println (:wat::string::concat "call det="  (:wat::core::str (:wat::rete::deterministic? call-form))))
    (:wat::kernel::println (:wat::string::concat "fn   pure=" (:wat::core::str (:wat::rete::pure? fn-form))))
    (:wat::kernel::println (:wat::string::concat "fn   det="  (:wat::core::str (:wat::rete::deterministic? fn-form))))))
