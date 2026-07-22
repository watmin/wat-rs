;; dumps the macroexpansion of the do-wrapped (defsurface+defservice) macro call, and the
;; bare (defsurface-only) macro call, side by side, via macroexpand + ast->source.

(:wat::core::defmacro :probe::just-surface2
  [def-form <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::defsurface :probe::Bare2 :nature :wat::kernel::Peer'
     :messages
     [~def-form
      (:wat::core::defrecord :probe::Bare2::Req [c <- :wat::core::i64])]
     :features
     [(echo [self <- :probe::Bare2 req <- :probe::Bare2::Req] -> :wat::core::i64 :max-request-bytes 524288)]))

(:wat::core::defmacro :probe::wrapped-surface
  [def-form <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::do
     (:wat::core::defsurface :probe::Wrapped :nature :wat::kernel::Peer'
       :messages
       [~def-form
        (:wat::core::defrecord :probe::Wrapped::Req [c <- :wat::core::i64])]
       :features
       [(echo [self <- :probe::Wrapped req <- :probe::Wrapped::Req] -> :wat::core::i64 :max-request-bytes 524288)])
     (:wat::service::defservice :probe::wrappedsvc'
       :satisfies :probe::Wrapped
       :durable []
       :impls
       [(echo [s req] (:wat::service::Outcome::Reply s (:probe::Wrapped::Req/c req)))])))

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
