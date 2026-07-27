(:wat::core::defn :user::c01 [] -> :wat::core::bool
  (:wat::core::List?
    (:wat::core::match (:wat::core::read-string
      (:wat::core::write-forms
        (:wat::core::match (:wat::core::read-string
          "(:wat::core::defn :f [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ x 1))") ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))))
