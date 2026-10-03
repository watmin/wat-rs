;; NARROW: 2-param surface (Pair :- [A B]) satisfied by a PLAIN RECORD.
;; receiver clean + (Pair/fst b):i64 → the bug is the SERVICE-HANDLE satisfier
;; receiver errors ("expects :probe::Pair; got :probe::ISBox") → the bug is the 2-PARAM count
;; CLAIM: (Pair/fst b) == 42 — a plain record satisfies a 2-param surface and dispatches.
(wat.core/defsurface probe/Pair :- [A B] :nature wat.type/Struct
  :features [(fst [self :- (probe/Pair :- [A B])] :- A)])
(wat.core/defrecord probe/ISBox [i :- wat.type/i64  s :- wat.type/String])
(wat.core/extend-type probe/ISBox (probe/Pair :- [wat.type/i64 wat.type/String])
  (fst [self] (probe.ISBox/i self)))
(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [b  (probe/ISBox :i 42 :s "hi")
     ok (wat.core/ann-form (probe.Pair/fst b) wat.type/i64)]
    (wat.core/do
      (wat.test/assert-eq ok 42)
      (wat.kernel/println "narrowed"))))
