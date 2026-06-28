;; T8: lambda captures non-portable Sender (NEGATIVE extraction) — positive STARTUP.
;; The program freezes OK; extraction of the lambda must return NonPortableCapture.
(:wat::core::defn :my::make-snd [] -> :wat::core::Fn(wat::core::i64)->wat::core::i64
  (:wat::core::let
              [[tx rx] (:wat::kernel::make-channel :wat::core::i64)
               dropped rx]
              (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64
                (:wat::core::do
                  tx
                  n))))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
