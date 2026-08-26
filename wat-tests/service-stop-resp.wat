;; wat-tests/service-stop-resp.wat — arc 291 strike-3b RED probe: stop → resp decouple.
;;
;; THE CONTRACT, proven at the surface: `stop`'s RETURN is DECOUPLED from the live State. A `:stop`
;; callback projects the final State → a serializable `resp` of the AUTHOR'S type — here `:i64` (the count),
;; NOT the `::Record`. `(<svc>/stop h)` returns that i64 directly. This is the out-locus mirror of
;; `:init` (which builds State from a Record in-locus); `:stop` renders State to resp out-locus.
;;
;; ONE defservice, two deftests differing in exactly one token (the locus). Modeled on
;; service-admin-facet.wat (owner-only stop via the Handle) + the shipped `:init`/`:stop` callbacks.
;;
;; arc 291 4b-ii: State is now a defstruct; :durable [count] mints ::Record; ::State holds it.
;; :init defaults (pure-data, ephemeral empty). start takes ::Record(0).
;; Op body reads through State/durable. State building uses State/new (Record c).
;; :stop projection now reads through State/durable: (Record/count (State/durable s)).

;; ── the surface (the counter protocol, lifted) ───────────────────────────────
;; arc 278 S4c: the surface OWNS its protocol messages (:messages) so a :satisfies
;; service ships them across a process fork.
(:wat::core::defsurface :wat-tests::RespCounter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat-tests::RespCounter::IncrementRequest  [n <- :wat::core::i64])
   (:wat::core::defenum :wat-tests::RespCounter::IncrementResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(increment [self <- :wat-tests::RespCounter  req <- :wat-tests::RespCounter::IncrementRequest] -> :wat-tests::RespCounter::IncrementResponse :max-request-bytes 524288)])

;; ── the service: a counter; :stop projects State → i64 (the count) ──
(:wat::service::defservice :wat-tests::resp-counter
  :satisfies :wat-tests::RespCounter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(increment [s ctx req]
     (:wat::core::let [c (:wat::i64::+
                           (:wat-tests::resp-counter::Record/count (:wat-tests::resp-counter::State/durable s))
                           (:wat-tests::RespCounter::IncrementRequest/n req))]
       (:wat::service::Outcome::Reply
         (:wat-tests::resp-counter::State :durable (:wat-tests::resp-counter::Record :count c))
         (:wat-tests::RespCounter::IncrementResponse::Ok c))))  ]
  ;; :stop — the projection: final State → its count (an i64). The stop RETURN is this i64,
  ;; decoupled from the ::Record. Read count through State/durable.
  :stop (:wat::core::fn [s <- :wat-tests::resp-counter::State] -> :wat::core::i64
          (:wat-tests::resp-counter::Record/count (:wat-tests::resp-counter::State/durable s))))

;; ── thread tier ──────────────────────────────────────────────────────────────
;; Increment to 7; the Handle-holder stops; stop returns the PROJECTED i64 (7), not a Record.
(:wat::test::deftest :wat-tests::service::stop-resp-on-thread
  
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::resp-counter/start :locus (:wat::spawn::thread) :record (:wat-tests::resp-counter::Record :count 0))
       c (:wat::core::match (:wat::kernel::connect (:wat-tests::resp-counter::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       _ (:wat::core::match (:wat-tests::RespCounter/increment c (:wat-tests::RespCounter::IncrementRequest :n 7))
           ((:wat::kernel::RecvOutcome::Message _resp) nil)
           ((:wat::kernel::RecvOutcome::Lost _c) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message _c) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
       final (:wat-tests::resp-counter/stop h)]
      final)
    7))

;; ── process tier — IDENTICAL except the locus token ──────────────────────────
(:wat::test::deftest :wat-tests::service::stop-resp-on-process
  
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::resp-counter/start :locus (:wat::spawn::process) :record (:wat-tests::resp-counter::Record :count 0))
       c (:wat::core::match (:wat::kernel::connect (:wat-tests::resp-counter::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       _ (:wat::core::match (:wat-tests::RespCounter/increment c (:wat-tests::RespCounter::IncrementRequest :n 7))
           ((:wat::kernel::RecvOutcome::Message _resp) nil)
           ((:wat::kernel::RecvOutcome::Lost _c) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message _c) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
       final (:wat-tests::resp-counter/stop h)]
      final)
    7))
