;; tests/services/probe_arc255_21_coord_names_its_transport.wat — stone 255.21 (C-b1b).
;;
;; The ACCEPTED twins of `probe_arc255_21_coord_claims_either_transport.wat.bad`: a thread handle's
;; `coord` is a Shared address and a process handle's a Wire one, through both `Dialable` and
;; `TypedCapability`. Driven by `probe_arc255_21_capability_names_its_transport.rs`.
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
     :Ok              [reply <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo  :durable []  :ephemeral []
  :impls [(echo [s ctx req]
            (:wat::service::Outcome.Reply {:state s
              :reply (:probe::Echo::EchoResponse.Ok {:reply (:probe::Echo::EchoRequest/msg req)})}))])

(:wat::core::defn :probe::thread-coord-is-shared []
  -> (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply :wat::kernel::Shared])
  (:wat::core::let
    [h (:probe::echo/start :locus (:wat::spawn::thread) :record (:probe::echo::Record))]
    (:wat::capability::Dialable/coord h)))

(:wat::core::defn :probe::process-coord-is-wire []
  -> (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply :wat::kernel::Wire])
  (:wat::core::let
    [h (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))]
    (:wat::capability::Dialable/coord h)))

(:wat::core::defn :probe::thread-typedcap-coord-is-shared []
  -> (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply :wat::kernel::Shared])
  (:wat::core::let
    [h (:probe::echo/start :locus (:wat::spawn::thread) :record (:probe::echo::Record))]
    (:wat::capability::TypedCapability/coord h)))

(:wat::core::defn :probe::process-typedcap-coord-is-wire []
  -> (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply :wat::kernel::Wire])
  (:wat::core::let
    [h (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))]
    (:wat::capability::TypedCapability/coord h)))
