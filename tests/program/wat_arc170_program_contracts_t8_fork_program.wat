;; tests/program/wat_arc170_program_contracts_t8_fork_program.wat — NEGATIVE: fork-program callsite.
;; Freeze must fail with BareLegacyForkProgram diagnostic.
(:wat::core::defn :user::main [stdin <- :wat::io::IOReader stdout <- :wat::io::IOWriter stderr <- :wat::io::IOWriter argv <- :wat::core::Vector<wat::core::String>] -> :wat::kernel::ExitCode
  (:wat::core::do
    (:wat::kernel::fork-program "" :wat::core::None)
    (:wat::core::u8 0)))
