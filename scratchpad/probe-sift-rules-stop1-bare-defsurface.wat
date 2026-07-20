;; isolates: does a macro whose SOLE return value IS a bare defsurface (no `do` wrapper, no
;; paired defservice) get its :messages accessors minted correctly? Tests whether
;; `is_defsurface_form(&expanded)` (src/macros/expand.rs:53) fires when the defsurface is the
;; DIRECT top-level expansion of the macro call (not nested inside a `do`).

(:wat::core::defmacro :probe::just-surface
  [def-form <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::defsurface :probe::Bare :nature :wat::kernel::Peer'
     :messages
     [~def-form
      (:wat::core::defrecord :probe::Bare::Req [c <- :wat::core::i64])]
     :features
     [(echo [self <- :probe::Bare req <- :probe::Bare::Req] -> :wat::core::i64)]))

(:probe::just-surface (:wat::core::defrecord :probe::Bare::Marker [x <- :wat::core::i64]))

;; accessor check WITHOUT any service — just a plain fn calling the derived accessor.
(:wat::core::defn :user::check [] -> :wat::core::i64
  (:probe::Bare::Req/c (:probe::Bare::Req :c 5)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::str "check=" (:user::check))))
