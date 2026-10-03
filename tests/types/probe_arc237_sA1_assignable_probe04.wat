;; Fixture probe 04: exact :wat::core::Record into :wat::core::Record — must type-check Ok (regression).
(wat.core/defrecord my/Circle [radius :- wat.type/f64])
(wat.core/defrecord my/Square [side :- wat.type/f64])
(wat.core/defn my/needs-record [v :- wat.type/Record] :- wat.type/f64 1.0)
(wat.core/defn my/passthru [v :- wat.type/Record] :- wat.type/f64 (my/needs-record v))
