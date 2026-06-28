;; Negative fixture: tests 11 + 15 — redef of :a without set-redef! → DefRedefForbidden.
(:wat::core::def :a 1)
(:wat::core::def :a 2)
