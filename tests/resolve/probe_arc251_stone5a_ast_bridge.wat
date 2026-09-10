(:wat::core::defn :user::c01 [] -> :wat::core::bool
  (:wat::core::List?
    (:wat::core::first
      (:wat::core::ast->children
        (:wat::core::match (:wat::core::read-string "((:a 1) (:b 2))") [:wat::core::ReadOutcome.Forms {:forms __forms} __forms] [:wat::core::ReadOutcome.Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])))))
(:wat::core::defn :user::c02 [] -> :wat::core::bool
  (:wat::core::List?
    (:wat::core::first
      (:wat::core::ast->children
        (:wat::core::first
          (:wat::core::ast->children
            (:wat::core::match (:wat::core::read-string "((:a 1) (:b 2))") [:wat::core::ReadOutcome.Forms {:forms __forms} __forms] [:wat::core::ReadOutcome.Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])))))))
