;; tests/program/wat_arc170_program_contracts_t18c_recv_all_multi.wat
;;
;; Arc 278 IPC de-prime — THE recv-all' HELPER'S OWN GATE.
;;
;; t18 proves recv-all' on a single-output peer ([42]); this fixture proves the
;; "ALL" — a process peer that emits SEVERAL `println` values must be drained into
;; the FULL collected Vector, in order, before the clean-EOF `RecvOutcome::Closed`
;; turns into `(Ok outputs)`. The child readln's a seed `n` (fed by the parent's
;; `send'`), then `println`s n, n*2, n*3 as three separate messages and exits.
;; recv-all' drains all three -> `Ok [n 2n 3n]`; with n=7 -> [7 14 21].
;;
;; The Err arm surfaces the LociDiedError (never swallowed) exactly as t18 does —
;; the death, were the peer to die mid-drain, rides in the Result's Err.

(:wat::core::defn :my::test::echo-multi [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [n  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                _  (:wat::kernel::println n)
                _  (:wat::kernel::println (:wat::i64::* n 2))
                _  (:wat::kernel::println (:wat::i64::* n 3))]
               nil))))
     _ (:wat::core::match (:wat::kernel::send p 7)
         (:wat::kernel::SendOutcome::Sent nil)
         (:wat::kernel::SendOutcome::Closed nil)
         ;; arc 278 #73 — uniform, precondition is the recv-all' right below: a stop
         ;; that interrupted this write is still in force when the read parks, so the
         ;; drain returns Err[Stopped] and the caller is told once, by that Result.
         (:wat::kernel::SendOutcome::Stopped nil)
         ((:wat::kernel::SendOutcome::Lost _c) nil))]
    (:wat::core::match (:wat::kernel::recv-all p)
      ((:wat::core::Ok outputs) outputs)
      ((:wat::core::Err cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None)))))
