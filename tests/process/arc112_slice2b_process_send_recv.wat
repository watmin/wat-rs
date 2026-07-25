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

;; Parent: spawn a process peer via spawn-program' (process), feed the child's
;; readln with send', drain its println output with recv' — the primed peer wire
;; (arc 278 IPC de-prime). What Stone C did through Sender/from-pipe + send +
;; Receiver/from-pipe + recv over a 4-field Process, the composed primes do
;; directly on the Process' peer. This probe verifies the primed comms scheme
;; wires through the type-checker at the process boundary (freeze-only).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [peer (:wat::kernel::spawn-program' (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:my::echo-worker))))
     _sent (:wat::core::match (:wat::kernel::send' peer 41)
             (:wat::kernel::SendOutcome::Sent nil)
             (:wat::kernel::SendOutcome::Closed nil)
             ((:wat::kernel::SendOutcome::Lost _c) nil))
     _val (:wat::core::match (:wat::kernel::recv' peer)
            ((:wat::kernel::RecvOutcome::Message _m) nil)
            ((:wat::kernel::RecvOutcome::Lost _cause) nil)
            (:wat::kernel::RecvOutcome::Closed nil))]
    nil))
