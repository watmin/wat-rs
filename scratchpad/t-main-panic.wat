(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [_ (:wat::core::Result/expect
                        (:wat::eval-ast! (:wat::core::read-string "(:wat::core::this-verb-does-not-exist)"))
                        "boom at runtime")]
    nil))
