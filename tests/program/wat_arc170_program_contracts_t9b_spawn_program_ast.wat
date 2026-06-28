;; tests/program/wat_arc170_program_contracts_t9b_spawn_program_ast.wat — NEGATIVE: spawn-program-ast callsite.
;; Freeze must fail with BareLegacySpawnProgram diagnostic.
(:wat::core::defn :user::main [stdin <- :wat::io::IOReader stdout <- :wat::io::IOWriter stderr <- :wat::io::IOWriter argv <- :wat::core::Vector<wat::core::String>] -> :wat::kernel::ExitCode
  (:wat::core::do
    (:wat::kernel::spawn-program-ast (:wat::core::Vector :wat::WatAST) :wat::core::None)
    (:wat::core::u8 0)))
