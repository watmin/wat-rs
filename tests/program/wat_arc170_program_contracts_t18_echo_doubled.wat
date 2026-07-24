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
;; The recv'-drain is INLINE (a file-local tail-recursive `defn`) — no shared helper
;; is minted here; whether the drain earns a named primed library helper is a later
;; wave. The loop matches the recv'-outcome wall exactly:
;;   Message[v] -> collect + continue   (arc 278 the value, never a swallowed `_`)
;;   Closed     -> done (return the collected outputs)   (a GENUINE clean EOF)
;;   Lost[cause]-> surface the LociDiedError (never swallow a peer death)

;; Tail-recursive drain of a process peer's `println` stream into a Vector<i64>.
;; Reads until the child exits cleanly (peer EOF -> RecvOutcome::Closed).
(:wat::core::defn :my::test::drain-doubled
  [p   <- :wat::kernel::Process'<wat::core::i64,wat::core::i64>
   acc <- :wat::core::Vector<wat::core::i64>]
  -> :wat::core::Vector<wat::core::i64>
  (:wat::core::match (:wat::kernel::recv' p)
    ((:wat::kernel::RecvOutcome::Message v)
      (:my::test::drain-doubled p (:wat::core::conj acc v)))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed acc)))

;; Spawn a process peer whose :user::main readln's an i64 and println's it doubled;
;; send' 21 to feed the child's readln; drain the doubled outputs -> [42].
(:wat::core::defn :my::test::echo-doubled [] -> :wat::core::Vector<wat::core::i64>
  (:wat::core::let
    [p (:wat::kernel::spawn-program' (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [n (:wat::kernel::readln )
                _ (:wat::kernel::println (:wat::core::i64::* n 2))]
               nil))))
     _ (:wat::core::match (:wat::kernel::send' p 21)
         (:wat::kernel::SendOutcome::Sent nil)
         (:wat::kernel::SendOutcome::Closed nil)
         ((:wat::kernel::SendOutcome::Lost _c) nil))]
    (:my::test::drain-doubled p (:wat::core::Vector :wat::core::i64))))
