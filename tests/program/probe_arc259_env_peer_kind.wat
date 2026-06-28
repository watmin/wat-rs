;; tests/program/probe_arc259_env_peer_kind.wat — co-located fixture for probe_arc259_env_peer_kind.rs,
;; slurped via startup_beside(file!()).

;; compute: construct Env with PeerKind::thread, check conforms? to :wat::program::PeerKind.
(:wat::core::defn :probe::compute [] -> :wat::core::bool
  (:wat::core::conforms?
    (:wat::program::Env/wat.peer-kind
      (:wat::program::Env (:wat::time::now) (:wat::time::now) 0 0
        :wat::program::PeerKind::thread 1 (:wat::program::EmptyEnv)))
    :wat::program::PeerKind))

;; seam: run by invoke_user_main; asserts the seam stamps :process for the root main.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::test::assert-eq<:wat::program::PeerKind>
      (:wat::program::Env/wat.peer-kind (:wat::program::env))
      :wat::program::PeerKind::process)
    nil))
