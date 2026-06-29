;; tests/program/probe_arc259_user_program_slot.wat — co-located fixture for probe_arc259_user_program_slot.rs,
;; slurped via startup_beside(file!()).

;; compute: construct Env with EmptyEnv in user.program, check conforms? to :wat::core::Record.
(:wat::core::defn :probe::compute [] -> :wat::core::bool
  (:wat::core::conforms?
    (:wat::program::Env/user.program
      (:wat::program::Env (:wat::time::now) (:wat::time::now) 0 0
        :wat::program::PeerKind::process 1 (:wat::program::EmptyEnv)))
    :wat::core::Record))

;; seam: run by invoke_user_main; asserts user.program defaults to :wat::program::EmptyEnv.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::test::assert-true
      (:wat::core::conforms?
        (:wat::program::Env/user.program (:wat::program::env))
        :wat::program::EmptyEnv))
    nil))
