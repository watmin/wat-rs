;; tests/comms/probe_arc272_6a_capability_handoff.wat — co-located fixture for the capability
;; handoff probe, slurped via startup_beside(file!()). No placeholder main at the top level.
;; The inner :user::main inside the forms block is the CHILD's entrypoint — it is kept.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc  (:wat::kernel::spawn-program' (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  ;; the child mints its OWN rendezvous: autobind, no name (step 2b).
                  [b    (:wat::kernel::listener' (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
                   addr (:wat::spawn::Bound/address b)
                   ;; the self-peer carries the Address' capability child->parent (S = Address').
                   self (:wat::program::self-peer
                          :wat::kernel::Address'<wat::core::i64,wat::core::i64> :wat::core::i64)
                   ;; hand the parent the capability — the lock-step handoff (it now has perfect knowledge).
                   _    (:wat::kernel::send' self addr)
                   ;; accept the parent's dial on our own listener; round-trip n -> n+100.
                   c    (:wat::kernel::accept' (:wat::spawn::Bound/listener b))
                   n    (:wat::kernel::recv' c)
                   _    (:wat::kernel::send' c (:wat::core::+ n 100))]
                  nil))))
     ;; recv' the child's minted capability over the lineage channel (blocks until the child sends it).
     addr (:wat::kernel::recv' svc)
     ;; dial the capability — the child is guaranteed listening (it sent AFTER listen()).
     c    (:wat::kernel::connect' addr)
     _    (:wat::kernel::send' c 5)
     got  (:wat::kernel::recv' c)]
    got))
