;; Fixture probe 08: (subtype? :wat::Record :wat::holon::Record) → false (directional).
(:wat::core::defn :user::probe08 [] -> :wat::core::bool
  (:wat::core::subtype? :wat::Record :wat::holon::Record))
