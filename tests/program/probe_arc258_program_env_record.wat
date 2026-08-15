;; tests/program/probe_arc258_program_env_record.wat — co-located fixture for probe_arc258_program_env_record.rs,
;; slurped via startup_beside(file!()).
;;
;; Arc 293 annihilation: :user::MyEnv (recordtype :wat::program::Env) and c02-compute deleted.
;; :wat::program::Env is a plain flat record; user-type inheritance is rejected at parse time.

;; c01: construct base program::Env with started-at=5000, read epoch-millis of started-at.
(:wat::core::defn :probe::c01-compute [] -> :wat::core::i64
  (:wat::time::epoch-millis
    (:wat::program::Env/started-at
      (:wat::program::Env
        :started-at (:wat::time::at-millis 5000)
        :peer-started-at (:wat::time::at-millis 0)
        :process-id 0 :os-thread-id 0 :peer-kind :wat::program::PeerKind::process :cpu-count 1
        :user-data (:wat::program::EmptyEnv)))))
