;; Arc 255 STONE 71 (THE WALL) — the typed twin of `probe_stone255_71_template_untyped.wat.bad`:
;; once the macro's OWN template names its constructor's type, every use passes, same as
;; ordinary code.

(:wat::core::defmacro :user::mk-pair-typed
  [a <- wat.type/AST b <- wat.type/AST] -> wat.type/AST
  `(wat.type/Tuple :- [wat.type/i64 wat.type/i64] ~a ~b))

(:wat::core::defn :user::use-pair-typed [] -> (wat.type/Tuple :- [wat.type/i64 wat.type/i64])
  (:user::mk-pair-typed 1 2))
