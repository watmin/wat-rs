;; SUBJECT — the builder's expression. A nested literal construction passed to a
;; parameter declared with the base enums. The nesting is not the problem and never was.
(wat.core/defn user/app-describe
  [o :- (wat.core/Option :- [(wat.core/Result :- [wat.type/i64 wat.type/String])])]
  :- wat.type/nil (wat.kernel/println "ok"))
(wat.core/defn user/main [] :- wat.type/nil
  (user/app-describe
    (wat.core/Option.Some {:value (wat.core/Result.Err {:error "inner-boom"})})))
