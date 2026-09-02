;; wat-tests/service-admin-facet.wat — arc 291 strike-3a RED probe: the admin/data facet split.
;;
;; THE CONTRACT, proven at the surface: `stop` is OWNER-ONLY. It moves off the client `Op` enum onto the
;; Handle's admin surface — so its caller argument flips from a CLIENT peer (`connect'`-derived) to the
;; `Handle` itself (held only by the spawner). A client holding only the dial-`Address'` has no `stop`
;; method at all; the Handle-holder calls `(<svc>/stop handle)`.
;;
;; ONE defservice, two deftests differing in exactly one token (the locus). Modeled on
;; service-locus-parity.wat + service-init-parity.wat (uses the shipped `:init`).
;;
;; arc 291 4b-ii: State is now a defstruct; :durable [count] mints ::Record; ::State holds it.
;; :init defaults (pure-data, ephemeral empty). start takes ::Record(0).
;; Op body reads through State/durable. State building uses State/new (Record c).
;; stop defaults to (fn [s] -> ::Record (State/durable s)) → final is a ::Record.
;; Assertion reads Record/count final.

;; ── the surface (the counter protocol, lifted) ───────────────────────────────
;; arc 278 S4c: the surface OWNS its protocol messages (:messages) so a :satisfies
;; service ships them across a process fork.
(:wat::core::defsurface :wat-tests::AdminCounter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat-tests::AdminCounter::IncrementRequest  [n <- :wat::core::i64])
   (:wat::core::defenum :wat-tests::AdminCounter::IncrementResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(increment [self <- :wat-tests::AdminCounter  req <- :wat-tests::AdminCounter::IncrementRequest] -> :wat-tests::AdminCounter::IncrementResponse :max-request-bytes 524288)])

;; ── the service: a counter; Increment is a client (data-plane) op; stop is admin (control-plane) ──
(:wat::service::defservice :wat-tests::admin-counter
  :satisfies :wat-tests::AdminCounter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(increment [s ctx req]
     (:wat::core::let [c (:wat::i64::+
                           (:wat-tests::admin-counter::Record/count (:wat-tests::admin-counter::State/durable s))
                           (:wat-tests::AdminCounter::IncrementRequest/n req))]
       (:wat::service::Outcome::Continue
         (:wat-tests::admin-counter::State :durable (:wat-tests::admin-counter::Record :count c))
         (:wat::core::Some (:wat-tests::AdminCounter::Reply::Increment (:wat-tests::AdminCounter::IncrementResponse::Ok c))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat-tests::AdminCounter::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:wat-tests::admin-counter::Op])]))))])

;; ── thread tier ──────────────────────────────────────────────────────────────
;; A client (dial-Address') does the data op; the Handle-holder issues the admin stop.
;; stop takes the HANDLE (h), not the client peer (c) — owner-only by construction.
;; stop defaults to returning the ::Record — extract count via Record/count.
(:wat::test::deftest :wat-tests::service::admin-stop-on-thread
  
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::admin-counter/start :locus (:wat::spawn::thread) :record (:wat-tests::admin-counter::Record :count 0))
       c (:wat::core::match (:wat::kernel::connect (:wat-tests::admin-counter::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       _ (:wat::core::match (:wat-tests::AdminCounter/increment c (:wat-tests::AdminCounter::IncrementRequest :n 7))
           ((:wat::kernel::RecvOutcome::Message _resp) nil)
           ((:wat::kernel::RecvOutcome::Lost _c) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message _c) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
       final (:wat-tests::admin-counter/stop h)]
      (:wat-tests::admin-counter::Record/count final))
    7))

;; ── process tier — IDENTICAL except the locus token ──────────────────────────
(:wat::test::deftest :wat-tests::service::admin-stop-on-process
  
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::admin-counter/start :locus (:wat::spawn::process) :record (:wat-tests::admin-counter::Record :count 0))
       c (:wat::core::match (:wat::kernel::connect (:wat-tests::admin-counter::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       _ (:wat::core::match (:wat-tests::AdminCounter/increment c (:wat-tests::AdminCounter::IncrementRequest :n 7))
           ((:wat::kernel::RecvOutcome::Message _resp) nil)
           ((:wat::kernel::RecvOutcome::Lost _c) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message _c) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
       final (:wat-tests::admin-counter/stop h)]
      (:wat-tests::admin-counter::Record/count final))
    7))
