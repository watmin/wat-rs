;; isolates: does a macro whose SOLE return value IS a bare defsurface (no `do` wrapper, no
;; paired defservice) get its :messages accessors minted correctly? Tests whether
;; `is_defsurface_form(&expanded)` (src/macros/expand.rs:53) fires when the defsurface is the
;; DIRECT top-level expansion of the macro call (not nested inside a `do`).

(:wat::core::defmacro :probe::just-surface
  [def-form <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::defsurface :probe::Bare :nature :wat::kernel::Peer
     :messages
     [~def-form
      (:wat::core::defrecord :probe::Bare::EchoRequest [c <- :wat::core::i64])
      ;; arc 278 #74 — `<Op>Response` is LAW: `echo` was a bare-primitive heretic (`->
      ;; :wat::core::i64`, no Response enum at all). "the heretics self identify by their
      ;; tongue" (builder ruling, 2026-08-05) — minted a conforming `EchoResponse` in the
      ;; mandated shape; the probe's own subject (does a macro whose sole return value IS a
      ;; bare defsurface mint its :messages accessors correctly?) is unaffected by the
      ;; response type's shape.
      (:wat::core::defenum :probe::Bare::EchoResponse :wat::enum::Pure
        :Ok               [c <- :wat::core::i64]
        :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
        :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
     :features
     [(echo [self <- :probe::Bare req <- :probe::Bare::EchoRequest] -> :probe::Bare::EchoResponse :max-request-bytes 524288)]))

(:probe::just-surface (:wat::core::defrecord :probe::Bare::Marker [x <- :wat::core::i64]))

;; accessor check WITHOUT any service — just a plain fn calling the derived accessor.
(:wat::core::defn :user::check [] -> :wat::core::i64
  (:probe::Bare::EchoRequest/c (:probe::Bare::EchoRequest :c 5)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::str "check=" (:user::check))))
