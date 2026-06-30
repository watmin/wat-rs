;; tests/program/probe_arc259_peer_env_install.wat — co-located fixture for probe_arc259_peer_env_install.rs,
;; slurped via startup_beside(file!()).

;; compute-a: spawn a thread peer that sends its own os-thread-id back.
(:wat::core::defn :probe::compute-a [] -> :wat::core::i64
  (:wat::core::let [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
                           (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                             (:wat::kernel::send' self
                               (:wat::program::Env/wat.os-thread-id (:wat::program::env)))))
                    got (:wat::kernel::recv' peer)]
    got))

;; compute-b: spawn a thread peer that sends 111 if its peer-kind is :thread, else 222.
(:wat::core::defn :probe::compute-b [] -> :wat::core::i64
  (:wat::core::let [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
                           (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                             (:wat::kernel::send' self
                               (:wat::core::if
                                 (:wat::core::= (:wat::program::Env/wat.peer-kind (:wat::program::env)) :wat::program::PeerKind::thread) -> :wat::core::i64
                                 111 222))))
                    got (:wat::kernel::recv' peer)]
    got))

