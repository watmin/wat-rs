;; tests/program/probe_arc259_peer_env_install.wat — co-located fixture for probe_arc259_peer_env_install.rs,
;; slurped via startup_beside(file!()).

;; compute-a: spawn a thread peer that sends its own os-thread-id back.
(:wat::core::defn :probe::compute-a [] -> :wat::core::i64
  (:wat::core::let [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
                           (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                             (:wat::kernel::send' self
                               (:wat::program::Env/wat.os-thread-id (:wat::program::env)))))
                    ;; arc 278 recv'-outcome wall — recv' returns a matchable RecvOutcome<i64>.
                    ;; OWNER role (the test is the final caller): ::Message m flows out as got;
                    ;; ::Lost/::Closed surface the cause loudly (eprintln, divergent-return).
                    r   (:wat::kernel::recv' peer)
                    got (:wat::core::match r
                          ((:wat::kernel::RecvOutcome::Message m) m)
                          ((:wat::kernel::RecvOutcome::Lost cause)
                            (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None))
                          (:wat::kernel::RecvOutcome::Closed
                            (:wat::kernel::assertion-failed! "recv': peer closed before sending its os-thread-id" :wat::core::None :wat::core::None)))]
    got))

;; compute-b: spawn a thread peer that sends 111 if its peer-kind is :thread, else 222.
(:wat::core::defn :probe::compute-b [] -> :wat::core::i64
  (:wat::core::let [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
                           (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                             (:wat::kernel::send' self
                               (:wat::core::if
                                 (:wat::core::= (:wat::program::Env/wat.peer-kind (:wat::program::env)) :wat::program::PeerKind::thread)
                                 111 222))))
                    ;; arc 278 recv'-outcome wall — OWNER role (test is the final caller).
                    r   (:wat::kernel::recv' peer)
                    got (:wat::core::match r
                          ((:wat::kernel::RecvOutcome::Message m) m)
                          ((:wat::kernel::RecvOutcome::Lost cause)
                            (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None))
                          (:wat::kernel::RecvOutcome::Closed
                            (:wat::kernel::assertion-failed! "recv': peer closed before sending its peer-kind" :wat::core::None :wat::core::None)))]
    got))

