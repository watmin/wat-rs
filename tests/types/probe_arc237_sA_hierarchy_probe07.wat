;; Fixture probe 07: (subtype? :wat::holon::Record :wat::Record) → true.
(:wat::core::defn :user::probe07 [] -> :wat::core::bool
  (:wat::core::subtype? :wat::holon::Record :wat::Record))
