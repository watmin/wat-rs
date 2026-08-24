;; tests/program/wat_arc170_program_contracts_t6_launch_factory.wat — spawn-program' (process) with quasiquote + unquote.
;;
;; Arc 278 IPC de-prime — migrated off the non-prime `:wat::kernel::spawn-process` onto the
;; composed primes. The runtime-built child program (a `(:wat::core::Vector :wat::WatAST main-form)`
;; quasiquote factory that splices `~offset` into the child body) is handed to `spawn-program'
;; (process)` the SAME way it was handed to spawn-process — the process clause accepts a forms
;; VALUE (`(Vector :- [wat::WatAST])`), so the factory shape is preserved unchanged. Only the DRIVER
;; flipped to the peer wire: `send' 7` feeds the child's `readln`, and `(n + offset)`'s `println`
;; crosses back as a `recv'` `RecvOutcome::Message`. Returns the recv'd i64 directly (== offset+7).
(:wat::core::defn :my::launch [offset <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [main-form `(:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [n    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                     _out (:wat::kernel::println
                            (:wat::core::i64::+ n ~offset))]
                    nil))
     p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::Vector :wat::WatAST main-form))
     _ (:wat::core::match (:wat::kernel::send p 7)
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
        (:wat::kernel::assertion-failed! "launch: child closed before sending its value" :wat::core::None :wat::core::None)))))
