(wat.core/defmacro my/probe
  [body :- wat.type/AST]
  :- wat.type/AST
  `(wat.core/do
     (wat.core/defstruct my.probe/Point
       [x :- wat.type/i64
        y :- wat.type/i64])
     ~body))

(my/probe
  (wat.core/defn my.probe/make-origin [] :- my.probe/Point (my.probe/Point :x 0 :y 0)))
