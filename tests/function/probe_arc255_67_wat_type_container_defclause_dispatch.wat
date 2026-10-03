;; arc 255.67 — a `defclause` with two container-kind-competing clauses, one spelled with a
;; `wat.type/` head, must still dispatch on the value's ACTUAL container kind at runtime.
;; Regression for `value_matches_type_by_name`'s (`src/function/subsume.rs`) container-head
;; check: it compared the declared clause's raw, un-denoted Parametric head against the
;; canonical `wat::core::…` name with a bare `==`, so a `(wat.type/Vector :- [T])` clause
;; never matched a real Vector value — every call fell through to `NoMatchingClause`. Found by
;; this stone's own stdlib conversion of `:wat::test::spawn-peer`'s
;; `(wat.type/Vector :- [wat.type/AST])` clause.
(wat.core/defclause probe/which-container
  ([xs :- (wat.type/Vector :- [wat.type/i64])] :- wat.type/String "vector")
  ([xs :- (wat.type/List :- [wat.type/i64])] :- wat.type/String "list"))

(wat.core/defn user/vector-clause-dispatches [] :- wat.type/bool
  (wat.core/= (probe/which-container [1 2 3]) "vector"))

(wat.core/defn user/list-clause-dispatches [] :- wat.type/bool
  (wat.core/= (probe/which-container (wat.type/List :- [wat.type/i64] 1 2 3)) "list"))
