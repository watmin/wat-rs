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
(:wat::core::defsurface :wat-tests::OffsetCounter :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :wat-tests::OffsetCounter::TotalRequest  [])
   (:wat::core::defrecord :wat-tests::OffsetCounter::TotalResponse [value <- :wat::core::i64])]
  :features
  [(total [self <- :wat-tests::OffsetCounter  req <- :wat-tests::OffsetCounter::TotalRequest] -> :wat-tests::OffsetCounter::TotalResponse)])

(:wat::service::defservice :wat-tests::offset-counter
  :satisfies :wat-tests::OffsetCounter
  :durable   [count <- :wat::core::i64]
  :ephemeral [base <- :wat::core::i64]
  :init (:wat::core::fn [record <- :wat-tests::offset-counter::Record
                         offset <- :wat::core::i64]
          -> :wat-tests::offset-counter::State
          (:wat-tests::offset-counter::State record offset))
  :impls
  [(total [s req]
     (:wat::service::Outcome::Reply s
       (:wat-tests::OffsetCounter::TotalResponse
         (:wat::core::i64::+
           (:wat-tests::offset-counter::Record/count (:wat-tests::offset-counter::State/durable s))
           (:wat-tests::offset-counter::State/base s)))))])

;; thread tier: start with (Record 5) + live offset 100 -> Total == 105 (durable.count 5 + ephemeral.base 100).
;; The offset is the second :init arg — the live operating-input the law exists for.
;; IGNORED pending arc 291 4b-iv-a: dispatch-admin applies init to ONE arg (service.wat:427); the
;; ship (Admin::Init/Resume) must carry the WHOLE init-arg tuple. RED-marker; 4b-iv-a un-ignores green.
(:wat::test::deftest' :wat-tests::service::multiparam-init-on-thread
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::offset-counter/start :locus (:wat::spawn::thread)
           :record (:wat-tests::offset-counter::Record 5) :offset 100)
       c (:wat::kernel::connect' (:wat-tests::offset-counter::Handle/addr h))
       r (:wat-tests::OffsetCounter/total c (:wat-tests::OffsetCounter::TotalRequest))]
      (:wat-tests::OffsetCounter::TotalResponse/value r))
    105))

;; process tier: identical except the locus — the live offset crosses the wire as EDN in Admin::Init.
(:wat::test::deftest' :wat-tests::service::multiparam-init-on-process
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::offset-counter/start :locus (:wat::spawn::process)
           :record (:wat-tests::offset-counter::Record 5) :offset 100)
       c (:wat::kernel::connect' (:wat-tests::offset-counter::Handle/addr h))
       r (:wat-tests::OffsetCounter/total c (:wat-tests::OffsetCounter::TotalRequest))]
      (:wat-tests::OffsetCounter::TotalResponse/value r))
    105))
