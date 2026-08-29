;; Item 7 step 2 — `is-List?` and `is-Tag?` were ALL-FALSE in the earlier confusion matrix, which
;; means "correct" OR "never fires" and cannot tell them apart. This asks each one directly, on a
;; value built by the constructor that is supposed to produce its shape.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "is-List? on (:wat::holon::List ...):")
    (:wat::kernel::println
      (:wat::holon::is-List? (:wat::holon::List (:wat::core::Vector :wat::holon::HolonAST #holon 1 #holon 2))))
    (:wat::kernel::println "is-Vector? on the same List (must be false if they partition):")
    (:wat::kernel::println
      (:wat::holon::is-Vector? (:wat::holon::List (:wat::core::Vector :wat::holon::HolonAST #holon 1 #holon 2))))
    (:wat::kernel::println "is-Tag? on a uuid-derived holon:")
    (:wat::kernel::println (:wat::holon::is-Tag? #holon "plain-string"))))
