;; tests/program/wat_arc170_program_contracts.wat — canonical trivial :user::main (PARENT_TRIVIAL).
;; Canonical shape [] -> :nil; body must DO something (arc-170 UselessMain wall).
;; Used by t1 (canonical), t2 (nil value), t4/t12-t16 (spawn-process parent world) via startup_beside(file!()).
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "parent trivial main"))
