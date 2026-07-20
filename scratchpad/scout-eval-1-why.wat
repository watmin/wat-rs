;; scout-eval-1: capture the ACTUAL Err from eval-ast! on a bare fn-form.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [pred-src "(:wat::core::fn [n <- :wat::core::i64] -> :wat::core::bool (:wat::core::> n 3))"
     form     (:wat::core::read-string pred-src)
     pure     (:wat::rete::pure? form)
     det      (:wat::rete::deterministic? form)
     res      (:wat::eval-ast! form)]
    (:wat::kernel::println (:wat::core::str pure))
    (:wat::kernel::println (:wat::core::str det))
    (:wat::kernel::println (:wat::core::str res))))
