;; tests/program/probe_arc259_env_identity.wat — co-located fixture for probe_arc259_env_identity.rs,
;; slurped via startup_beside(file!()).

;; c01-compute: construct Env with process-id=12345, return the process-id field.
(:wat::core::defn :probe::c01-compute [] -> :wat::core::i64
  (:wat::program::Env/wat.process-id
    (:wat::program::Env :wat.started-at (:wat::time::now) :wat.peer-started-at (:wat::time::now) :wat.process-id 12345 :wat.os-thread-id 67890
      :wat.peer-kind :wat::program::PeerKind::process :wat.cpu-count 1 :user.program (:wat::program::EmptyEnv))))

;; seam: run by invoke_user_main; reads both id fields for effect — accessor errors if
;; the seam did not stamp them.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::program::Env/wat.process-id (:wat::program::env))
    (:wat::program::Env/wat.os-thread-id (:wat::program::env))
    nil))
