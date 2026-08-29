;; Which spelling actually produces each classifier? A guessed literal is a ledger cell that
;; passes for the wrong reason.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "Set  <- #holon #{1 2}:")   (:wat::kernel::println (:wat::holon::is-Set? #holon #{1 2}))
    (:wat::kernel::println "Map  <- #holon {:a 1}:")   (:wat::kernel::println (:wat::holon::is-Map? #holon {:a 1}))
    (:wat::kernel::println "Vec  <- #holon [1 2]:")    (:wat::kernel::println (:wat::holon::is-Vector? #holon [1 2]))
    (:wat::kernel::println "Kw   <- #holon :a:")       (:wat::kernel::println (:wat::holon::is-Keyword? #holon :a))
    (:wat::kernel::println "Nil  <- #holon nil:")      (:wat::kernel::println (:wat::holon::is-Nil? #holon nil))
    (:wat::kernel::println "Sym  <- #holon nil:")      (:wat::kernel::println (:wat::holon::is-Symbol? #holon nil))
    (:wat::kernel::println "Tag  <- Bind/left of uuid holon:")
    (:wat::kernel::println
      (:wat::holon::is-Tag?
        (:wat::core::Option/expect (:wat::holon::Bind/left (:wat::holon::to-holon (:wat::core::Uuid/nil))) "uuid is a Bind")))))
