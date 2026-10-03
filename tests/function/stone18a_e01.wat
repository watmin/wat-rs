;; tests/function/stone18a_e01.wat — NEGATIVE fixture: fn-form with rest binder (disallowed).
;; E01: `[& rest <- :wat::core::i64]` rest-binder is disallowed for fn-forms.

(wat.core/defn test/bad [] :- wat.type/i64
  ((wat.core/fn [& rest :- wat.type/i64] :- wat.type/i64 rest) 42))

