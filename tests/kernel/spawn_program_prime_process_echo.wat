;; tests/kernel/spawn_program_prime_process_echo.wat — the proven arc112 echo+1
;; forms-server body, read from disk (never inlined) by
;; spawn_program_prime_process.rs and handed to `spawn_process_peer` as
;; `Vec<WatAST>` via `parse_all_with_file`. Reads one i64 from fd 0, writes
;; n+1 to fd 1.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [n (:wat::kernel::readln -> :wat::core::i64)
                    _ (:wat::kernel::println (:wat::core::i64::+ n 1))]
    nil))
