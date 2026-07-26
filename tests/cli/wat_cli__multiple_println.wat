;; :user::main calls println twice; stdout accumulates both writes. See
;; tests/cli/wat_cli.rs::program_writes_multiple_times_to_stdout.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "hello")
    (:wat::kernel::println "world")
    nil))
