;; tests/program/probe_arc211_program_env_ambient.wat — co-located fixture for probe_arc211_program_env_ambient.rs,
;; slurped via startup_beside(file!()).

;; c02: construct Env with (started-at=5000, peer-started-at=6000) and return epoch-millis of peer-started-at.
(:wat::core::defn :probe::c02-compute [] -> :wat::core::i64
  (:wat::time::epoch-millis
    (:wat::program::Env/wat.peer-started-at
      (:wat::program::Env
        (:wat::time::at-millis 5000)
        (:wat::time::at-millis 6000)
        0 0 :wat::program::PeerKind::process 1
        (:wat::program::EmptyEnv)))))

;; c03: read started-at from the installed ambient env via (:wat::program::env).
(:wat::core::defn :probe::c03-compute [] -> :wat::core::i64
  (:wat::time::epoch-millis
    (:wat::program::Env/wat.started-at
      (:wat::program::env))))

;; build-env: construct a ProgramEnv with started-at=5000, peer-started-at=0 (for c03 setup).
(:wat::core::defn :probe::build-env [] -> :wat::program::Env
  (:wat::program::Env
    (:wat::time::at-millis 5000)
    (:wat::time::at-millis 0)
    0 0 :wat::program::PeerKind::process 1
    (:wat::program::EmptyEnv)))

;; c04: user::main that reads from the ambient env (proves invoke_user_main installs it).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::program::Env/wat.started-at (:wat::program::env))
    nil))
