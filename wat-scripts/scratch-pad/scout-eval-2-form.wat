;; scout-eval-2: inspect what read-string produces for a fn-form.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [pred-src "(:wat::core::fn [n <- :wat::core::i64] -> :wat::core::bool (:wat::core::> n 3))"
     form     (:wat::core::read-string pred-src)]
    (:wat::kernel::println (:wat::core::write-forms form))))
