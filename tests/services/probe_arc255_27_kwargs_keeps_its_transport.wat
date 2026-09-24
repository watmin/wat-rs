;; tests/services/probe_arc255_27_kwargs_keeps_its_transport.wat — stone 255.27 (C-b5).
;;
;; The ACCEPTED twin of `probe_arc255_27_kwargs_keeps_its_transport.wat.bad`: a thread handle
;; passed through the kwargs check keeps its transport, so its coord is a Shared address.
;; Driven by `probe_arc255_27_kwargs_keeps_its_transport.rs`.
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

(:wat::core::defn :probe::work
  [item <- :wat::core::String
   & [echo <- (:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])]]
  -> :wat::core::String
  item)

(:wat::core::defn :probe::kwargs-thread-handle-claimed-shared []
  -> (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply :wat::kernel::Transport.Shared])
  (:wat::core::let
    [h    (:probe::echo/start :locus (:wat::spawn::thread) :record (:probe::echo::Record))
     pair (:probe::work::kwargs-check :echo h)]
    (:wat::capability::TypedCapability/coord (:probe::work::GrantHandles/echo (:wat::core::second pair)))))
