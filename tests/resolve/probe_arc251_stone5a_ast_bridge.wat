(:wat::core::defn :user::c01 [] -> :wat::core::bool
  (:wat::core::List?
    (:wat::core::first
      (:wat::core::ast->children
        (:wat::core::match (:wat::core::read-string "((:a 1) (:b 2))") ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))))))
(:wat::core::defn :user::c02 [] -> :wat::core::bool
  (:wat::core::List?
    (:wat::core::first
      (:wat::core::ast->children
        (:wat::core::first
          (:wat::core::ast->children
            (:wat::core::match (:wat::core::read-string "((:a 1) (:b 2))") ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))))))))
