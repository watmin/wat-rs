;; Can a concrete (Address' :- [S R]) be erased to bare Address' via ann-form, stored, and
;; sent as a bare-D PoolMsg::Setup? Test the TYPE questions only (no child).

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                      :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo  :durable [] :ephemeral []
  :impls [(echo [s ctx req]
            (:wat::service::Outcome::Reply s
              (:probe::Echo::EchoResponse::Ok (:probe::Echo::EchoRequest/msg req))))])

;; bare-D PoolMsg (the parent-side shape)
(:wat::core::defenum :probe::PoolMsg :- [I] :wat::enum::Pure
  :Setup [addr <- :wat::kernel::Address]
  :Work  [s    <- :I])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh  (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     ea  (:probe::echo::Handle/addr eh)                       ;; concrete (Address' :- [Op Reply])
     eab (:wat::core::ann-form ea :wat::kernel::Address)      ;; erase -> bare Address'
     v   (:wat::core::Vector :- [:wat::kernel::Address] eab)       ;; store bare in (Vector :- [Address'])
     msg (:probe::PoolMsg::Setup (:wat::core::first v))]       ;; bare-D Setup constructor
    (:wat::kernel::println "erase-ok")))
