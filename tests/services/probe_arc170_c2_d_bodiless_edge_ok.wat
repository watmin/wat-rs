;; Arc 170 C2 candidate D — the BODILESS extend-type mechanism, promoted from the disconfirming
;; probe (scratchpad/probe-v-bodiless.wat) that proved it this session.
;;
;; A hand-defined LOCAL `:probe::TypedCapability<S,R>` surface (coord/grant/revoke) — distinct
;; from (but the exact shape that) the real `:wat::capability::TypedCapability<S,R>` (baked
;; wat/capability.wat, auto-emitted per-service wat/service.wat) — with a BODILESS extend-type:
;; `(extend-type :probe::echo'::Handle :probe::TypedCapability<probe::Echo::Op,probe::Echo::Reply>)`
;; — no method bodies. Registers the satisfaction EDGE only; `coord`/`grant` are served, at
;; runtime, off the flat `Handle/coord` key the handle would need ANYWAY (a real service Handle
;; gets that key for free from the real auto-emitted Dialable/Capability; this probe fakes it by
;; hand-defining the surface — see the swap sibling for the real auto-emitted mechanism's own
;; coverage via `probe_arc170_w2a_kwargs_check_mint*`). Isolates: does a bodiless extend-type
;; register assignability WITHOUT re-declaring methods (the DuplicateDefine the first C2-D
;; attempt walled on)? Must freeze clean.

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg <- :wat::core::String])
             (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
               :Ok              [reply <- :wat::core::String]
               :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
               :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::echo :satisfies :probe::Echo :durable [] :ephemeral []
  :impls [(echo [s ctx req] (:wat::service::Outcome::Reply s (:probe::Echo::EchoResponse::Ok (:probe::Echo::EchoRequest/msg req))))])
(:wat::core::defsurface :probe::TypedCapability :- [S R] :nature :wat::core::Struct
  :features
  [(coord  [self <- (:probe::TypedCapability :- [S R])] -> (:wat::kernel::Address :- [S R]))
   (grant  [self <- (:probe::TypedCapability :- [S R])  pids <- (:wat::core::Vector :wat::core::i64)] -> :wat::core::nil)
   (revoke [self <- (:probe::TypedCapability :- [S R])  pids <- (:wat::core::Vector :wat::core::i64)] -> :wat::core::nil)])

;; *** BODILESS extend-type — edge only, no method bodies ***
(:wat::core::extend-type :probe::echo::Handle (:probe::TypedCapability :- [:probe::Echo::Op :probe::Echo::Reply]))

;; hold at the abstract combined type; call BOTH grant + typed-coord through it.
(:wat::core::defn :probe::use-both
  [h <- (:probe::TypedCapability :- [:probe::Echo::Op :probe::Echo::Reply])]
  -> (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply])
  (:wat::core::let
    [_ (:probe::TypedCapability/grant h (:wat::core::Vector :wat::core::i64 42))]
    (:probe::TypedCapability/coord h)))

;; a raw echo'::Handle must be assignable to the combined-surface param.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     _  (:probe::use-both eh)]
    nil))
