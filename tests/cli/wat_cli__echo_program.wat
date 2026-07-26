;; Minimal :user::main that echoes stdin to stdout — the hello-world of the
;; wat CLI. Exercises the canonical [] -> :nil signature (arc 170), kernel
;; readln / println EDN-only contract (arc 170 slice 1f-ι), crossbeam channel
;; wiring, stdio bridge threads, clean shutdown. See
;; tests/cli/wat_cli.rs::echo_program_reads_stdin_writes_stdout.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [line (:wat::kernel::readln)]
    (:wat::kernel::println line)))
