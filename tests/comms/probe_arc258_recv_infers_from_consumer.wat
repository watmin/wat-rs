;; tests/comms/probe_arc258_recv_infers_from_consumer.wat — co-located fixture slurped via startup_beside(file!()).
;; Arc 258.5 — recv' infers its type from the constraining consumer (connect').
;; CHECK-LEVEL probe: (connect' (recv' svc)) with NO -> :T must type-check.
;; GREEN once connect' unifies its arg against Address'<fresh,fresh> so the fresh O binds.

(:wat::core::defn :user::compute [] -> :wat::core::nil
  (:wat::core::let
    [svc  (:wat::kernel::spawn-program' (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  [b    (:wat::kernel::listener' (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
                   addr (:wat::spawn::Bound/address b)
                   self (:wat::program::self-peer
                          :wat::kernel::Address'<wat::core::i64,wat::core::i64> :wat::core::i64)
                   _    (:wat::kernel::send' self addr)]
                  nil))))
     addr (:wat::kernel::recv' svc)
     c    (:wat::kernel::connect' addr)]
    nil))

