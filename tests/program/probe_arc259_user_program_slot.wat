;; tests/program/probe_arc259_user_program_slot.wat — co-located fixture for probe_arc259_user_program_slot.rs,
;; slurped via startup_beside(file!()).

;; compute: construct Env with EmptyEnv in user-data, check conforms? to :wat::core::Record.
(:wat::core::defn :probe::compute [] -> :wat::core::bool
  (:wat::core::conforms?
    (:wat::program::Env/user-data
      (:wat::program::Env :started-at (:wat::time::now) :peer-started-at (:wat::time::now) :process-id 0 :os-thread-id 0
        :peer-kind :wat::program::PeerKind::process :cpu-count 1 :user-data (:wat::program::EmptyEnv)))
    :wat::core::Record))

;; seam: run by invoke_user_main; asserts user-data defaults to :wat::program::EmptyEnv.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::test::assert-true
      (:wat::core::conforms?
        (:wat::program::Env/user-data (:wat::program::env))
        :wat::program::EmptyEnv))
    nil))
