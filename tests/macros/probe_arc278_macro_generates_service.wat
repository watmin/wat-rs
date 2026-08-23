;; Arc 278 — Option A RED gate: a macro that GENERATES a service.
;;
;; The Rules-form UX needs a macro that emits BOTH a defsurface (its :messages
;; spliced from user defs) AND a :satisfies defservice. A macro returns ONE
;; form, so it wraps the pair in a `(do …)`. At HEAD, expand.rs routes a
;; defsurface through `hoist_surface_messages` (which lifts its :messages
;; recordtype/defenum decls to top level so their field accessors + ::Variant
;; ctors mint) ONLY when the defsurface is the DIRECT top-level expansion
;; (`is_defsurface_form`, expand.rs:53). A defsurface nested in the macro's `do`
;; falls to the else branch and is spliced up RAW — its :messages are never
;; hoisted → `:probe::Echo::EchoRequest/c` is UnresolvedReference at freeze →
;; StartupError.
;;
;; GREEN when Option A makes the do/let-splice re-enter the per-form dispatch,
;; so a do-nested defsurface gets the SAME hoist_surface_messages treatment a
;; direct one gets.
;;
;; NB: per-op message types follow defservice's S1/gRPC naming convention
;; (service.wat:894 — `<Op>Request`/`<Op>Response`, op `echo` → `Echo…`), so the
;; ONLY thing red at HEAD is the hoist gap, nothing incidental.

(:wat::core::defmacro :probe::echo-defsvc
  [def-form <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::do
     (:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
       :messages
       [~def-form
        (:wat::core::defrecord :probe::Echo::EchoRequest [c <- :wat::core::i64])
        (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
          :Ok              [n <- :wat::core::i64]
          :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
          :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
       :features
       [(echo [self <- :probe::Echo req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
     (:wat::service::defservice :probe::echosvc
       :satisfies :probe::Echo
       :durable []
       :impls
       [(echo [s ctx req]
          (:wat::core::let
            [c (:probe::Echo::EchoRequest/c req)]
            (:wat::service::Outcome::Reply s (:probe::Echo::EchoResponse::Ok c))))])))

;; Invoke the macro: it emits the do-wrapped surface+service, splicing in a user
;; def (a Marker record) as the surface's first :messages member.
(:probe::echo-defsvc (:wat::core::defrecord :probe::Marker [x <- :wat::core::i64]))

;; The RED assertion: the surface's generated FIELD ACCESSOR must mint. At HEAD
;; `:probe::Echo::EchoRequest/c` is UnresolvedReference at freeze (StartupError);
;; GREEN it resolves + returns 42. (Startup succeeding at all also proves the
;; defservice's generated client stub type-checks against the hoisted types.)
(:wat::core::defn :user::check [] -> :wat::core::i64
  (:wat::core::let
    [req (:probe::Echo::EchoRequest :c 42)]
    (:probe::Echo::EchoRequest/c req)))
