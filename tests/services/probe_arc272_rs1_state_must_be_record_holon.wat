;; Holon — :durable-parent :wat::holon::Record now parents the ::Record (the durable soul),
;; NOT the State struct. So (record? (State/durable s)) is TRUE (holon record);
;; (record? s) is FALSE (s is a defstruct, not a record).
(:wat::service::defservice :my::hcounter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :ops
  [(:IsHolonRecord [s <- :State]
                   -> [yes <- :wat::core::bool]
     (:wat::service::Outcome::Reply s (:my::hcounter::IsHolonRecordResponse
                                        (:wat::core::record? (:my::hcounter::State/durable s)))))]
  :durable-parent :wat::holon::Record)

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [h (:my::hcounter/start :locus (:wat::spawn::thread) :record (:my::hcounter::Record 0))
     c (:wat::kernel::connect' (:my::hcounter::Handle/addr h))
     r (:my::hcounter/is-holon-record c (:my::hcounter/is-holon-record-request))]
    (:my::hcounter::IsHolonRecordResponse/yes r)))
