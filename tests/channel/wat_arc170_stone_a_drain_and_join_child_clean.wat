;; spawn-process child for stone_a T2 — prints three stdout lines, then exits clean (nil).
;; Read by the probe and handed to build_spawn_process_call (a separate subprocess, not the parent world).
;; Arc 278 no-hidden-failures: the incidental `(eprintln "diag")` here migrated to `println` — eprintln
;; is now a TERMINATING form (dying declaration), so it would crash the child and defeat this test's
;; sole contract (a CLEAN exit → Process/drain-and-join returns Ok(())). The stderr pipe now drains
;; empty-to-EOF, which still exercises the drain-both-pipes discipline.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::kernel::println "line-one")
     _ (:wat::kernel::println "line-two")
     _ (:wat::kernel::println "diag")]
    nil))
