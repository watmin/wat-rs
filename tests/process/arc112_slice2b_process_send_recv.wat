;; tests/process/arc112_slice2b_process_send_recv.wat — co-located fixture for arc112_slice2b_process_send_recv.rs
;; startup_beside(file!()) world — typed-channel send + recv scheme wires through the type-checker
;; at the process boundary (Stone C shape: Sender/from-pipe + Receiver/from-pipe wrappers).

;; Child: Stone C contract — 0-arity, readln + println.
(:wat::core::defn :my::echo-worker
  [] -> :wat::core::nil
  (:wat::core::let
    [n (:wat::kernel::readln )
     _ (:wat::kernel::println (:wat::core::i64::+ n 1))]
    nil))

;; Parent: spawn-process + wrap pipes + send/recv via Stone C wrappers.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [proc (:wat::kernel::spawn-process
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:my::echo-worker))))
     tx   (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  proc))
     rx   (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout proc))
     _sent (:wat::core::Result/expect
             (:wat::kernel::send tx 41)
             "send failed")
     recv-result (:wat::kernel::recv rx)
     _val (:wat::core::match recv-result 
            ((:wat::core::Ok (:wat::core::Some v)) v)
            ((:wat::core::Ok :wat::core::None)    0)
            ((:wat::core::Err _)                  0))]
    nil))
