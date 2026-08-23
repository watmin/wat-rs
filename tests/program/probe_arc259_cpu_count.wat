;; tests/program/probe_arc259_cpu_count.wat — co-located fixture for probe_arc259_cpu_count.rs,
;; slurped via startup_beside(file!()).

;; compute: construct Env with cpu-count=8, return the stamped value.
(:wat::core::defn :probe::compute [] -> :wat::core::i64
  (:wat::program::Env/cpu-count
    (:wat::program::Env :started-at (:wat::time::now) :peer-started-at (:wat::time::now) :process-id 0 :os-thread-id 0
      :peer-kind :wat::program::PeerKind::process :cpu-count 8 :user-data (:wat::program::EmptyEnv))))

;; seam: run by invoke_user_main; asserts the env's cpu-count equals the live verb.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::test::assert-eq
      (:wat::program::Env/cpu-count (:wat::program::env))
      (:wat::program::cpu-count))
    nil))
