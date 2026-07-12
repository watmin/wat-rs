;; tests/program/probe_arc259_cpu_count.wat — co-located fixture for probe_arc259_cpu_count.rs,
;; slurped via startup_beside(file!()).

;; compute: construct Env with wat.cpu-count=8, return the stamped value.
(:wat::core::defn :probe::compute [] -> :wat::core::i64
  (:wat::program::Env/wat.cpu-count
    (:wat::program::Env :wat.started-at (:wat::time::now) :wat.peer-started-at (:wat::time::now) :wat.process-id 0 :wat.os-thread-id 0
      :wat.peer-kind :wat::program::PeerKind::process :wat.cpu-count 8 :user.program (:wat::program::EmptyEnv))))

;; seam: run by invoke_user_main; asserts the env's cpu-count equals the live verb.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::test::assert-eq<:wat::core::i64>
      (:wat::program::Env/wat.cpu-count (:wat::program::env))
      (:wat::program::cpu-count))
    nil))
