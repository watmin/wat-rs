;; tests/comms/probe_arc209_c0b1_thread_connection.wat — co-located fixture slurped via startup_beside(file!()).
;; Arc 209 Stone C0b.1 — thread-tier connection (listener' / connect' / accept').
;; Service accepts one client, doubles its number. Client sends 5, expects 10.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair  (:wat::kernel::listener' (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     l     (:wat::spawn::Bound/listener pair)
     addr  (:wat::spawn::Bound/address pair)
     svc   (:wat::kernel::spawn-program' (:wat::spawn::thread)
              (:wat::core::fn [_admin <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                (:wat::core::let
                  [conn (:wat::kernel::accept' l)
                   n    (:wat::kernel::recv' conn)
                   _    (:wat::kernel::send' conn (:wat::core::* n 2))]
                  nil)))
     conn  (:wat::kernel::connect' addr)
     _     (:wat::kernel::send' conn 5)
     reply (:wat::kernel::recv' conn)]
    reply))

