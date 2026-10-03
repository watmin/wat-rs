;; THE DENIED EXPRESSION — a `match` arm that hash-destructures the subject:
;;     ({v :field} <body using v>)
;; Core supports it (receiver-polymorphic over record / struct / HashMap).
;; rete refuses it: "match map-destructure is not lowered in v1" (expr_ir.rs:670,:680).
(wat.core/defrecord md/Point [x :- wat.type/i64  y :- wat.type/i64])

(wat.core/defn md/core-side [p :- md/Point] :- wat.type/i64
  (wat.core/match p
    [{vx :x  vy :y} (wat.i64/+ vx vy)]))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/do
    (wat.kernel/println "CORE, hash-destructure match arm on a record:")
    (wat.kernel/println (md/core-side (md/Point :x 40 :y 2)))))
