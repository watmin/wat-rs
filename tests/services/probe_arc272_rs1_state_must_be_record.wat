;; Arc 278 S4c migration: :ops RETIRED — the service wears a surface (:satisfies + :impls).
;; SUBJECT UNCHANGED (orthogonal to the surface migration): :durable [fields] mints ::Record
;; (the EDN soul); ::State is a defstruct. Handler reads through State/durable, builds next
;; State via State/Record, stop returns ::Record.
(:wat::core::defsurface :my::Counter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :my::Counter::IncrementRequest  [n     <- :wat::core::i64])
   (:wat::core::defenum :my::Counter::IncrementResponse :wat::enum::Pure
     :Ok              [count <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(increment [self <- :my::Counter  req <- :my::Counter::IncrementRequest] -> :my::Counter::IncrementResponse :max-request-bytes 524288)])

(:wat::service::defservice :my::counter
  :satisfies :my::Counter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(increment [s ctx req]
     (:wat::core::let [c (:wat::i64::+ (:my::counter::Record/count (:my::counter::State/durable s)) (:my::Counter::IncrementRequest/n req))]
       (:wat::service::Outcome::Reply (:my::counter::State :durable (:my::counter::Record :count c)) (:my::Counter::IncrementResponse::Ok c))))])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h     (:my::counter/start :locus (:wat::spawn::thread) :record (:my::counter::Record :count 0))
     c     (:wat::core::match (:wat::kernel::connect (:my::counter::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _     (:wat::core::match (:my::counter/increment c (:my::Counter::IncrementRequest :n 5))
             ((:wat::kernel::RecvOutcome::Message _resp) nil)
             ((:wat::kernel::RecvOutcome::Lost _c) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message _c) :wat::core::None :wat::core::None))
             (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None))
             (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
     ;; arc 291 3a-ii-β: stop is owner-only — takes the Handle (h), not the client peer (c).
     ;; arc 291 4b-ii: stop returns ::Record (durable soul); read count via Record/count.
     final (:my::counter/stop h)]
    (:my::counter::Record/count final)))
