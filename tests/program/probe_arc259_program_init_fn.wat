;; tests/program/probe_arc259_program_init_fn.wat — co-located fixture for probe_arc259_program_init_fn.rs,
;; slurped via startup_beside(file!()).

(:wat::core::defrecord :user::MyEnv [port <- :wat::core::i64])

;; compute-init: spawn a thread peer with (thread/init f) where f returns MyEnv{port:8080};
;; peer reads user-data's port and sends it back.
(:wat::core::defn :probe::compute-init [] -> :wat::core::i64
  (:wat::core::let
    [peer (:wat::test::spawn-peer
            (:wat::spawn::thread/init
              (:wat::core::fn [] -> :wat::core::Record (:user::MyEnv :port 8080)))
            (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
              (:wat::core::match
                (:wat::kernel::send self
                  (:user::MyEnv/port
                    (:wat::program::Env/user-data (:wat::program::env))))
                (:wat::kernel::SendOutcome::Sent nil)
                (:wat::kernel::SendOutcome::Closed nil)
                ;; arc 278 #73 — this is the worker's final send back to the parent;
                ;; a stop here is terminal for the worker either way, same as Closed.
                (:wat::kernel::SendOutcome::Stopped nil)
                ((:wat::kernel::SendOutcome::Lost _c) nil))))
     got (:wat::core::match (:wat::kernel::recv peer)
           ((:wat::kernel::RecvOutcome::Message m) m)
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))]
    got))

;; compute-error-init: spawn a thread peer with an init-fn that divides by zero —
;; the peer dies before sending. Arc 278 recv'-wall: recv' returns a matchable RecvOutcome VALUE
;; (never a raise) — the dead peer surfaces as ::Lost. We MATCH and RETURN the Lost cause's
;; `Failure/message` (the init-fn's crash reason) as a VALUE the .rs asserts.
(:wat::core::defn :probe::compute-error-init [] -> :wat::core::String
  (:wat::core::let
    [peer (:wat::test::spawn-peer
            (:wat::spawn::thread/init
              (:wat::core::fn [] -> :wat::core::Record
                (:wat::core::do (:wat::core::/ 1 0) (:wat::program::EmptyEnv))))
            (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
              (:wat::core::match (:wat::kernel::send self 7)
                (:wat::kernel::SendOutcome::Sent nil)
                (:wat::kernel::SendOutcome::Closed nil)
                ;; arc 278 #73 — this is the worker's final send back to the parent;
                ;; a stop here is terminal for the worker either way, same as Closed.
                (:wat::kernel::SendOutcome::Stopped nil)
                ((:wat::kernel::SendOutcome::Lost _c) nil))))]
    ;; The peer must be KILLED before it can send its 7 — recv' must NOT deliver a smuggled ::Message.
    ;; The init-fn crash dies before the post-spawn send: on this tier the peer exits before buffering a
    ;; crash reason, so it surfaces as ::Closed (a clean-EOF kill); a reason-carrying tier would surface
    ;; ::Lost. Both prove the kill — only a ::Message (the smuggled 7) is the failure. ::Stopped is not
    ;; expected here either (nothing in this test requests a stop); named distinctly, not folded in.
    (:wat::core::match (:wat::kernel::recv peer)
      ((:wat::kernel::RecvOutcome::Message _m) "SMUGGLED-VALUE")
      ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::LociDiedError/message cause))
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "PEER-DIED-CLOSED") (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; compute-default: spawn a plain (thread) peer — user-data defaults to EmptyEnv;
;; peer sends 1 if conforms?, else 0.
(:wat::core::defn :probe::compute-default [] -> :wat::core::i64
  (:wat::core::let
    [peer (:wat::test::spawn-peer (:wat::spawn::thread)
            (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
              (:wat::core::match
                (:wat::kernel::send self
                  (:wat::core::if
                    (:wat::core::conforms?
                      (:wat::program::Env/user-data (:wat::program::env))
                      :wat::program::EmptyEnv)
                    1 0))
                (:wat::kernel::SendOutcome::Sent nil)
                (:wat::kernel::SendOutcome::Closed nil)
                ;; arc 278 #73 — this is the worker's final send back to the parent;
                ;; a stop here is terminal for the worker either way, same as Closed.
                (:wat::kernel::SendOutcome::Stopped nil)
                ((:wat::kernel::SendOutcome::Lost _c) nil))))
     got (:wat::core::match (:wat::kernel::recv peer)
           ((:wat::kernel::RecvOutcome::Message m) m)
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))]
    got))

