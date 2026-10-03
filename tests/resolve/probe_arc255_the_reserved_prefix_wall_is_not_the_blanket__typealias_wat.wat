;; A USER `typealias` under `:wat::*`. MUST be refused.
(wat.core/typealias wat.core/SneakyA wat.type/i64)
(wat.core/defn user/main [] :- wat.type/nil (wat.kernel/println "x"))
