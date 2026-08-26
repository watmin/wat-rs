(:wat::core::defn :user::end-col [] -> :wat::core::i64
  (:wat::core::let
    [tree (:wat::core::match (:wat::core::read-string "(a b c)") ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     form (:wat::core::first (:wat::core::ast->children tree))
     espan (:wat::core::ast-end-span form)]
    (:wat::core::Option/expect
      (:wat::hashmap::get espan :col)
      "end :col")))
