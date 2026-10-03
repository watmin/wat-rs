;; tests/function/variadic_define_arity_err.wat — NEGATIVE fixture.
;; Caller omits the required fixed param `init`; type checker surfaces ArityMismatch.

(wat.core/defn my/sum-of [init :- wat.type/i64 & xs :- (wat.type/Vector :- [wat.type/i64])] :- wat.type/i64
  (wat.core/foldl
              (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64
                (wat.i64/+ acc x))
              init
              xs))

(wat.core/defn user/compute [] :- wat.type/i64 (my/sum-of))

