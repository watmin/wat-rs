;; ⛔ THE PAIRED CONTROL — the SAME question at a USER defn, with the same signature
;; shape the intrinsic declares. Green today; it is what proves the two paths differ
;; rather than that the value is wrong.
(:wat::core::defn :user::takes-opt :- [T] [o <- (:wat::core::Option :- [:T])] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::takes-opt (:wat::core::Option::Some {:value 42})))
