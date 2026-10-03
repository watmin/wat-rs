;; CONTROL — the third lexical class: a bare LOWERCASE legacy primitive.
;; Not a type var (lowercase), not FQDN. It lives only in is_builtin_primitive.
(wat.core/defn user/f [x :- wat.type/i64] :- wat.type/i64 x)
(wat.core/defn user/main [] :- wat.type/nil (wat.kernel/println (user/f 7)))
