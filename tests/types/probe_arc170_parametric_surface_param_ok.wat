;; Arc 170 C2 — complete parametric surfaces (Gaps 1+2), POSITIVE control.
;; The abstract-`(Dialable :- [S R])` path: a fn whose param is the parametric SURFACE itself
;; (`(Dialable :- [probe::Echo::Op probe::Echo::Reply])`), whose body calls `Dialable/coord` on it.
;;  - Gap 2: `(Dialable/coord d)` on an abstract `(Dialable :- [Echo::Op Echo::Reply])` receiver must
;;    resolve to `(Address' :- [Echo::Op Echo::Reply])` (the surface's `T` instantiated from the
;;    receiver's args), NOT the raw `(Address' :- [S R])` → the declared return type-checks.
;;  - Gap 1: a raw `echo'::Handle` (which satisfies `(Dialable :- [Echo::Op Echo::Reply])` via the
;;    auto-emitted full-args extend-type edge) must be assignable to the parametric `Dialable` param.
;; No hand-written `defsurface Dialable`/`extend-type` — the surface is baked (wat/capability.wat)
;; and auto-emitted per service (wat/service.wat).

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
            (:wat::service::Outcome::Reply s
              (:probe::Echo::EchoResponse::Ok (:probe::Echo::EchoRequest/msg req))))])

;; abstract parametric-surface param + coord on it (Gap 2 return + Gap 1 accepting the handle)
(:wat::core::defn :probe::takes-dialable
  [d <- (:wat::capability::Dialable :- [:probe::Echo::Op :probe::Echo::Reply])]
  -> (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply])
  (:wat::capability::Dialable/coord d))

;; `:probe::run` (a non-main defn — no `:user::main`, per the arc-170 `[] -> :nil` / UselessMain
;; wall) dials nothing; it exists so the checker sees a raw `echo'::Handle` flow into the
;; `(Dialable :- […])` param (Gap 1). Returns the coord'd Address'.
(:wat::core::defn :probe::run [] -> (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply])
  (:wat::core::let
    [eh (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))]
    (:probe::takes-dialable eh)))
