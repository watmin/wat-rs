;; tests/program/probe_arc258_program_env_record.wat — co-located fixture for probe_arc258_program_env_record.rs,
;; slurped via startup_beside(file!()).

(:wat::core::recordtype :user::MyEnv :wat::program::Env [port <- :wat::core::i64])

;; c01: construct base program::Env with started-at=5000, read epoch-millis of started-at.
(:wat::core::defn :probe::c01-compute [] -> :wat::core::i64
  (:wat::time::epoch-millis
    (:wat::program::Env/wat.started-at
      (:wat::program::Env
        (:wat::time::at-millis 5000)
        (:wat::time::at-millis 0)
        0 0 :wat::program::PeerKind::process 1
        (:wat::program::EmptyEnv)))))

;; c02: extend program::Env with user field, construct the child record, read port.
(:wat::core::defn :probe::c02-compute [] -> :wat::core::i64
  (:user::MyEnv/port
    (:user::MyEnv
      (:wat::time::at-millis 1)
      (:wat::time::at-millis 0)
      0 0 :wat::program::PeerKind::process 1
      (:wat::program::EmptyEnv)
      8080)))
