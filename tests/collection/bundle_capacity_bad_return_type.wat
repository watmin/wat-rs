;; NEGATIVE fixture — must fail at type-check.
;; :my::probe declares HolonAST return but Bundle returns (Result :- [HolonAST CapacityExceeded]).
(:wat::core::defn :my::probe [] -> :wat::holon::HolonAST
  (:wat::holon::Bundle (:wat::core::Vector :wat::holon::HolonAST
    (:wat::holon::to-holon "a")
    (:wat::holon::to-holon "b"))))

