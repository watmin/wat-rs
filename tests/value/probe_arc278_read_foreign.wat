;; tests/value/probe_arc278_read_foreign.wat — the Stone A RED gate (co-located fixture,
;; slurped via startup_beside(file!())). No placeholder main — startup_beside loads defns only.
;;
;; Stone A — `:wat::edn::read-foreign`: the dynamic EDN decode (the keystone).
;; The consumer LACKS the tag's type, so an unknown tag reconstructs as a self-describing
;; DYNAMIC value instead of raising UnknownTag — and it is RECURSIVE: a foreign record
;; CONTAINING a foreign variant field decodes all the way down.
;;
;;   #some.unknown/Rec {:kind #some.unknown.Kind/Click [42]}
;;     read-foreign → ForeignRecord {class "some.unknown/Rec", :kind → ForeignVariant …}
;;     ForeignRecord/get fr :kind → the nested ForeignVariant (a foreign value in a field)
;;     ForeignVariant/variant that → :Click  (the recursive path proven)
;;
;; Foreign accessors traffic in :wat::core::Value at the dynamic boundaries (heterogeneous
;; by nature — R7 universal top); they runtime-check the concrete foreign kind and raise a
;; clean error on a mismatch (no-hidden-failures, R41).

;; :my::compute — read-foreign the nested-unknown EDN, navigate to the NESTED variant's name.
;; Proves: read-foreign builds a ForeignRecord; get reaches the :kind field; that field is
;; itself a ForeignVariant (recursion); its variant is :Click.
(:wat::core::defn :my::compute [] -> :wat::core::Keyword
  (:wat::core::let
    [fr   (:wat::edn::read-foreign "#some.unknown/Rec {:kind #some.unknown.Kind/Click [42]}")
     kind (:wat::edn::ForeignRecord/get fr :kind)]
    (:wat::edn::ForeignVariant/variant kind)))

;; :my::strict-errors — the SAME input through STRICT read STILL raises UnknownTag.
;; The no-hidden-failures floor (R41 EGO SVM LEX) is untouched: strict is strict.
;; At green, `read` on the unknown tag raises → call_beside returns Err → the .rs expect_err's.
(:wat::core::defn :my::strict-errors [] -> :wat::core::Value
  (:wat::edn::read "#some.unknown/Rec {:kind #some.unknown.Kind/Click [42]}"))
