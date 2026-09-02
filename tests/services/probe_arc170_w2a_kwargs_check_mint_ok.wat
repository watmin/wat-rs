;; Arc 170 W2a/C2-D — auto-minted `<fqdn>::kwargs-check` POSITIVE control.
;; :probe::enrich is a kwargs defn (two (Peer' :- [S R]) handle fields) — defn's kwargs branch
;; (wat/core.wat:876) auto-mints :probe::enrich::kwargs-check, a checker fn whose Peer'
;; field types are head-swapped to (TypedCapability :- [S R]) (data-typed fields pass through
;; unchanged; arc 170 C2 candidate D). A CORRECTLY-typed kwargs call to the AUTO-MINTED
;; checker (not the work fn itself), passing RAW HANDLES (no Dialable/coord upcast — the
;; handle satisfies TypedCapability directly via the bodiless auto-emit), must freeze
;; clean. No hand-written defsurface/extend-type — TypedCapability is BAKED
;; (wat/capability.wat) and AUTO-EMITTED per-service, bodiless (wat/service.wat).

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
            (:wat::service::Outcome::Continue s
              (:wat::core::Some (:probe::Echo::Reply::Echo (:probe::Echo::EchoResponse::Ok (:probe::Echo::EchoRequest/msg req)))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Echo::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::echo::Op])])))])

(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Kv::GetRequest  [k <- :wat::core::String])
   (:wat::core::defenum :probe::Kv::GetResponse :wat::enum::Pure
     :Ok              [v <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(get [self <- :probe::Kv  req <- :probe::Kv::GetRequest] -> :probe::Kv::GetResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::kv
  :satisfies :probe::Kv  :durable []  :ephemeral []
  :impls [(get [s ctx req]
            (:wat::service::Outcome::Continue s
              (:wat::core::Some (:probe::Kv::Reply::Get (:probe::Kv::GetResponse::Ok (:probe::Kv::GetRequest/k req)))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Kv::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::kv::Op])])))])

;; the kwargs work-fn -> AUTO-mints :probe::enrich::kwargs-check
(:wat::core::defn :probe::enrich
  [item <- :wat::core::String
   & [echo <- (:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])
      kv   <- (:wat::kernel::Peer :- [:probe::Kv::Op :probe::Kv::Reply])]]
  -> :wat::core::String
  (:wat::core::match (:probe::Echo/echo echo (:probe::Echo::EchoRequest :msg item)) ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
    ((:probe::Echo::EchoResponse::Ok reply) reply)
    ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
      (:wat::kernel::assertion-failed! "enrich: unexpected RequestTooLarge"
        :wat::core::None :wat::core::None))
    ((:probe::Echo::EchoResponse::RequestMalformed mpath mexpected mgot)
      (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

;; arc 170 C2 D — the checker RETURNS `(::Coords, ::GrantHandles)` (a Tuple: the pure
;; field-ordered Address'+data record, and the impure parent-local typed-handle struct);
;; the checker's service params are `(TypedCapability :- [S R])`, so the call site passes RAW
;; HANDLES (no coord upcast — the bodiless edge admits them directly). `:user::main`
;; discards the pair (`_pair`, a plain let-binding) so the call still exercises the
;; param-type gate while `main` keeps its required `[] -> :nil` contract.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh    (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     kvh   (:probe::kv/start   :locus (:wat::spawn::process) :record (:probe::kv::Record))
     _pair (:probe::enrich::kwargs-check :echo eh :kv kvh)]
    nil))
