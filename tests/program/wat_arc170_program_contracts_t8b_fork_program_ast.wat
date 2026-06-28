;; tests/program/wat_arc170_program_contracts_t8b_fork_program_ast.wat — NEGATIVE: fork-program-ast callsite.
;; Freeze must fail with BareLegacyForkProgram diagnostic.
(:wat::core::defn :user::main [stdin <- :wat::io::IOReader stdout <- :wat::io::IOWriter stderr <- :wat::io::IOWriter argv <- :wat::core::Vector<wat::core::String>] -> :wat::kernel::ExitCode
  (:wat::core::do
    (:wat::kernel::fork-program-ast (:wat::core::Vector :wat::WatAST))
    (:wat::core::u8 0)))
