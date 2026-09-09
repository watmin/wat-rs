;; HALF 1 CONTROL — the SAME annotation, with this program's own use!.
;; Accepted now, must stay accepted. This is the fixture that says the stone
;; narrows the SCOPE and does not ban the annotation.
(:wat::core::use! :rust::sqlite::Connection)
(:wat::core::defn :user::f [c <- :rust::sqlite::Connection] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
