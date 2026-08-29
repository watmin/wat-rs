;; `#holon [1 2 3]` quotes to a WatAST and lifts via `watast_to_holon`.
;; `(to-holon (Vector ...))` lifts a runtime Value::Vec, which classifies as "Vector".
;; Do the two paths agree? If not, one datum has two holons and every predicate over it is
;; path-dependent.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [via-tag  #holon [1 2 3]
     via-fn   (:wat::holon::to-holon (:wat::core::Vector :wat::core::i64 1 2 3))]
    (:wat::core::do
      (:wat::kernel::println "classifier via #holon:")   (:wat::kernel::println (:wat::holon::extract-classifier via-tag))
      (:wat::kernel::println "classifier via to-holon:") (:wat::kernel::println (:wat::holon::extract-classifier via-fn))
      (:wat::kernel::println "is-Vector? via #holon:")   (:wat::kernel::println (:wat::holon::is-Vector? via-tag))
      (:wat::kernel::println "is-Vector? via to-holon:") (:wat::kernel::println (:wat::holon::is-Vector? via-fn))
      (:wat::kernel::println "do the two holons COINCIDE?")
      (:wat::kernel::println (:wat::holon::coincident? via-tag via-fn)))))
