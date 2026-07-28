;; Minimal :user::main that echoes stdin to stdout — the hello-world of the
;; wat CLI. Exercises the canonical [] -> :nil signature (arc 170), kernel
;; readln / println EDN-only contract (arc 170 slice 1f-ι), crossbeam channel
;; wiring, stdio bridge threads, clean shutdown. See
;; tests/cli/wat_cli.rs::echo_program_reads_stdin_writes_stdout.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [line (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))]
    (:wat::kernel::println line)))
