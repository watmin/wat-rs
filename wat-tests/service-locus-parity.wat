;; wat-tests/service-locus-parity.wat — arc 272 6b-iii: the locus-parity proof at the WAT level.
;;
;; ONE defservice (the counter); two deftests that differ in EXACTLY ONE token — the locus
;; (:wat::spawn::thread) vs (:wat::spawn::process). The generated client face (start, connect',
;; increment/get, the request constructors, the Handle, the Response accessor) is byte-identical.
;; This is the parity contract written as a test: swap the locus, the same service runs.
;;
;; The Rust-level proof is `tests/probe_arc272_6b_defservice_on_process.rs` (a forking [[test]] binary);
;; this dogfoods the same surface in wat. defservice names NO transport — the (process) literal the
;; service rides lives only in the ProcessOpts `launch` arm (design C).
;;
;; arc 291 4b-ii: State is now a defstruct; :durable mints ::Record (the soul); ::State holds it.
;; start takes a ::Record (not a pre-built ::State). Accessors read through State/durable.

;; ── the surface (the counter protocol, lifted) ───────────────────────────────
;; arc 278 S4c: the surface OWNS its protocol messages (:messages) so a :satisfies
;; service ships them across a process fork.
(:wat::core::defsurface :wat-tests::Counter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat-tests::Counter::GetRequest       [])
   (:wat::core::defenum :wat-tests::Counter::GetResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :wat-tests::Counter::IncrementRequest  [n <- :wat::core::i64])
   (:wat::core::defenum :wat-tests::Counter::IncrementResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(get       [self <- :wat-tests::Counter  req <- :wat-tests::Counter::GetRequest]       -> :wat-tests::Counter::GetResponse :max-request-bytes 524288)
   (increment [self <- :wat-tests::Counter  req <- :wat-tests::Counter::IncrementRequest] -> :wat-tests::Counter::IncrementResponse :max-request-bytes 524288)])

;; ── the service, defined once at top-level (shared by both deftests) ──────────
(:wat::service::defservice :wat-tests::counter
  :satisfies :wat-tests::Counter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(get [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:wat-tests::Counter::Reply::Get (:wat-tests::Counter::GetResponse::Ok
         (:wat-tests::counter::Record/count (:wat-tests::counter::State/durable s))))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat-tests::Counter::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:wat-tests::counter::Op])])))
   (increment [s ctx req]
     (:wat::core::let [c (:wat::i64::+
                           (:wat-tests::counter::Record/count (:wat-tests::counter::State/durable s))
                           (:wat-tests::Counter::IncrementRequest/n req))]
       (:wat::service::Outcome::Continue
         (:wat-tests::counter::State :durable (:wat-tests::counter::Record :count c))
         (:wat::core::Some (:wat-tests::Counter::Reply::Increment (:wat-tests::Counter::IncrementResponse::Ok c))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat-tests::Counter::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:wat-tests::counter::Op])]))))])

;; ── thread tier ──────────────────────────────────────────────────────────────
(:wat::test::deftest :wat-tests::service::counter-on-thread
  
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::counter/start :locus (:wat::spawn::thread) :record (:wat-tests::counter::Record :count 0))
       c (:wat::core::match (:wat::kernel::connect (:wat-tests::counter::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       _ (:wat::core::match (:wat-tests::Counter/increment c (:wat-tests::Counter::IncrementRequest :n 5))
           ((:wat::kernel::RecvOutcome::Message _resp) nil)
           ((:wat::kernel::RecvOutcome::Lost _c) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message _c) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
       r (:wat-tests::Counter/get c (:wat-tests::Counter::GetRequest))]
      (:wat::core::match r ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv  
        ((:wat-tests::Counter::GetResponse::Ok value) value)
        ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
        ((:wat-tests::Counter::GetResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "counter-get: unexpected RequestTooLarge"
            :wat::core::None :wat::core::None))
        ((:wat-tests::Counter::GetResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))
    5))

;; ── process tier — IDENTICAL except the locus token ──────────────────────────
(:wat::test::deftest :wat-tests::service::counter-on-process
  
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::counter/start :locus (:wat::spawn::process) :record (:wat-tests::counter::Record :count 0))
       c (:wat::core::match (:wat::kernel::connect (:wat-tests::counter::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       _ (:wat::core::match (:wat-tests::Counter/increment c (:wat-tests::Counter::IncrementRequest :n 5))
           ((:wat::kernel::RecvOutcome::Message _resp) nil)
           ((:wat::kernel::RecvOutcome::Lost _c) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message _c) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
       r (:wat-tests::Counter/get c (:wat-tests::Counter::GetRequest))]
      (:wat::core::match r ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv  
        ((:wat-tests::Counter::GetResponse::Ok value) value)
        ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
        ((:wat-tests::Counter::GetResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "counter-get: unexpected RequestTooLarge"
            :wat::core::None :wat::core::None))
        ((:wat-tests::Counter::GetResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))
    5))
