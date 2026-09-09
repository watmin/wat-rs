;; HALF 1 CONTROL — the stdlib's OWN :rust:: annotations must stay legal.
;; wat/sqlite.wat annotates :rust::sqlite::Connection and use!s it in the same
;; scope. If scope-awareness is drawn wrong, NOTHING loads and this is the first
;; fixture to say so.
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "loaded"))
