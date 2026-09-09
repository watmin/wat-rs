;; HALF 1 SUBJECT — a USER annotation of a rust type this program never declared.
;; The stdlib use!s :rust::sqlite::Connection (wat/sqlite.wat:38), and the wall
;; consults the MERGED set, so it is accepted today. `resolve` refuses the same
;; name in call-head position; the wall must agree with resolve.
(:wat::core::defn :user::f [c <- :rust::sqlite::Connection] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
