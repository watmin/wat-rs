(:wat::core::defn :user::c01 [] -> :wat::core::i64
  (:wat::core::Option/expect
    (:wat::hashmap::get
      (:wat::core::ast-span
        (:wat::core::first (:wat::core::ast->children
          (:wat::core::first (:wat::core::ast->children
            (:wat::core::match (:wat::core::read-string "(:wat::core::map x)") ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))))))
      :line)
    "field"))
(:wat::core::defn :user::c02 [] -> :wat::core::i64
  (:wat::core::Option/expect
    (:wat::hashmap::get
      (:wat::core::ast-span
        (:wat::core::first (:wat::core::ast->children
          (:wat::core::first (:wat::core::ast->children
            (:wat::core::match (:wat::core::read-string "(:wat::core::map x)") ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))))))
      :col)
    "field"))
(:wat::core::defn :user::c03 [] -> :wat::core::i64
  (:wat::core::Option/expect
    (:wat::hashmap::get
      (:wat::core::ast-span
        (:wat::core::first (:wat::core::rest (:wat::core::ast->children
          (:wat::core::first (:wat::core::ast->children
            (:wat::core::match (:wat::core::read-string "(:wat::core::map x)") ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))))))))
      :col)
    "field"))
