;; dumps the macroexpansion of the do-wrapped (defsurface+defservice) macro call, and the
;; bare (defsurface-only) macro call, side by side, via macroexpand + ast->source.

;; arc 278 #74 — `<Op>Response` is LAW: `echo` on both surfaces below was a bare-primitive
;; heretic (`-> :wat::core::i64`, no Response enum at all). "the heretics self identify by
;; their tongue" (builder ruling, 2026-08-05) — minted a conforming `EchoResponse` on each; the
;; probe's own subject (macroexpansion of a bare-defsurface vs. a do-wrapped defsurface+
;; defservice) is unaffected by the response type's shape. NOTE (arc 278 #74 architecture): the
;; law is enforced at `defsurface` REGISTRATION (`synthesize_surface_protocol`), which these two
;; `:probe::Bare2`/`:probe::Wrapped` declarations never reach — they exist only as quasiquoted
;; DATA inside a `defmacro` body, realized solely via `:wat::core::macroexpand` at RUNTIME
;; (`:user::main`, below), never as a real top-level macro invocation `expand_all` expands at
;; compile time. So the armed check cannot see them; minting `EchoResponse` here is table-driven
;; correctness (matching the surrounding convention), not something the check independently
;; confirms for this file.
(:wat::core::defmacro :probe::just-surface2
  [def-form <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::defsurface :probe::Bare2 :nature :wat::kernel::Peer
     :messages
     [~def-form
      (:wat::core::defrecord :probe::Bare2::EchoRequest [c <- :wat::core::i64])
      (:wat::core::defenum :probe::Bare2::EchoResponse :wat::enum::Pure
        :Ok               [c <- :wat::core::i64]
        :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
        :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
     :features
     [(echo [self <- :probe::Bare2 req <- :probe::Bare2::EchoRequest] -> :probe::Bare2::EchoResponse :max-request-bytes 524288)]))

(:wat::core::defmacro :probe::wrapped-surface
  [def-form <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::do
     (:wat::core::defsurface :probe::Wrapped :nature :wat::kernel::Peer
       :messages
       [~def-form
        (:wat::core::defrecord :probe::Wrapped::EchoRequest [c <- :wat::core::i64])
        (:wat::core::defenum :probe::Wrapped::EchoResponse :wat::enum::Pure
          :Ok               [c <- :wat::core::i64]
          :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
          :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
       :features
       [(echo [self <- :probe::Wrapped req <- :probe::Wrapped::EchoRequest] -> :probe::Wrapped::EchoResponse :max-request-bytes 524288)])
     (:wat::service::defservice :probe::wrappedsvc
       :satisfies :probe::Wrapped
       :durable []
       :impls
       [(echo [s req] (:wat::service::Outcome::Continue s (:wat::core::Some (:probe::Wrapped::Reply::Echo (:probe::Wrapped::EchoResponse::Ok (:probe::Wrapped::EchoRequest/c req)))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Wrapped::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::wrappedsvc::Op])])))])))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [bare-form    (:wat::core::quote (:probe::just-surface2 (:wat::core::defrecord :probe::M1 [x <- :wat::core::i64])))
     wrapped-form (:wat::core::quote (:probe::wrapped-surface (:wat::core::defrecord :probe::M2 [x <- :wat::core::i64])))
     bare-exp     (:wat::core::macroexpand bare-form)
     wrapped-exp  (:wat::core::macroexpand wrapped-form)]
    (:wat::core::do
      (:wat::kernel::println "==== BARE (defsurface-only) EXPANSION ====")
      (:wat::kernel::println (:wat::core::ast->source bare-exp))
      (:wat::kernel::println "==== WRAPPED (do [defsurface defservice]) EXPANSION ====")
      (:wat::kernel::println (:wat::core::ast->source wrapped-exp)))))
