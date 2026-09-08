;; RELAND-1 over-reach detector — a use!d :rust::* annotation must be ACCEPTED.
;; :rust::sqlite::Connection is in wat-rs defaults and is use!d by stdlib
;; (wat/sqlite.wat). It is NOT in TypeEnv / builtin_names / subtype_edges
;; (the 838-file false-positive pair). Store 3 is the only reason this is 0.
(:wat::core::use! :rust::sqlite::Connection)
(:wat::core::defn :user::f [c <- :rust::sqlite::Connection] -> :rust::sqlite::Connection c)
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
