;; wat-tests/service-multiparam-init.wat — arc 291 4b-iv-a: multi-param :init (the foundation).
;;
;; THE LAW: :init is (Record, …operating-inputs) -> State. The FIRST param is the durable Record
;; (mandatory, always present even when empty); params 2+ are live operating-inputs provided FRESH
;; by start/resume (addresses, config — never durable). This probe ISOLATES the law from contract-
;; distribution (NO :calls): an offset-counter whose :init takes its Record AND a live i64 `offset`
;; (an operating-input, stored in :ephemeral, NOT on the durable record).
;;
;; RED at HEAD: start-body ships only the FIRST init arg (ship-ref); Admin::Init carries ONE field;
;; dispatch-admin applies init to ONE value -> a 2-param init is an arity mismatch (offset dropped).
;; GREEN after 4b-iv-a: Admin::Init/Resume carry the WHOLE init-arg tuple; dispatch-admin applies
;; init to all of them; start/resume thread all init params.

;; ── the surface (the counter protocol, lifted) ───────────────────────────────
;; arc 278 S4c: the surface OWNS its protocol messages (:messages) so a :satisfies
;; service ships them across a process fork.
(:wat::core::defsurface :wat-tests::OffsetCounter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat-tests::OffsetCounter::TotalRequest  [])
   (:wat::core::defenum :wat-tests::OffsetCounter::TotalResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(total [self <- :wat-tests::OffsetCounter  req <- :wat-tests::OffsetCounter::TotalRequest] -> :wat-tests::OffsetCounter::TotalResponse :max-request-bytes 524288)])

(:wat::service::defservice :wat-tests::offset-counter
  :satisfies :wat-tests::OffsetCounter
  :durable   [count <- :wat::core::i64]
  :ephemeral [base <- :wat::core::i64]
  :init (:wat::core::fn [record <- :wat-tests::offset-counter::Record
                         offset <- :wat::core::i64]
          -> :wat-tests::offset-counter::State
          (:wat-tests::offset-counter::State :durable record :base offset))
  :impls
  [(total [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:wat-tests::OffsetCounter::Reply::Total (:wat-tests::OffsetCounter::TotalResponse::Ok
         (:wat::i64::+
           (:wat-tests::offset-counter::Record/count (:wat-tests::offset-counter::State/durable s))
           (:wat-tests::offset-counter::State/base s))))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat-tests::OffsetCounter::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:wat-tests::offset-counter::Op])])))])

;; thread tier: start with (Record 5) + live offset 100 -> Total == 105 (durable.count 5 + ephemeral.base 100).
;; The offset is the second :init arg — the live operating-input the law exists for.
;; IGNORED pending arc 291 4b-iv-a: dispatch-admin applies init to ONE arg (service.wat:427); the
;; ship (Admin::Init/Resume) must carry the WHOLE init-arg tuple. RED-marker; 4b-iv-a un-ignores green.
(:wat::test::deftest :wat-tests::service::multiparam-init-on-thread
  
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::offset-counter/start :locus (:wat::spawn::thread)
           :record (:wat-tests::offset-counter::Record :count 5) :offset 100)
       c (:wat::core::match (:wat::kernel::connect (:wat-tests::offset-counter::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       r (:wat-tests::OffsetCounter/total c (:wat-tests::OffsetCounter::TotalRequest))]
      (:wat::core::match r ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv  
        ((:wat-tests::OffsetCounter::TotalResponse::Ok value) value)
        ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
        ((:wat-tests::OffsetCounter::TotalResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "offset-counter-total: unexpected RequestTooLarge"
            :wat::core::None :wat::core::None))
        ((:wat-tests::OffsetCounter::TotalResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))
    105))

;; process tier: identical except the locus — the live offset crosses the wire as EDN in Admin::Init.
(:wat::test::deftest :wat-tests::service::multiparam-init-on-process
  
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::offset-counter/start :locus (:wat::spawn::process)
           :record (:wat-tests::offset-counter::Record :count 5) :offset 100)
       c (:wat::core::match (:wat::kernel::connect (:wat-tests::offset-counter::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       r (:wat-tests::OffsetCounter/total c (:wat-tests::OffsetCounter::TotalRequest))]
      (:wat::core::match r ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv  
        ((:wat-tests::OffsetCounter::TotalResponse::Ok value) value)
        ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
        ((:wat-tests::OffsetCounter::TotalResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "offset-counter-total: unexpected RequestTooLarge"
            :wat::core::None :wat::core::None))
        ((:wat-tests::OffsetCounter::TotalResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))
    105))
