;; probe-what-defservice-emits.wat — pure introspection, nothing executed.
;;
;; A service arm pays ~30us for a large unused match arm; the identical structure
;; in a defn pays 0 (probe-does-an-unused-arm-cost-in-a-service.wat vs
;; probe-does-an-unused-arm-cost.wat). defservice is a PURE-WAT defmacro
;; (wat/service.wat:212), so whatever explains that is in what it EMITS.
;;
;; Dump the expansion and read where the impl body lands.

(:wat::core::defsurface :em::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :em::Echo::PingRequest [])
   (:wat::core::defenum :em::Echo::PingResponse :wat::enum::Pure
     :Pong            []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :em::Echo  req <- :em::Echo::PingRequest] -> :em::Echo::PingResponse
     :max-request-bytes 524288)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::write-forms
    (:wat::core::macroexpand (:wat::core::quote
      (:wat::service::defservice :em::echo
        :satisfies :em::Echo
        :durable   [n <- :wat::core::i64]
        :ephemeral []
        :init (:wat::core::fn [record <- :em::echo::Record] -> :em::echo::State
                (:em::echo::State :durable record))
        :impls
        [(ping [s ctx req] MARKER-THE-IMPL-BODY-GOES-HERE)]))))))
