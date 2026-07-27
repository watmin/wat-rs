;; scout-eval-3: unwrap the read-string outer list, THEN pure?/det?/eval-ast!/apply.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [pred-src "(:wat::core::fn [n <- :wat::core::i64] -> :wat::core::bool (:wat::core::> n 3))"
     wrapped  (:wat::core::match (:wat::core::read-string pred-src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     kids     (:wat::core::ast->children wrapped)
     form     (:wat::core::first kids)
     pure     (:wat::rete::pure? form)
     det      (:wat::rete::deterministic? form)
     res      (:wat::eval-ast! form)
     pred-fn  (:wat::core::Result/expect res "eval-ast! failed")
     keeps5   (:wat::core::apply  pred-fn [5])
     drops2   (:wat::core::apply  pred-fn [2])]
    (:wat::kernel::println (:wat::core::str pure))
    (:wat::kernel::println (:wat::core::str det))
    (:wat::kernel::println (:wat::core::str keeps5))
    (:wat::kernel::println (:wat::core::str drops2))))
