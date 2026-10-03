;; Does a param typed [I :-> O] accept a 2-arg fn? And can a param be typed generically
;; (a bare type-param W) to accept any-arity fn, then reified via fn-forms?
;; CLAIM: takes-generic(a 2-arg fn) == 7 (the fn body's own constant; takes-generic never
;; calls f — this just proves a generic-W param ACCEPTS a 2-arg fn argument at all).

(wat.core/defn probe/takes-1 [f :- [wat.type/i64 :-> wat.type/i64]] :- wat.type/i64
  (f 3))

(wat.core/defn probe/takes-generic :- [W] [f :- W] :- wat.type/i64 7)

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [two (wat.core/fn [a :- wat.type/i64  b :- wat.type/i64] :- wat.type/i64 (wat.core/+ a b))
     ;; pass a 2-arg fn to a generic param (should be fine)
     g   (probe/takes-generic two)]
    (wat.core/do
      (wat.test/assert-eq g 7)
      (wat.kernel/println g))))
