;; tests/kernel/spawn_program_prime_process_echo.wat — the proven arc112 echo+1
;; forms-server body, read from disk (never inlined) by
;; spawn_program_prime_process.rs and handed to `spawn_process_peer` as
;; `Vec<WatAST>` via `parse_all_with_file`. Reads one i64 from fd 0, writes
;; n+1 to fd 1.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [n (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    _ (:wat::kernel::println (:wat::i64::+ n 1))]
    nil))
