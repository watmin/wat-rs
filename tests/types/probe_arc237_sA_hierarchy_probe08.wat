;; Fixture probe 08: (subtype? :wat::core::Record :wat::holon::Record) → false (directional).
(:wat::core::defn :user::probe08 [] -> wat.type/bool
  (:wat::core::subtype? wat.type/Record :wat::holon::Record))
