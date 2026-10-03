;; A USER `defstruct` under `:wat::*`. MUST be refused.
(wat.core/defstruct wat.core/SneakyS [f :- wat.type/i64])
(wat.core/defn user/main [] :- wat.type/nil (wat.kernel/println "x"))
