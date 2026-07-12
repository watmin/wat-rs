;; tests/comms/probe_arc272_6c2_record_ipc_derisk.wat — co-located fixture for the record IPC
;; de-risk probe, slurped via startup_beside(file!()). No placeholder main — startup_beside loads
;; defns only. The inner :user::main inside the forms block is the CHILD's entrypoint, not a
;; placeholder; it is kept.

(:wat::core::defrecord :user::Pt [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc (:wat::kernel::spawn-program' (:wat::spawn::process)
            (:wat::core::forms
              ;; The forked child runs a FRESH startup (stdlib prelude + these forms only) — it does
              ;; NOT inherit the parent's top-level defs. So the record must be defined HERE too (D1's
              ;; SocketAddressWire avoids this by living in spawn.wat/stdlib, loaded in every universe).
              (:wat::core::defrecord :user::Pt [x <- :wat::core::i64  y <- :wat::core::i64])
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  ;; the child mints a plain base record and hands it to the parent over the self-peer.
                  [self (:wat::program::self-peer :user::Pt :wat::core::i64)
                   _    (:wat::kernel::send' self (:user::Pt :x 7 :y 35))]
                  nil))))
     ;; the parent recv's the record off the lineage channel; reconstruct via the EDN wire.
     pt  (:wat::kernel::recv' svc)]
    (:wat::core::+ (:user::Pt/x pt) (:user::Pt/y pt))))
