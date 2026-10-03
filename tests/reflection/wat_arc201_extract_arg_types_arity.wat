;; tests/reflection/wat_arc201_extract_arg_types_arity.wat
;; Fixture for test extract_arg_types_arity_matches_extract_arg_names.
;; Probe: 3-arg fn — extract-arg-types and extract-arg-names return same length.
(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
              [f     (wat.core/fn [a :- wat.type/i64 b :- wat.type/String c :- wat.type/i64]
                       :- wat.type/String
                       b)
               sig   (wat.runtime/signature-of-fn f)
               names (wat.runtime/extract-arg-names sig)
               tys   (wat.runtime/extract-arg-types sig)
               nlen  (wat.core/length names)
               tlen  (wat.core/length tys)]
              (wat.kernel/println nlen)
              (wat.kernel/println tlen)))
