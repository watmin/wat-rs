;; tests/program/probe_arc259_env_peer_kind.wat — co-located fixture for probe_arc259_env_peer_kind.rs,
;; slurped via startup_beside(file!()).

;; compute: construct Env with PeerKind::thread, check conforms? to :wat::program::PeerKind.
(:wat::core::defn :probe::compute [] -> :wat::core::bool
  (:wat::core::conforms?
    (:wat::program::Env/peer-kind
      (:wat::program::Env :started-at (:wat::time::now) :peer-started-at (:wat::time::now) :process-id 0 :os-thread-id 0
        :peer-kind :wat::program::PeerKind::thread :cpu-count 1 :user-data (:wat::program::EmptyEnv)))
    :wat::program::PeerKind))

;; seam: run by invoke_user_main; asserts the seam stamps :process for the root main.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::test::assert-eq
      (:wat::program::Env/peer-kind (:wat::program::env))
      :wat::program::PeerKind::process)
    nil))
