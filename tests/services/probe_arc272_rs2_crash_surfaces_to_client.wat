;; A service with one op whose handler CRASHES (assertion-failed! raises inside the serve loop).
;; arc 291 4b-ii: State is now a defstruct; :durable mints ::Record; start takes ::Record.
(:wat::service::defservice :my::svc
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :ops
  [(:Boom [s <- :State]
          -> [ok <- :wat::core::bool]
     (:wat::kernel::assertion-failed!
       "boom — the handler crashed on purpose"
       (:wat::core::Some "boom")
       (:wat::core::Some "ok")))])

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [h (:my::svc/start :locus (:wat::spawn::thread) :record (:my::svc::Record 0))
     c (:wat::kernel::connect' (:my::svc::Handle/addr h))
     _ (:my::svc/boom c (:my::svc/boom-request))]
    true))
