;; tests/wat_lang/probe_assert_true_false.wat
;; :wat::test::assert-true / assert-false assertion mechanism probes.

(:wat::core::defn :t::assert-true-on-true [] -> :wat::core::nil
  (:wat::core::do (:wat::test::assert-true (:wat::core::= 1 1)) nil))

(:wat::core::defn :t::assert-true-on-false [] -> :wat::core::nil
  (:wat::core::do (:wat::test::assert-true (:wat::core::= 1 2)) nil))

(:wat::core::defn :t::assert-false-on-false [] -> :wat::core::nil
  (:wat::core::do (:wat::test::assert-false (:wat::core::= 1 2)) nil))

(:wat::core::defn :t::assert-false-on-true [] -> :wat::core::nil
  (:wat::core::do (:wat::test::assert-false (:wat::core::= 1 1)) nil))
