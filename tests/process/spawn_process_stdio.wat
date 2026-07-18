;; tests/process/spawn_process_stdio.wat — co-located fixture for spawn_process_stdio.rs
;; startup_beside(file!()) world — Row F: child calls println, parent reads Process/stdout.
;;
;; :my::launch spawns the child; the child prints the i64 value 42. The Rust driver fetches
;; :my::launch, applies it to get the Process value, then reads back the typed value directly
;; in Rust (no further wat evaluation is needed beyond the spawn itself).

(:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::nil,wat::core::i64>
  (:wat::kernel::spawn-process
    (:wat::core::forms
      (:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println 42)))))
