;; Co-located fixture for probe_arc255_81_retired_name_refuses.rs.
;; The three doors take the retired spelling as a keyword-node's string, so check
;; accepts the program and the runtime door is what refuses it.
;; The constructor form is data here (a string). Check would refuse it as a head
;; before eval_in_frozen, which is the path the test is measuring.

(wat.core/defn user/type-equal [] :- wat.type/nil
  (wat.core/type-equal?
    (wat.core/keyword-node ":wat::type::i64")
    (wat.core/keyword-node ":wat::core::i64")))

(wat.core/defn user/metadata [] :- wat.type/nil
  (wat.runtime/metadata-of (wat.core/keyword-node ":wat::core::char")))

(wat.core/defn user/render-doc [] :- wat.type/nil
  (wat.kernel/println
    (wat.core/render-doc (wat.core/keyword-node ":wat::core::char"))))

(wat.core/defn user/constructor-src [] :- wat.type/String
  "(:wat::core::PersistentVector :- [:wat::type::i64] 1)")
