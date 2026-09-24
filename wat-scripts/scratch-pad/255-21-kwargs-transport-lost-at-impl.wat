;; wat-scripts/scratch-pad/255-21-kwargs-transport-lost-at-impl.wat — stone 255.21 (C-b1b), STOP.
;;
;; WITNESS of the hole 255.21 could not close inside the kwargs macro. The generated
;; `::kwargs-check` now DECLARES a transport param per service field (`T0` here), and its
;; `::Kwargs` constructor binds it from the handle (measured: `(Kwargs' h)` on a thread handle is
;; `(… ::Kwargs :- [:wat::kernel::Shared])`). But the call then goes through `$impl`, whose param is
;; `(Kwargs :- [T0])`, and `assignable`'s transport-slot arm (`transport_param_instantiates`,
;; check.rs, C-b5 ground) ADMITS `Shared` against the fresh variable without BINDING it. The
;; result `(GrantHandles :- [_])` has a free transport, so BOTH defns below type-check (rc=0).
;; When `:probe::kwargs-thread-handle-claimed-wire` stops type-checking, the hole is closed — move
;; it to a .wat.bad row beside tests/services/probe_arc255_21_coord_claims_either_transport.wat.bad.
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

(:wat::core::defn :probe::kwargs-thread-handle-claimed-wire []
  -> (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply :wat::kernel::Transport.Wire])
  (:wat::core::let
    [h    (:probe::echo/start :locus (:wat::spawn::thread) :record (:probe::echo::Record))
     pair (:probe::work::kwargs-check :echo h)]
    (:wat::capability::TypedCapability/coord (:probe::work::GrantHandles/echo (:wat::core::second pair)))))

(:wat::core::defn :probe::kwargs-thread-handle-claimed-shared []
  -> (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply :wat::kernel::Transport.Shared])
  (:wat::core::let
    [h    (:probe::echo/start :locus (:wat::spawn::thread) :record (:probe::echo::Record))
     pair (:probe::work::kwargs-check :echo h)]
    (:wat::capability::TypedCapability/coord (:probe::work::GrantHandles/echo (:wat::core::second pair)))))
