(:wat::core::defn :user::c01 [] -> :wat::core::bool
  (:wat::core::=
    (:wat::core::ast-name
      (:wat::core::first
        (:wat::core::ast->children (:wat::core::match (:wat::core::read-string "<-") ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))))
    "<-"))
(:wat::core::defn :user::c02 [] -> :wat::core::bool
  (:wat::core::=
    (:wat::core::ast-kind
      (:wat::core::first
        (:wat::core::ast->children (:wat::core::match (:wat::core::read-string ":-") ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))))
    "keyword"))
(:wat::core::defn :user::c03 [] -> :wat::core::bool
  (:wat::core::=
    (:wat::core::ast-name (:wat::core::symbol-node "wat.core/map"))
    "wat.core/map"))
(:wat::core::defn :user::c04 [] -> :wat::core::bool
  (:wat::core::=
    (:wat::core::ast-name (:wat::core::keyword-node ":-"))
    ":-"))
