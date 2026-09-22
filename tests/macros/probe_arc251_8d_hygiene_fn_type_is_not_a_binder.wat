;; Arc 251 stone 251.8d-ii — ⛔ THE POSITIVE CONTROL, and the shape that blocked the
;; whole converted stdlib.
;;
;; This is `wat/gen.wat`'s `record` macro in miniature: a macro body that is a `let`
;; (so Gate E runs), whose quasiquote template is a `fn` with a HYGIENIC `~`-spliced
;; binder and a SYMBOL-spelled type annotation. Before 251.8d-ii the binder scan
;; stepped through the params vector one item at a time and refused every Symbol that
;; was not `->`/`<-`/`&`, so `wat.core/i64` — the TYPE — was read as a literal binder
;; and this macro was refused at definition. Under the converted stdlib that single
;; site produced 5 286 floor failures.
;;
;; It must register, expand and COMPUTE. A gate that refuses a type is not an intact
;; wall, it is a broken one.
(:wat::core::defmacro :my::typed-id
  [x <- :wat::WatAST]
  -> :wat::WatAST
  (:wat::core::let
    [cv (:wat::core::fresh-symbol "coord")]
    `((wat.core/fn [~cv :- wat.core/i64] :- wat.core/i64 ~cv) ~x)))

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::= (:my::typed-id 7) 7))
