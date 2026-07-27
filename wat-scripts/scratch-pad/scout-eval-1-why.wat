;; scout-eval-1: capture the ACTUAL Err from eval-ast! on a bare fn-form.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [pred-src "(:wat::core::fn [n <- :wat::core::i64] -> :wat::core::bool (:wat::core::> n 3))"
     form     (:wat::core::match (:wat::core::read-string pred-src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     pure     (:wat::rete::pure? form)
     det      (:wat::rete::deterministic? form)
     res      (:wat::eval-ast! form)]
    (:wat::kernel::println (:wat::core::str pure))
    (:wat::kernel::println (:wat::core::str det))
    (:wat::kernel::println (:wat::core::str res))))
