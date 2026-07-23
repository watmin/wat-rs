;; tests/process/spawn_process_stdin.wat — co-located fixture for spawn_process_stdin.rs
;; startup_beside(file!()) world — Row G: parent writes to Process/stdin, child reads via readln.
;;
;; :my::launch spawns the child; the child reads one i64 via readln, adds 1, prints via println.
;; The Rust driver fetches :my::launch, applies it to get the Process value, then drives the
;; typed-send/typed-recv interaction and process-exit assertions directly in Rust (no further
;; wat evaluation is needed beyond the spawn itself).

(:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::i64,wat::core::i64>
  (:wat::kernel::spawn-process
    (:wat::core::forms
      (:wat::core::defn :user::main [] -> :wat::core::nil
        (:wat::core::let
          [n    (:wat::kernel::readln )
           _out (:wat::kernel::println (:wat::core::i64::+ n 1))]
          nil)))))
