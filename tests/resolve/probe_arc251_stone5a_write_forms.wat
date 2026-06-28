(:wat::core::defn :user::c01 [] -> :wat::core::bool
  (:wat::core::List?
    (:wat::core::read-string
      (:wat::core::write-forms
        (:wat::core::read-string
          "(:wat::core::defn :f [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ x 1))")))))
