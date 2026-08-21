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
;;     ForeignRecord/get fr :kind → Some (the nested ForeignVariant)
;;     ForeignVariant/variant that → :Click  (the recursive path proven)
;;
;; Foreign accessors traffic in :wat::core::Value at the dynamic boundaries (heterogeneous
;; by nature — R7 universal top); they runtime-check the concrete foreign kind and raise a
;; clean error on a mismatch (no-hidden-failures, R41). `ForeignRecord/get` returns
;; Option (miss is None, never a raise — HashMap/get's contract).

;; :my::compute — read-foreign the nested-unknown EDN, navigate to the NESTED variant's name.
;; Proves: read-foreign builds a ForeignRecord; get reaches the :kind field; that field is
;; itself a ForeignVariant (recursion); its variant is :Click.
(:wat::core::defn :my::compute [] -> :wat::core::Keyword
  (:wat::core::match
    (:wat::edn::read-foreign "#some.unknown/Rec {:kind #some.unknown.Kind/Click [42]}")
    ((:wat::edn::ReadForeignOutcome::Value fr)
      (:wat::edn::ForeignVariant/variant
        (:wat::core::Option/expect
          (:wat::edn::ForeignRecord/get fr :kind)
          "nested :kind")))
    ((:wat::edn::ReadForeignOutcome::Malformed _)
      (:wat::kernel::assertion-failed! "read-foreign of well-formed EDN was :Malformed"
        :wat::core::None :wat::core::None))))

;; :my::missing-field-is-none — get of an absent key is None, never a raise.
(:wat::core::defn :my::missing-field-is-none [] -> :wat::core::bool
  (:wat::core::match
    (:wat::edn::read-foreign "#some.unknown/Rec {:kind #some.unknown.Kind/Click [42]}")
    ((:wat::edn::ReadForeignOutcome::Value fr)
      (:wat::core::match (:wat::edn::ForeignRecord/get fr :nope)
        (:wat::core::None true)
        ((:wat::core::Some _) false)))
    ((:wat::edn::ReadForeignOutcome::Malformed _) false)))

;; :my::malformed-is-malformed — junk EDN is :Malformed, never a raise.
(:wat::core::defn :my::malformed-is-malformed [] -> :wat::core::bool
  (:wat::core::match (:wat::edn::read-foreign "{not edn")
    ((:wat::edn::ReadForeignOutcome::Value _) false)
    ((:wat::edn::ReadForeignOutcome::Malformed _) true)))

;; :my::strict-errors — the SAME input through STRICT read STILL raises UnknownTag.
;; The no-hidden-failures floor (R41 EGO SVM LEX) is untouched: strict is strict.
;; At green, `read` on the unknown tag raises → call_beside returns Err → the .rs expect_err's.
(:wat::core::defn :my::strict-errors [] -> :wat::core::Value
  (:wat::edn::read "#some.unknown/Rec {:kind #some.unknown.Kind/Click [42]}"))
