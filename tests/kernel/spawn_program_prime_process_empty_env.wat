;; tests/kernel/spawn_program_prime_process_empty_env.wat — the `env_fn` expression argument
;; spawn_program_prime_process.rs hands to the low-level `spawn_process_peer` Rust API. NOT a
;; startable program (no top-level defn/:user::main) — a single expression string the child
;; re-parses to build its `ProgramEnv`. Read from disk (never inlined) so no test carries a
;; string literal wat's own reader would accept.
(:wat::program::EmptyEnv)
