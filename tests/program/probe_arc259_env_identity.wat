;; tests/program/probe_arc259_env_identity.wat — co-located fixture for probe_arc259_env_identity.rs,
;; slurped via startup_beside(file!()).

;; c01-compute: construct Env with process-id=12345, return the process-id field.
(:wat::core::defn :probe::c01-compute [] -> :wat::core::i64
  (:wat::program::Env/wat.process-id
    (:wat::program::Env (:wat::time::now) (:wat::time::now) 12345 67890
      :wat::program::PeerKind::process 1 (:wat::program::EmptyEnv))))

;; seam: run by invoke_user_main; reads both id fields for effect — accessor errors if
;; the seam did not stamp them.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::program::Env/wat.process-id (:wat::program::env))
    (:wat::program::Env/wat.os-thread-id (:wat::program::env))
    nil))
