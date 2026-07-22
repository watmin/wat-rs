;; Arc 278 S4c migration: :ops RETIRED — the service wears a surface (:satisfies + :impls).
;; SUBJECT UNCHANGED: a service with one op whose handler CRASHES (assertion-failed! raises
;; inside the serve loop). The far-side crash must SURFACE to the client as a raise.
(:wat::core::defsurface :my::Svc :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :my::Svc::BoomRequest  [])
   (:wat::core::defenum :my::Svc::BoomResponse :wat::enum::Pure
     :Ok              [ok <- :wat::core::bool]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(boom [self <- :my::Svc  req <- :my::Svc::BoomRequest] -> :my::Svc::BoomResponse :max-request-bytes 524288)])

(:wat::service::defservice :my::svc
  :satisfies :my::Svc
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(boom [s req]
     (:wat::kernel::assertion-failed!
       "boom — the handler crashed on purpose"
       (:wat::core::Some "boom")
       (:wat::core::Some "ok")))])

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [h (:my::svc/start :locus (:wat::spawn::thread) :record (:my::svc::Record :count 0))
     c (:wat::kernel::connect' (:my::svc::Handle/addr h))
     _ (:my::svc/boom c (:my::Svc::BoomRequest))]
    true))
