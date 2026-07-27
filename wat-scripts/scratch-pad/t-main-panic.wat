(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [_ (:wat::core::Result/expect
                        (:wat::eval-ast! (:wat::core::match (:wat::core::read-string "(:wat::core::this-verb-does-not-exist)") ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))
                        "boom at runtime")]
    nil))
