;; wat-tests/service-init-parity.wat — arc 291 strike-1 RED probe: the `:init` keystone, both loci.
;;
;; THE PROPHECY, proven small: a service whose State is built by an `:init` callback FROM EDN ARGS,
;; run IN-LOCUS — so `start` takes an EDN seed (42), not a pre-built State. ONE defservice, two
;; deftests differing in EXACTLY one token (the locus). Modeled byte-for-byte on the GREEN
;; `service-locus-parity.wat`; the ONLY addition is the `:init` clause.
;;
;; arc 291 4b-ii: State is now a defstruct; :durable [count] mints ::Record; ::State holds it.
;; :init now defaults to (fn [d <- ::Record] -> ::State (::State d)) for pure-data services.
;; start takes a ::Record (not a raw i64). The "seeded" semantics now live in start taking the
;; record: (seeded-counter/start locus (seeded-counter::Record 42)).
;; Op body reads count through State/durable.

;; ── the surface (the counter protocol, lifted) ───────────────────────────────
;; arc 278 S4c: the surface OWNS its protocol messages (:messages) so a :satisfies
;; service ships them across a process fork.
(:wat::core::defsurface :wat-tests::SeededCounter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat-tests::SeededCounter::GetRequest  [])
   (:wat::core::defenum :wat-tests::SeededCounter::GetResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(get [self <- :wat-tests::SeededCounter  req <- :wat-tests::SeededCounter::GetRequest] -> :wat-tests::SeededCounter::GetResponse :max-request-bytes 524288)])

;; ── the service, defined once at top-level (shared by both deftests) ──────────
;; :init defaults — pure-data service, ephemeral empty → default init = (fn [d <- ::Record] -> ::State (::State d))
(:wat::service::defservice :wat-tests::seeded-counter
  :satisfies :wat-tests::SeededCounter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(get [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat-tests::SeededCounter::GetResponse::Ok
         (:wat-tests::seeded-counter::Record/count (:wat-tests::seeded-counter::State/durable s)))))])

;; ── thread tier ──────────────────────────────────────────────────────────────
;; start takes the Record (seeded-counter::Record 42); init defaults to State/new(d).
(:wat::test::deftest :wat-tests::service::seeded-counter-on-thread
  
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::seeded-counter/start :locus (:wat::spawn::thread) :record (:wat-tests::seeded-counter::Record :count 42))
       c (:wat::core::match (:wat::kernel::connect (:wat-tests::seeded-counter::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       r (:wat-tests::SeededCounter/get c (:wat-tests::SeededCounter::GetRequest))]
      (:wat::core::match r ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv  
        ((:wat-tests::SeededCounter::GetResponse::Ok value) value)
        ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
        ((:wat-tests::SeededCounter::GetResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "seeded-counter-get: unexpected RequestTooLarge"
            :wat::core::None :wat::core::None))
        ((:wat-tests::SeededCounter::GetResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))
    42))

;; ── process tier — IDENTICAL except the locus token ──────────────────────────
;; the Record crosses the wire; init builds State child-side; State never crosses.
(:wat::test::deftest :wat-tests::service::seeded-counter-on-process
  
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::seeded-counter/start :locus (:wat::spawn::process) :record (:wat-tests::seeded-counter::Record :count 42))
       c (:wat::core::match (:wat::kernel::connect (:wat-tests::seeded-counter::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       r (:wat-tests::SeededCounter/get c (:wat-tests::SeededCounter::GetRequest))]
      (:wat::core::match r ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv  
        ((:wat-tests::SeededCounter::GetResponse::Ok value) value)
        ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
        ((:wat-tests::SeededCounter::GetResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "seeded-counter-get: unexpected RequestTooLarge"
            :wat::core::None :wat::core::None))
        ((:wat-tests::SeededCounter::GetResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))
    42))
