;; first on an EMPTY Vector — must raise (runtime) or fail check (HEAD). Expect error.
(wat.core/defn p/f [] :- wat.type/i64 (wat.core/first (wat.type/Vector :- [wat.type/i64])))
