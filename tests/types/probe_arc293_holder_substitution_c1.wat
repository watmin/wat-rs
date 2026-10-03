;; tests/types/probe_arc293_holder_substitution_c1.wat — case 1: core record accepted where :wat::core::Record wanted

(wat.core/defrecord geo/Pt [x :- wat.type/i64  y :- wat.type/i64])
(wat.core/defn u/wants-record [r :- wat.type/Record] :- wat.type/Record r)
(wat.core/defn user/main [] :- wat.type/nil
  (u/wants-record (geo/Pt :x 1 :y 2))
  nil)
