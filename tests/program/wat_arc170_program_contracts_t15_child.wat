;; tests/program/wat_arc170_program_contracts_t15_child.wat
;; Child program for T15 (t15_spawn_process_child_panic_disconnects_recv_and_exits_nonzero):
;; panics intentionally before printing anything (Option/expect on None).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::Option/expect -> :wat::core::nil
              :wat::core::None
              "intentional panic in child"))
