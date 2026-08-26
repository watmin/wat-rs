;; Fixture: test 2 — sequential def: :b references :a defined above.
(:wat::core::def :wat-arc157-def-sequential-ok::a 1)
(:wat::core::def :wat-arc157-def-sequential-ok::b (:wat::i64::+ :wat-arc157-def-sequential-ok::a 1))
