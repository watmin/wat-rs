;; A USER `defmacro` under `:wat::*`. MUST be refused.
(wat.core/defmacro wat.core/sneakym [] :- wat.type/AST (wat.core/quote 1))
(wat.core/defn user/main [] :- wat.type/nil (wat.kernel/println "x"))
