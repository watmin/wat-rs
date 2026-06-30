;; tests/program/probe_arc259_program_init_fn.wat — co-located fixture for probe_arc259_program_init_fn.rs,
;; slurped via startup_beside(file!()).

(:wat::core::defrecord :user::MyEnv [port <- :wat::core::i64])

;; compute-init: spawn a thread peer with (thread/init f) where f returns MyEnv{port:8080};
;; peer reads user.program's port and sends it back.
(:wat::core::defn :probe::compute-init [] -> :wat::core::i64
  (:wat::core::let
    [peer (:wat::kernel::spawn-program'
            (:wat::spawn::thread/init
              (:wat::core::fn [] -> :wat::core::Record (:user::MyEnv 8080)))
            (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
              (:wat::kernel::send' self
                (:user::MyEnv/port
                  (:wat::program::Env/user.program (:wat::program::env))))))
     got (:wat::kernel::recv' peer)]
    got))

;; compute-error-init: spawn a thread peer with an init-fn that divides by zero —
;; the peer dies before sending, so recv' raises (compute errors).
(:wat::core::defn :probe::compute-error-init [] -> :wat::core::i64
  (:wat::core::let
    [peer (:wat::kernel::spawn-program'
            (:wat::spawn::thread/init
              (:wat::core::fn [] -> :wat::core::Record
                (:wat::core::do (:wat::core::/ 1 0) (:wat::program::EmptyEnv))))
            (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
              (:wat::kernel::send' self 7)))
     got (:wat::kernel::recv' peer)]
    got))

;; compute-default: spawn a plain (thread) peer — user.program defaults to EmptyEnv;
;; peer sends 1 if conforms?, else 0.
(:wat::core::defn :probe::compute-default [] -> :wat::core::i64
  (:wat::core::let
    [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
            (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
              (:wat::kernel::send' self
                (:wat::core::if
                  (:wat::core::conforms?
                    (:wat::program::Env/user.program (:wat::program::env))
                    :wat::program::EmptyEnv) -> :wat::core::i64
                  1 0))))
     got (:wat::kernel::recv' peer)]
    got))

