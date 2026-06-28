;; tests/program/wat_arc170_program_contracts_t7_non_portable.wat — spawn-process capturing non-portable Sender.
;; Freeze may succeed (portability check fires at runtime) or fail (type-checker rejects); both are valid.
(:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::i64,wat::core::i64>
  (:wat::core::let
    [pair (:wat::kernel::make-channel :wat::core::nil)
     extra-tx (:wat::core::first pair)]
    (:wat::kernel::spawn-process
      (:wat::core::fn
        [rx <- :wat::kernel::Receiver<wat::core::i64>
         tx <- :wat::kernel::Sender<wat::core::i64>]
        -> :wat::core::nil
        (:wat::core::let
          [n
            (:wat::core::Option/expect -> :wat::core::i64
              (:wat::core::Result/expect -> :wat::core::Option<wat::core::i64>
                (:wat::kernel::recv rx)
                "recv failed")
              "stream closed")
           _send
            (:wat::core::Result/expect -> :wat::core::nil
              (:wat::kernel::send extra-tx n)
              "send failed")]
          :wat::core::nil)))))
