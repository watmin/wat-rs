;; tests/program/probe_arc258_program_env_record.wat — co-located fixture for probe_arc258_program_env_record.rs,
;; slurped via startup_beside(file!()).
;;
;; Arc 293 annihilation: :user::MyEnv (recordtype :wat::program::Env) and c02-compute deleted.
;; :wat::program::Env is a plain flat record; user-type inheritance is rejected at parse time.

;; c01: construct base program::Env with started-at=5000, read epoch-millis of started-at.
(:wat::core::defn :probe::c01-compute [] -> :wat::core::i64
  (:wat::time::epoch-millis
    (:wat::program::Env/wat.started-at
      (:wat::program::Env
        :wat.started-at (:wat::time::at-millis 5000)
        :wat.peer-started-at (:wat::time::at-millis 0)
        :wat.process-id 0 :wat.os-thread-id 0 :wat.peer-kind :wat::program::PeerKind::process :wat.cpu-count 1
        :user.program (:wat::program::EmptyEnv)))))
