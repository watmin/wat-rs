;; Arc 170 W2a — auto-minted `<fqdn>::kwargs-check` POSITIVE control.
;; :probe::enrich is a kwargs defn (two Peer'<S,R> handle fields) — defn's kwargs branch
;; (wat/core.wat:876) auto-mints :probe::enrich::kwargs-check, a checker fn whose Peer'
;; field types are head-swapped to Address'<S,R> (data-typed fields pass through
;; unchanged). A CORRECTLY-typed kwargs call to the AUTO-MINTED checker (not the work
;; fn itself) must freeze clean. No hand-written defsurface/extend-type for Dialable —
;; it is BAKED (wat/capability.wat) and AUTO-EMITTED per-service (wat/service.wat).

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defrecord :probe::Echo::EchoResponse [reply <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse)])
(:wat::service::defservice :probe::echo'
  :satisfies :probe::Echo  :durable []  :ephemeral []
  :impls [(echo [s req]
            (:wat::service::Outcome::Reply s
              (:probe::Echo::EchoResponse (:probe::Echo::EchoRequest/msg req))))])

(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Kv::GetRequest  [k <- :wat::core::String])
   (:wat::core::defrecord :probe::Kv::GetResponse [v <- :wat::core::String])]
  :features
  [(get [self <- :probe::Kv  req <- :probe::Kv::GetRequest] -> :probe::Kv::GetResponse)])
(:wat::service::defservice :probe::kv'
  :satisfies :probe::Kv  :durable []  :ephemeral []
  :impls [(get [s req]
            (:wat::service::Outcome::Reply s
              (:probe::Kv::GetResponse (:probe::Kv::GetRequest/k req))))])

;; the kwargs work-fn -> AUTO-mints :probe::enrich::kwargs-check
(:wat::core::defn :probe::enrich
  [item <- :wat::core::String
   & [echo <- :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>
      kv   <- :wat::kernel::Peer'<probe::Kv::Op,probe::Kv::Reply>]]
  -> :wat::core::String
  (:probe::Echo::EchoResponse/reply (:probe::Echo/echo echo (:probe::Echo::EchoRequest item))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh  (:probe::echo'/start :locus (:wat::spawn::process) :record (:probe::echo'::Record))
     kvh (:probe::kv'/start   :locus (:wat::spawn::process) :record (:probe::kv'::Record))]
    (:probe::enrich::kwargs-check :echo (:wat::capability::Dialable/coord eh)
                                  :kv   (:wat::capability::Dialable/coord kvh))))
