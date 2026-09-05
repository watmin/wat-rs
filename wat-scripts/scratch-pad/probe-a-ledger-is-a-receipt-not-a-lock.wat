;; probe-a-ledger-is-a-receipt-not-a-lock.wat — the dead-owner hole, and its cure.
;;
;; Arc 278. `:fanout::seen` is written BEFORE the work is reported (circuit.wat:86-125), and
;; the worker emits only when it is told it was first (circuit.wat:491). So the ledger is a
;; LOCK. A lock with no release has one failure mode:
;;
;;   A claims seq -> First -> ledger holds seq for A -> A DIES before emitting.
;;   The message is redelivered (A never acked). B claims -> not-first -> B stands down.
;;   B acks anyway. NOBODY EMITTED. The message is consumed and gone.
;;
;; This is not hypothetical: 3 of 6 drop-after runs die at `claim deadline exhausted`, which
;; is a worker dying while holding claims. The run aborts before the loss can be counted.
;;
;; `DESIGN-the-unknowable-state` predicted the class in one line months ago:
;;   "The consumer claims before it emits, so a lost claim-reply converts at-least-once
;;    delivery into at-most-once processing."
;;
;; ⛔ THE DISCONFIRMING QUESTION. A lease was the obvious fix and is the wrong one: it needs a
;; clock and STILL cannot tell a SLOW owner from a DEAD one, so it trades loss for double-emit.
;; The question worth asking instead is whether moving the write to AFTER the report removes
;; the class outright:
;;
;;   CLAIM-BEFORE   write on claim, emit if first          -> a dead owner loses the message
;;   RECORD-AFTER   emit if not recorded, write after emit -> a dead owner costs a DUPLICATE
;;
;; Three scenarios, one service, one simulated death each. `emitted` is counted locally,
;; exactly as the circuit counts outcomes.
;;
;;   s1 CLAIM-BEFORE, A dies before emitting  -> expect emitted=0   ⛔ LOST
;;   s2 RECORD-AFTER, A dies before emitting  -> expect emitted=1   ✅ no loss
;;   s3 RECORD-AFTER, A dies AFTER emitting   -> expect emitted=2   ⚠ duplicate, NOT loss
;;
;; s3 is the honest half: record-after does not achieve exactly-once. It converts every LOSS
;; into a DUPLICATE, which is the irreducible window and precisely what this arc's own ruling
;; already calls the honest invariant — `distinct=N; dup >= 0`.

(:wat::config::set-redef! true)

(:wat::core::defsurface :dl::Ledger :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :dl::Ledger::ClaimRequest [seq <- :wat::core::String])
   (:wat::core::defenum :dl::Ledger::ClaimResponse :wat::enum::Pure
     :First [] :NotFirst []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :dl::Ledger::CheckRequest [seq <- :wat::core::String])
   (:wat::core::defenum :dl::Ledger::CheckResponse :wat::enum::Pure
     :Recorded [] :Absent []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :dl::Ledger::MarkRequest [seq <- :wat::core::String])
   (:wat::core::defenum :dl::Ledger::MarkResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(claim [self <- :dl::Ledger  req <- :dl::Ledger::ClaimRequest]
     -> :dl::Ledger::ClaimResponse :max-request-bytes 524288)
   (check [self <- :dl::Ledger  req <- :dl::Ledger::CheckRequest]
     -> :dl::Ledger::CheckResponse :max-request-bytes 524288)
   (mark [self <- :dl::Ledger  req <- :dl::Ledger::MarkRequest]
     -> :dl::Ledger::MarkResponse :max-request-bytes 524288)])

