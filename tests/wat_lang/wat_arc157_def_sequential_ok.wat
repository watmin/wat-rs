;; Fixture: test 2 — sequential def: :b references :a defined above.
(:wat::core::def :a 1)
(:wat::core::def :b (:wat::core::i64::+ :a 1))
