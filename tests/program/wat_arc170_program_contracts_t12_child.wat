;; tests/program/wat_arc170_program_contracts_t12_child.wat
;; Child program for T12 (t12_spawn_process_child_emits_without_recv):
;; emits one line without reading rx first.
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "hello-from-fork"))
