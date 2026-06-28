;; tests/program/wat_arc170_program_contracts_t11_legacy_main.wat — NEGATIVE: 3-arg main (IGNORED test).
;; freeze_err expected; currently freeze succeeds (walker-disconnect) hence #[ignore].
(:wat::core::defn :user::main [stdin <- :wat::io::IOReader stdout <- :wat::io::IOWriter stderr <- :wat::io::IOWriter] -> :wat::core::nil nil)
