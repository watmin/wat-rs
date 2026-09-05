;; tests/program/wat_arc170_program_contracts_t5_launch_lambda.wat — spawn-program' (process) with inline forms.
;;
;; Arc 278 IPC de-prime — migrated off the non-prime `:wat::kernel::spawn-process` onto the
;; composed primes: `spawn-program' (process)` spawns the peer over the inline `(:wat::core::forms …)`
;; program, `send' 21` feeds the child's `readln`, and the doubled `println` crosses back to the
;; parent as a `recv'` `RecvOutcome::Message`. The CHILD BODY (readln n → println n*2 → nil) is
;; UNCHANGED from the old form; only the DRIVER flipped to the peer wire (spawn-process API →
;; spawn-program' / send' / recv'). Returns the recv'd i64 directly (== 42) so the test measures
;; the value that genuinely crossed the wire. Closest model: t18_echo_doubled.wat (SAME family).
(:wat::core::defn :my::launch [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [n    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                _out (:wat::kernel::println (:wat::i64::* n 2))]
               nil))))
     _ (:wat::core::match (:wat::kernel::send p 21)
         (:wat::kernel::SendOutcome::Sent nil)
         (:wat::kernel::SendOutcome::Closed nil)
         ;; arc 278 #73 — uniform, precondition is the recv' right below: a stop that
         ;; interrupted this write is still in force when the read parks, so the read
         ;; returns Stopped and the caller is told once, by the arm below.
         (:wat::kernel::SendOutcome::Stopped nil)
         ((:wat::kernel::SendOutcome::Lost _c) nil))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m) m)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "launch: stop requested before child sent its value — child was ALIVE, channel open" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "launch: child closed before sending its value" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
