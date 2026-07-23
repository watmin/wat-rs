;; tests/comms/probe_arc209_c0b1_thread_connection.wat — co-located fixture slurped via startup_beside(file!()).
;; Arc 209 Stone C0b.1 — thread-tier connection (listener' / connect' / accept').
;; Service accepts one client, doubles its number. Client sends 5, expects 10.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair  (:wat::kernel::listener' (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     l     (:wat::spawn::Bound/listener pair)
     addr  (:wat::spawn::Bound/address pair)
     svc   (:wat::kernel::spawn-program' (:wat::spawn::thread)
              (:wat::core::fn [_admin <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                (:wat::core::let
                  [conn (:wat::kernel::accept' l)
                   n    (:wat::core::match (:wat::kernel::recv' conn)
                          ((:wat::kernel::RecvOutcome::Message m) m)
                          ((:wat::kernel::RecvOutcome::Lost cause)
                            (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None))
                          (:wat::kernel::RecvOutcome::Closed
                            (:wat::kernel::assertion-failed! "recv': conn closed unexpectedly" :wat::core::None :wat::core::None)))
                   _    (:wat::core::match (:wat::kernel::send' conn (:wat::core::* n 2)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
                  nil)))
     conn  (:wat::kernel::connect' addr)
     _     (:wat::core::match (:wat::kernel::send' conn 5) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     reply (:wat::core::match (:wat::kernel::recv' conn)
             ((:wat::kernel::RecvOutcome::Message m) m)
             ((:wat::kernel::RecvOutcome::Lost cause)
               (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None))
             (:wat::kernel::RecvOutcome::Closed
               (:wat::kernel::assertion-failed! "recv': conn closed unexpectedly" :wat::core::None :wat::core::None)))]
    reply))

