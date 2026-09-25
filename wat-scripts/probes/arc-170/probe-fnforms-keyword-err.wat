(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [forms (:wat::kernel::fn-forms (:wat::keyword::from-name "no::such::fn") :x)]
    (:wat::kernel::println "should not reach here")))
