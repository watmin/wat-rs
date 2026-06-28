;; tests/types/probe_arc237_sC2d_same_data.wat
;; Co-located fixture for probe_arc237_sC2d_same_data.rs
;; Loaded via startup_beside(file!()). Each named fn is exercised by its sibling Rust test.

(:wat::core::defrecord :my::Pt    [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defrecord :my::Coord [x <- :wat::core::i64  y <- :wat::core::i64])

;; comp_* — COMPOSITION directly (= (record->map …) (record->map …))
(:wat::core::defn :user::comp-same-type-equal [] -> :wat::core::bool
  (:wat::core::= (:wat::core::record->map (:my::Pt 0 0))
                 (:wat::core::record->map (:my::Pt 0 0))))

(:wat::core::defn :user::comp-cross-type-equal [] -> :wat::core::bool
  (:wat::core::= (:wat::core::record->map (:my::Pt 0 0))
                 (:wat::core::record->map (:my::Coord 0 0))))

(:wat::core::defn :user::comp-diff-value [] -> :wat::core::bool
  (:wat::core::= (:wat::core::record->map (:my::Pt 0 0))
                 (:wat::core::record->map (:my::Pt 0 9))))

;; samedata_* — the verb :wat::Record/same-data?
(:wat::core::defn :user::samedata-same-type-equal [] -> :wat::core::bool
  (:wat::Record/same-data? (:my::Pt 0 0) (:my::Pt 0 0)))

(:wat::core::defn :user::samedata-cross-type-equal [] -> :wat::core::bool
  (:wat::Record/same-data? (:my::Pt 0 0) (:my::Coord 0 0)))

(:wat::core::defn :user::samedata-diff-value [] -> :wat::core::bool
  (:wat::Record/same-data? (:my::Pt 0 0) (:my::Pt 0 9)))
