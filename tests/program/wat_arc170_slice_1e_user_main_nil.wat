;; tests/program/wat_arc170_slice_1e_user_main_nil.wat — canonical :user::main fixture.
;; Canonical shape is [] -> :wat::core::nil; the body must DO something (arc-170 UselessMain wall).
;; Used for t1 (canonical freeze+invoke), t3 (argv ambient eval), t3 (current-thread eval).
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "canonical main ran"))
