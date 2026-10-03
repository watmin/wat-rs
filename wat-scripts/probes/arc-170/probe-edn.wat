;; CLAIM: edn/write . edn/read round-trips a Vector[i64] to data equal to the original —
;; asserted as DATA (the parsed value equals the original vector), never by comparing the
;; rendered text (coordinator correction, 255.76 weigh: "measuring strings is anti-wat").
;; The dynamic edn/read result is pinned by a TYPED CONSUMER (`check-roundtrip`'s declared
;; parameter header), never by `ann-form` ascription — ann-form is a retired crutch (arc 258);
;; "types are headers on function definitions" (coordinator correction).
(wat.core/defn probe/check-roundtrip
  [parsed :- (wat.type/Vector :- [wat.type/i64])  original :- (wat.type/Vector :- [wat.type/i64])]
  :- wat.type/nil
  (wat.test/assert-eq parsed original))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [original (wat.type/Vector :- [wat.type/i64] 2 4 6)
     rendered (wat.edn/write original)]
    (wat.core/do
      (probe/check-roundtrip (wat.edn/read rendered) original)
      (wat.kernel/println rendered))))
