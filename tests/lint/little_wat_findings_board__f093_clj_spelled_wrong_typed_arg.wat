;; Board specimen — the-little-wat F-093: a wrongly-typed argument at a CLOJURE-SPELLED call
;; site. Their snapshot recorded this as UNCHECKED — the program ran to exit 0.
;;
;; SHAPE: CURED at `bc93125aa` (arc 8c). The checker now REFUSES the call, so this row is the
;; board's regression pin: if it ever reads (0, 0) again, the clj-spelled call site has
;; stopped being type-checked.
;; ⛔ Do NOT "fix" this file by passing an i64. A well-typed call measures nothing.
(wat.core/defn bad/ident [n :- wat.type/i64] :- wat.type/i64 n)
(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println (wat.edn/write (bad/ident "a String"))))
