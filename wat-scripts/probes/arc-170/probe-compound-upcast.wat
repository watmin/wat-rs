;; probe-compound-upcast.wat — POSITIVE gate for the generalized expected-type-directed
;; up-cast rule (this strike): Tuple / Map / Set constructors/literals up-cast their
;; components against a known expected type, same as fbc60b94 did for `[...]` Vector.
;;
;; eh (echo'::Handle) IS-A :wat::capability::Capability via the defservice-auto-emitted
;; extend-type (same subtype pair fbc60b94's vector fix + probe-c1-capability-upcast.wat
;; already used for the scalar case).
;;
;; GATE: `wat --check` on this file must exit 0 (all three forms type-check: Tuple via
;; ann-form, Map via call-arg, Set via call-arg — each up-casts eh: Handle -> Capability
;; at construction). Tuple+Map also RUN to completion (proven separately). Set's runtime
;; execution hits a PRE-EXISTING, unrelated gap — `(HashSet :- [Capability])` panics on an
;; opaque-Handle element ("Value::RustOpaque is not atomizable") even via the OLD verbose
;; `(:wat::core::HashSet :wat::capability::Capability (:wat::capability::as-capability eh))`
;; ctor (verified: same panic pre-fix, nothing to do with this strike's checker change —
;; a runtime hashing limitation, out of this strike's "no runtime.rs change" scope).
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
             (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                                :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo :satisfies :probe::Echo :durable [] :ephemeral []
  :impls [(echo [s ctx req] (:wat::service::Outcome::Continue s
            (:wat::core::Some (:probe::Echo::Reply::Echo (:probe::Echo::EchoResponse::Ok (:wat::string::concat "echo:" (:probe::Echo::EchoRequest/msg req))))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Echo::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::echo::Op])])))])

(:wat::core::defn :probe::as-map [m <- (:wat::core::HashMap :- [:wat::core::keyword :wat::capability::Capability])]
  -> (:wat::core::HashMap :- [:wat::core::keyword :wat::capability::Capability])
  m)

(:wat::core::defn :probe::as-set [s <- (:wat::core::HashSet :- [:wat::capability::Capability])]
  -> (:wat::core::HashSet :- [:wat::capability::Capability])
  s)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     ;; Tuple — ann-form site: (:wat::core::Tuple :echo eh) ascribed to
     ;; :(wat::core::keyword,wat::capability::Capability); eh up-casts Handle -> Capability.
     pr (:wat::core::ann-form (:wat::core::Tuple :echo eh)
          (:wat::core::Tuple :- [:wat::core::keyword :wat::capability::Capability]))
     ;; Map — call-arg site: {:echo eh} against as-map's (HashMap :- [keyword Capability]) param.
     mp (:probe::as-map {:echo eh})
     ;; Set — call-arg site: #{eh} against as-set's (HashSet :- [Capability]) param.
     st (:probe::as-set #{eh})]
    (:wat::kernel::println "compound-upcast: ok")))
