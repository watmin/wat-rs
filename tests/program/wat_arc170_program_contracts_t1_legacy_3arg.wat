;; tests/program/wat_arc170_program_contracts_t1_legacy_3arg.wat — pre-arc-170 3-arg :user::main.
;; NEGATIVE fixture for IGNORED test t1_legacy_3arg_main_fires_walker: freeze should fail with
;; BareLegacyMainSignature; currently freeze succeeds (walker-disconnect), hence the #[ignore].
(:wat::core::defn :user::main [stdin <- :wat::io::IOReader stdout <- :wat::io::IOWriter stderr <- :wat::io::IOWriter] -> :wat::core::nil nil)
