;; Arc 278 S4c migration: :ops RETIRED — the service wears a surface (:satisfies + :impls).
;; SUBJECT UNCHANGED: :durable-parent :wat::holon::Record parents the ::Record (the durable
;; soul), NOT the State struct. So (record? (State/durable s)) is TRUE (holon record);
;; (record? s) is FALSE (s is a defstruct, not a record).
(:wat::core::defsurface :my::HCounter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :my::HCounter::IsHolonRecordRequest  [])
   (:wat::core::defenum :my::HCounter::IsHolonRecordResponse :wat::enum::Pure
     :Ok              [yes <- :wat::core::bool]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(is-holon-record [self <- :my::HCounter  req <- :my::HCounter::IsHolonRecordRequest] -> :my::HCounter::IsHolonRecordResponse :max-request-bytes 524288)])

(:wat::service::defservice :my::hcounter
  :satisfies :my::HCounter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(is-holon-record [s ctx req]
     (:wat::service::Outcome::Continue s (:wat::core::Some (:my::HCounter::Reply::IsHolonRecord (:my::HCounter::IsHolonRecordResponse::Ok
                                        (:wat::core::record? (:my::hcounter::State/durable s))))) (:wat::core::Vector :- [(:wat::service::Directed :- [:my::HCounter::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:my::hcounter::Op])])))]
  :durable-parent :wat::holon::Record)

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [h (:my::hcounter/start :locus (:wat::spawn::thread) :record (:my::hcounter::Record :count 0))
     c (:wat::core::match (:wat::kernel::connect (:my::hcounter::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     r (:my::hcounter/is-holon-record c (:my::HCounter::IsHolonRecordRequest))]
    (:wat::core::match r ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:my::HCounter::IsHolonRecordResponse::Ok yes) yes)
      ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
      ((:my::HCounter::IsHolonRecordResponse::RequestTooLarge bytes cap)
        (:wat::kernel::assertion-failed! "compute: unexpected RequestTooLarge"
          :wat::core::None :wat::core::None))
      ((:my::HCounter::IsHolonRecordResponse::RequestMalformed mpath mexpected mgot)
        (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
