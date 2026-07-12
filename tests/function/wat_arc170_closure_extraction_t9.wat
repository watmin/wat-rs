;; T9: captured struct holds Sender field (may fail to freeze — substrate may refuse).
;; If startup fails: T9 is vacuous (skip). If it succeeds: extraction must give ImpureCapture.
(:wat::core::defstruct :my::Pack
  [tx <- :wat::kernel::Sender<wat::core::i64>])
(:wat::core::defn :my::make-pack [] -> :wat::core::Fn(wat::core::i64)->wat::core::i64
  (:wat::core::let
              [[tx rx] (:wat::kernel::make-channel :wat::core::i64)
               pack (:my::Pack :tx tx)
               unused rx]
              (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64
                (:wat::core::do pack n))))