(:wat::service::defservice :dl::ledger
  :satisfies :dl::Ledger
  :durable   [n <- :wat::core::i64]
  ;; `locks` is the lock discipline (written on claim); `receipts` the receipt discipline
  ;; (written after the report). Same map type, opposite moment — the moment IS the stone.
  :ephemeral [locks    <- (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
              receipts <- (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])]
  :init (:wat::core::fn [record <- :dl::ledger::Record] -> :dl::ledger::State
          (:dl::ledger::State :durable record
            :locks (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
            :receipts (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])))
  :impls
  [(claim [s ctx req]
     (:wat::core::let
       [seq (:dl::Ledger::ClaimRequest/seq req)
        locks (:dl::ledger::State/locks s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:dl::Ledger::Reply])])
        alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:dl::ledger::Op])])
        held? (:wat::core::match (:wat::hashmap::get locks seq)
                ((:wat::core::Some _v) true) (:wat::core::None false))
        resp (:wat::core::if held?
               (:dl::Ledger::ClaimResponse::NotFirst) (:dl::Ledger::ClaimResponse::First))
        locks' (:wat::core::if held? locks (:wat::hashmap::assoc locks seq true))
        s' (:dl::ledger::State :durable (:dl::ledger::State/durable s)
             :locks locks' :receipts (:dl::ledger::State/receipts s))]
       (:wat::service::Outcome::Continue s'
         (:wat::core::Some (:dl::Ledger::Reply::Claim resp)) sends alarms)))
   (check [s ctx req]
     (:wat::core::let
       [seq (:dl::Ledger::CheckRequest/seq req)
        r (:dl::ledger::State/receipts s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:dl::Ledger::Reply])])
        alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:dl::ledger::Op])])
        resp (:wat::core::match (:wat::hashmap::get r seq)
               ((:wat::core::Some _v) (:dl::Ledger::CheckResponse::Recorded))
               (:wat::core::None (:dl::Ledger::CheckResponse::Absent)))]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:dl::Ledger::Reply::Check resp)) sends alarms)))
   (mark [s ctx req]
     (:wat::core::let
       [seq (:dl::Ledger::MarkRequest/seq req)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:dl::Ledger::Reply])])
        alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:dl::ledger::Op])])
        s' (:dl::ledger::State :durable (:dl::ledger::State/durable s)
             :locks (:dl::ledger::State/locks s)
             :receipts (:wat::hashmap::assoc (:dl::ledger::State/receipts s) seq true))]
       (:wat::service::Outcome::Continue s'
         (:wat::core::Some (:dl::Ledger::Reply::Mark (:dl::Ledger::MarkResponse::Ok)))
         sends alarms)))])

(:wat::core::defn :dl::claim [l <- :dl::Ledger  seq <- :wat::core::String] -> :wat::core::bool
  (:wat::core::match (:dl::Ledger/claim l (:dl::Ledger::ClaimRequest :seq seq))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:dl::Ledger::ClaimResponse::First) true)
        (_ false)))
    (_ false)))

(:wat::core::defn :dl::recorded? [l <- :dl::Ledger  seq <- :wat::core::String] -> :wat::core::bool
  (:wat::core::match (:dl::Ledger/check l (:dl::Ledger::CheckRequest :seq seq))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:dl::Ledger::CheckResponse::Recorded) true)
        (_ false)))
    (_ false)))

(:wat::core::defn :dl::mark! [l <- :dl::Ledger  seq <- :wat::core::String] -> :wat::core::nil
  (:wat::core::match (:dl::Ledger/mark l (:dl::Ledger::MarkRequest :seq seq))
    ((:wat::kernel::RecvOutcome::Message _r) nil)
    (_ nil)))

(:wat::core::defn :dl::await-ms [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::recv (:wat::kernel::after :wat::program::PeerKind::thread
                          (:wat::time::Milliseconds ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    (_ nil)))

(:wat::core::defn :dl::run [] -> :wat::core::String
  (:wat::core::let
    [h (:dl::ledger/start :locus (:wat::spawn::thread) :record (:dl::ledger::Record :n 0))
     l (:wat::core::match (:wat::kernel::connect (:dl::ledger::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "dl: dial failed" :wat::core::None :wat::core::None)))

     ;; s1 — CLAIM-BEFORE. A claims and dies before emitting. B is redelivered the message.
     a1 (:dl::claim l "s1")
     _w (:dl::await-ms 250)
     b1 (:dl::claim l "s1")
     e1 (:wat::core::+ (:wat::core::if a1 0 0) (:wat::core::if b1 1 0))

     ;; s2 — RECORD-AFTER. A checks, dies before emitting. B checks and emits.
     a2 (:dl::recorded? l "s2")
     b2 (:dl::recorded? l "s2")
     e2 (:wat::core::if b2 0 1)
     _m2 (:dl::mark! l "s2")

     ;; s3 — RECORD-AFTER. A checks, EMITS, dies before marking. B checks and emits again.
     a3 (:dl::recorded? l "s3")
     e3a (:wat::core::if a3 0 1)
     b3 (:dl::recorded? l "s3")
     e3b (:wat::core::if b3 0 1)
     _m3 (:dl::mark! l "s3")
     e3 (:wat::core::+ e3a e3b)]
    (:wat::core::format
      "s1-CLAIM-BEFORE emitted={a} ({av});s2-RECORD-AFTER-died-before-emit emitted={b} ({bv});s3-RECORD-AFTER-died-after-emit emitted={c} ({cv})"
      :a e1 :av (:wat::core::if (:wat::i64::= e1 0) "LOST" "ok")
      :b e2 :bv (:wat::core::if (:wat::i64::= e2 1) "no-loss" "WRONG")
      :c e3 :cv (:wat::core::if (:wat::i64::= e3 2) "duplicate-not-loss" "WRONG"))))

(:wat::core::defn :user::compute [] -> :wat::core::String (:dl::run))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println (:dl::run)))
