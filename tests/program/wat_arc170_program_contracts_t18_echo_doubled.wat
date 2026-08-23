;; tests/program/wat_arc170_program_contracts_t18_echo_doubled.wat
;;
;; Arc 278 IPC de-prime — THE BIDIRECTIONAL PRIME EXEMPLAR.
;;
;; What `run-hermetic-with-io` did through Sender/from-pipe + Receiver/from-pipe
;; over a 4-field Process, the composed primes do directly: spawn a process peer
;; (`spawn-program' (process)`), feed its `readln` with a `send'`, and drain the
;; values its `println` emits off the peer via `recv'` until the peer closes.
;;
;; The CHILD BODY is unchanged from the run-hermetic-with-io form — `readln` → the
;; doubled `println` are FINAL io verbs; under `spawn-program'` the child's `readln`
;; is fed by the parent's `send'`, and each `println` arrives at the parent as a
;; `recv'` `RecvOutcome::Message` (proven: tests/kernel/wat_hermetic_round_trip.wat,
;; peer_verb_round_trip_process.wat).
;;
;; The recv'-drain is the SHARED primed helper `:wat::kernel::recv-all'` (arc 278 IPC
;; de-prime — this consumer is its canonical call site, minted alongside in wat/spawn.wat).
;; recv-all' drains the peer honestly and returns a `Result<Vector<O>, LociDiedError>`:
;;   Ok[outputs] -> the peer closed cleanly (RecvOutcome::Closed); the collected values.
;;   Err[cause]  -> the peer DIED (RecvOutcome::Lost); the LociDiedError rides in the Err,
;;                  surfaced here via `assertion-failed!`, NEVER swallowed.

;; Spawn a process peer whose :user::main readln's an i64 and println's it doubled;
;; send' 21 to feed the child's readln; drain the doubled outputs -> [42].
(:wat::core::defn :my::test::echo-doubled [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [n (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                _ (:wat::kernel::println (:wat::core::i64::* n 2))]
               nil))))
     _ (:wat::core::match (:wat::kernel::send p 21)
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
