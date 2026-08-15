;; tests/program/probe_arc259_env_identity.wat — co-located fixture for probe_arc259_env_identity.rs,
;; slurped via startup_beside(file!()).

;; c01-compute: construct Env with process-id=12345, return the process-id field.
(:wat::core::defn :probe::c01-compute [] -> :wat::core::i64
  (:wat::program::Env/process-id
    (:wat::program::Env :started-at (:wat::time::now) :peer-started-at (:wat::time::now) :process-id 12345 :os-thread-id 67890
      :peer-kind :wat::program::PeerKind::process :cpu-count 1 :user-data (:wat::program::EmptyEnv))))

;; seam: run by invoke_user_main; reads both id fields for effect — accessor errors if
;; the seam did not stamp them.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::program::Env/process-id (:wat::program::env))
    (:wat::program::Env/os-thread-id (:wat::program::env))
    nil))
