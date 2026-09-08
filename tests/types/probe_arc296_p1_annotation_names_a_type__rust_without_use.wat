;; RELAND-1 over-reach detector — a :rust::* path with NO use! must still refuse.
;; :rust::test::Greeting is the floor's named false-positive (a wat_dispatch
;; FFI type). It is NOT in wat-rs defaults, not use!d here, not a TypeEnv
;; member. Accepting this would be a :rust:: prefix blanket (STOP-1).
(:wat::core::defn :user::f [g <- :rust::test::Greeting] -> :rust::test::Greeting g)
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
