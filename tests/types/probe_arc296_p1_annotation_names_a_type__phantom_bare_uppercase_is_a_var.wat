;; CONTROL — a bare Uppercase name is a type VARIABLE by the three-class rule,
;; auto-generalized by collect_free_type_vars. It must stay accepted even with no binder.
(:wat::core::defn :user::f [x <- :Whatever] -> :Whatever x)
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
