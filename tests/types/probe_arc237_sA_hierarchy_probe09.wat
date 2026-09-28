;; Fixture probe 09: (subtype? :wat::core::i64 :wat::core::f64) → false (unrelated leaves).
(:wat::core::defn :user::probe09 [] -> wat.type/bool
  (:wat::core::subtype? :wat::core::i64 :wat::core::f64))
