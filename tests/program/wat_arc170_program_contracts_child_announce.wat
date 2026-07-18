;; tests/program/wat_arc170_program_contracts_child_announce.wat
;; Shared child program for T13/T14/T16 (clean-exit-on-tx-drop, idempotent
;; wait, and sequential-spawn-no-leak): announces itself on stdout, then
;; returns nil (idle worker — no rx read).
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "spawned child"))
