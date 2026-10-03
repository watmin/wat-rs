;; probe: defclause with {:restricted-to [...]} called from an ALLOWED prefix — must check clean.
(wat.core/defclause probe/guarded
  {:restricted-to [probe]}
  ([a :- wat.type/i64] :- wat.type/i64 a))

(wat.core/defn probe/caller
  [x :- wat.type/i64]
  :- wat.type/i64
  (probe/guarded x))
