;; T13: user defmacro :my::triple that expands to i64::* n 3.
;; After macro expansion, body references only substrate — macro def must NOT land in prologue.
(wat.core/defmacro my/triple
  [x :- wat.type/AST]
  :- wat.type/AST
  (wat.core/quasiquote
    (wat.i64/* (wat.core/unquote x) 3)))
(wat.core/defn my/compute [n :- wat.type/i64] :- wat.type/i64 (my/triple n))
