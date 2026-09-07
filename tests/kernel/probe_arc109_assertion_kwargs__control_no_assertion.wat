;; CONTROL — no assertion-failed! at all. Proves the harness, the binary and the
;; fixture path work, so a RED row below cannot be "everything is red".
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "control"))
