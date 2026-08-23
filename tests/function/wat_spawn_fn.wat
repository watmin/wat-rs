;; tests/function/wat_spawn_fn.wat — positive fixture for spawn-thread tests.
;; Three distinct :my::compute_tN functions (no user::main needed for eval_in_frozen).
;;
;; Arc 259+ peer model: each worker drives its OWN self-peer — recv' an i64,
;; transform it, send' the result back. The parent spawns via spawn-program'
;; (:wat::spawn::thread), talks to the returned peer directly (send'/recv'), and
;; the peer reaps on scope-exit (RAII) — no explicit tx/rx or drain-and-join.

;; T1: named-define body — :app::increment worker + compute that spawns it by name.
(:wat::core::defn :app::increment [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
  (:wat::core::let
              [value
                (:wat::core::match (:wat::kernel::recv self)
                  ((:wat::kernel::RecvOutcome::Message m) m)
                  ((:wat::kernel::RecvOutcome::Lost cause)
                    (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                  (:wat::kernel::RecvOutcome::Stopped
                    (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                  (:wat::kernel::RecvOutcome::Closed
                    (:wat::kernel::assertion-failed! "recv': self closed unexpectedly" :wat::core::None :wat::core::None)))
               sum (:wat::core::i64::+ value 1)]
              (:wat::core::match (:wat::kernel::send self sum)
                (:wat::kernel::SendOutcome::Sent nil)
                (:wat::kernel::SendOutcome::Closed nil)
                ;; arc 278 #73 — this is the worker's final send back to the parent; a
                ;; stop here is terminal for the worker either way, same as Closed.
                (:wat::kernel::SendOutcome::Stopped nil)
                ((:wat::kernel::SendOutcome::Lost _c) nil))))

(:wat::core::defn :my::compute_t1 [] -> :wat::core::i64
  (:wat::core::let
              [peer
                (:wat::test::spawn-peer (:wat::spawn::thread) :app::increment)
               _ack
                (:wat::core::match (:wat::kernel::send peer 41)
                  (:wat::kernel::SendOutcome::Sent nil)
                  (:wat::kernel::SendOutcome::Closed nil)
                  ;; arc 278 #73 — uniform, and the precondition is the recv' right
                  ;; below: a stop that interrupted this write is still in force when
                  ;; the read parks, so the read returns Stopped and the caller is
                  ;; told once, by the arm below. Deciding here would decide it twice.
                  (:wat::kernel::SendOutcome::Stopped nil)
                  ((:wat::kernel::SendOutcome::Lost _c) nil))
               result
                (:wat::core::match (:wat::kernel::recv peer)
                  ((:wat::kernel::RecvOutcome::Message m) m)
                  ((:wat::kernel::RecvOutcome::Lost cause)
                    (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                  (:wat::kernel::RecvOutcome::Stopped
                    (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                  (:wat::kernel::RecvOutcome::Closed
                    (:wat::kernel::assertion-failed! "recv': peer closed unexpectedly" :wat::core::None :wat::core::None)))]
              result))

;; T2: inline fn literal body.
(:wat::core::defn :my::compute_t2 [] -> :wat::core::i64
  (:wat::core::let
              [peer
                (:wat::test::spawn-peer (:wat::spawn::thread)
                  (:wat::core::fn
                    [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])]
                     -> :wat::core::nil
                    (:wat::core::let
                      [value
                        (:wat::core::match (:wat::kernel::recv self)
                          ((:wat::kernel::RecvOutcome::Message m) m)
                          ((:wat::kernel::RecvOutcome::Lost cause)
                            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                          (:wat::kernel::RecvOutcome::Stopped
                            (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                          (:wat::kernel::RecvOutcome::Closed
                            (:wat::kernel::assertion-failed! "recv': self closed unexpectedly" :wat::core::None :wat::core::None)))
                       doubled (:wat::core::i64::* value 2)]
                      (:wat::core::match (:wat::kernel::send self doubled)
                        (:wat::kernel::SendOutcome::Sent nil)
                        (:wat::kernel::SendOutcome::Closed nil)
                        ;; arc 278 #73 — this is the worker's final send back to the
                        ;; parent; a stop here is terminal for the worker either way,
                        ;; same as Closed.
                        (:wat::kernel::SendOutcome::Stopped nil)
                        ((:wat::kernel::SendOutcome::Lost _c) nil)))))
               _ack
                (:wat::core::match (:wat::kernel::send peer 21)
                  (:wat::kernel::SendOutcome::Sent nil)
                  (:wat::kernel::SendOutcome::Closed nil)
                  ;; arc 278 #73 — uniform, and the precondition is the recv' right
                  ;; below: a stop that interrupted this write is still in force when
                  ;; the read parks, so the read returns Stopped and the caller is
                  ;; told once, by the arm below. Deciding here would decide it twice.
                  (:wat::kernel::SendOutcome::Stopped nil)
                  ((:wat::kernel::SendOutcome::Lost _c) nil))
               result
                (:wat::core::match (:wat::kernel::recv peer)
                  ((:wat::kernel::RecvOutcome::Message m) m)
                  ((:wat::kernel::RecvOutcome::Lost cause)
                    (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                  (:wat::kernel::RecvOutcome::Stopped
                    (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                  (:wat::kernel::RecvOutcome::Closed
                    (:wat::kernel::assertion-failed! "recv': peer closed unexpectedly" :wat::core::None :wat::core::None)))]
              result))

;; T3: closure capture — body captures `delta` from enclosing let.
(:wat::core::defn :my::compute_t3 [] -> :wat::core::i64
  (:wat::core::let
              [delta 100
               body
                (:wat::core::fn
                  [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])]
                   -> :wat::core::nil
                  (:wat::core::let
                    [n
                      (:wat::core::match (:wat::kernel::recv self)
                        ((:wat::kernel::RecvOutcome::Message v) v)
                        ((:wat::kernel::RecvOutcome::Lost cause)
                          (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                        (:wat::kernel::RecvOutcome::Stopped
                          (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                        (:wat::kernel::RecvOutcome::Closed
                          (:wat::kernel::assertion-failed! "recv': self closed unexpectedly" :wat::core::None :wat::core::None)))
                     sum (:wat::core::i64::+ n delta)]
                    (:wat::core::match (:wat::kernel::send self sum)
                      (:wat::kernel::SendOutcome::Sent nil)
                      (:wat::kernel::SendOutcome::Closed nil)
                      ;; arc 278 #73 — this is the worker's final send back to the
                      ;; parent; a stop here is terminal for the worker either way,
                      ;; same as Closed.
                      (:wat::kernel::SendOutcome::Stopped nil)
                      ((:wat::kernel::SendOutcome::Lost _c) nil))))
               peer
                (:wat::test::spawn-peer (:wat::spawn::thread) body)
               _ack
                (:wat::core::match (:wat::kernel::send peer 23)
                  (:wat::kernel::SendOutcome::Sent nil)
                  (:wat::kernel::SendOutcome::Closed nil)
                  ;; arc 278 #73 — uniform, and the precondition is the recv' right
                  ;; below: a stop that interrupted this write is still in force when
                  ;; the read parks, so the read returns Stopped and the caller is
                  ;; told once, by the arm below. Deciding here would decide it twice.
                  (:wat::kernel::SendOutcome::Stopped nil)
                  ((:wat::kernel::SendOutcome::Lost _c) nil))
               result
                (:wat::core::match (:wat::kernel::recv peer)
                  ((:wat::kernel::RecvOutcome::Message n) n)
                  ((:wat::kernel::RecvOutcome::Lost cause)
                    (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                  (:wat::kernel::RecvOutcome::Stopped
                    (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                  (:wat::kernel::RecvOutcome::Closed
                    (:wat::kernel::assertion-failed! "recv': peer closed unexpectedly" :wat::core::None :wat::core::None)))]
              result))
