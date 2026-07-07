;; Arc 278 S4c migration: :ops RETIRED — the service wears a surface (:satisfies + :impls).
;; SUBJECT UNCHANGED: :durable-parent :wat::holon::Record parents the ::Record (the durable
;; soul), NOT the State struct. So (record? (State/durable s)) is TRUE (holon record);
;; (record? s) is FALSE (s is a defstruct, not a record).
(:wat::core::defsurface :my::HCounter :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :my::HCounter::IsHolonRecordRequest  [])
   (:wat::core::defrecord :my::HCounter::IsHolonRecordResponse [yes <- :wat::core::bool])]
  :features
  [(is-holon-record [self <- :my::HCounter  req <- :my::HCounter::IsHolonRecordRequest] -> :my::HCounter::IsHolonRecordResponse)])

(:wat::service::defservice :my::hcounter
  :satisfies :my::HCounter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(is-holon-record [s req]
     (:wat::service::Outcome::Reply s (:my::HCounter::IsHolonRecordResponse
                                        (:wat::core::record? (:my::hcounter::State/durable s)))))]
  :durable-parent :wat::holon::Record)

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [h (:my::hcounter/start :locus (:wat::spawn::thread) :record (:my::hcounter::Record 0))
     c (:wat::kernel::connect' (:my::hcounter::Handle/addr h))
     r (:my::hcounter/is-holon-record c (:my::HCounter::IsHolonRecordRequest))]
    (:my::HCounter::IsHolonRecordResponse/yes r)))
