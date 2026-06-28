;; tests/process/probe_arc259_spawn_host_opts.wat — co-located fixture for probe_arc259_spawn_host_opts.rs
;; startup_beside(file!()) world — (thread) and (process) hosting-door keys type-check + construct.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::spawn::thread)
    (:wat::spawn::process)
    nil))
