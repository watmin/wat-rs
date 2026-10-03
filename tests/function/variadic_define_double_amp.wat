;; tests/function/variadic_define_double_amp.wat — NEGATIVE fixture.
;; Double `&` in define signature — Runtime MalformedForm expected.

(wat.core/defn my/bogus [& _a :- (wat.type/Vector :- [wat.type/i64]) & xs :- (wat.type/Vector :- [wat.type/i64])] :- wat.type/i64 0)

