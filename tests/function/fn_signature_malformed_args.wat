;; tests/function/fn_signature_malformed_args.wat — NEGATIVE: malformed args-vector triple.
;; `[x <- :wat::core::i64 y]` — position 1 has only one token instead of three.
;; startup MUST fail with a clear error on the malformed triple.

(wat.core/defn my/probe [] :- wat.type/i64
  ((wat.core/fn [x :- wat.type/i64 y]
               :- wat.type/i64
               x) 7))

