;; Arc 278 — RED fixture: a USER `extend-type` impl body that LIES about its type.
;;
;; The surface `:probe::Sink` declares `emit … -> :i64`. The `:probe::Mem` impl body
;; returns a String. This MUST fail `check_program` with a ReturnTypeMismatch once user
;; extend-type impl bodies are swept by `check_function_body` (check.rs:826) — the same
;; sweep that already checks BAKED extend-type impls (registered at freeze step 7.6,
;; before check) but NEVER reaches user impls (registered at step 9, after check).
;;
;; At HEAD this fixture FREEZES CLEAN (the flaw): the wrong-typed satisfier compiles.
;; The gate test's `Ok` arm therefore panics at HEAD; the fix turns it GREEN.
(:wat::core::defsurface :probe::Sink :holder :wat::core::Struct
  :features [(emit [self <- :probe::Sink  x <- :wat::core::i64] -> :wat::core::i64)])

(:wat::core::defstruct :probe::Mem [n <- :wat::core::i64])

(:wat::core::extend-type :probe::Mem :probe::Sink
  (emit [self x] "i am a string, not an i64"))        ;; WRONG: surface says -> :i64
