;; spawn-process child for stone_a T2 — prints two stdout lines + one stderr, then exits clean (nil).
;; Read by the probe and handed to build_spawn_process_call (a separate subprocess, not the parent world).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::kernel::println "line-one")
     _ (:wat::kernel::println "line-two")
     _ (:wat::kernel::eprintln "diag")]
    nil))
