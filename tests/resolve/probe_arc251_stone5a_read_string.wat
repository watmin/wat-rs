(:wat::core::defn :user::c01 [] -> :wat::core::bool
  (:wat::core::List? (:wat::core::match (:wat::core::read-string "(:wat::core::i64::+ 1 2)") ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))))
(:wat::core::defn :user::c02 [] -> :wat::core::bool
  (:wat::core::List? (:wat::core::match (:wat::core::read-string
    "(:wat::core::defn :f [x <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64 0)") ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))))
