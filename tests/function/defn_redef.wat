;; tests/function/defn_redef.wat — NEGATIVE: redef same name forbidden (T9).
;; Two defn forms with the same :user::f name. startup MUST fail with DefRedefForbidden.

(wat.core/defn user/f
  [x :- wat.type/i64] :- wat.type/i64
  x)

(wat.core/defn user/f
  [x :- wat.type/i64] :- wat.type/i64
  (wat.i64/+ x 1))
